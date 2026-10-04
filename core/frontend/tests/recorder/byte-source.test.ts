import {
  afterEach, describe, expect, it, vi,
} from 'vitest'

import { decodeCdr, encodeCdr } from '@/libs/blueos-api/cdr'
import { QueryFailedError } from '@/libs/blueos-api/errors'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { bytes } from '@/libs/blueos-api/services/recorder'
import type { Transport } from '@/libs/blueos-api/transport'
import { HttpByteSource } from '@/libs/mcap/adapters/http-byte-source'
import { recordingByteSource, ZenohByteSource } from '@/libs/recorder/byte-source'
import { DEFAULT_RECORDING_HTTP_PREFIX } from '@/libs/recorder/constants'
import { recordingDownloadUrl } from '@/libs/recorder/url'

import FakeTransport, { type PendingQuery } from '../blueos-api/fake-transport'

vi.mock('@/libs/zenoh', () => ({ default: { getSession: async () => ({}) } }))

const MEBIBYTE = 1024 * 1024

function requestOf(query: PendingQuery): { path: string, offset: number, length: number, from_end: boolean } {
  if (!query.body) {
    throw new Error('expected a request body')
  }
  return decodeCdr(bytes.requestSchema, query.body.payload)
}

function answer(query: PendingQuery, size: number, data: Uint8Array): void {
  query.reply({
    kind: 'sample',
    sample: {
      key: bytes.key,
      payload: encodeCdr(bytes.responseSchema, { size, data }),
      encoding: cdrEncoding(bytes.responseSchema),
    },
  })
}

/** Answers every query with bytes whose value is their offset in MiB, until `count` queries were answered. */
async function answerRanges(transport: FakeTransport, count: number, size: number): Promise<void> {
  for (let answered = 0; answered < count; answered += 1) {
    // eslint-disable-next-line no-await-in-loop
    const query = await transport.nextQuery()
    const { offset, length } = requestOf(query)
    answer(query, size, new Uint8Array(length).fill(offset / MEBIBYTE))
  }
}

describe('ZenohByteSource', () => {
  it('reads the size and the tail in one query, then a range with the bytes Query', async () => {
    const transport = new FakeTransport()
    const source = new ZenohByteSource(transport, 'flight/live.mcap')
    const tail = new Uint8Array(4096).map((_, index) => index % 251)

    const size = source.size()
    const sizeQuery = await transport.nextQuery()
    expect(sizeQuery.key).toBe(bytes.key)
    expect(requestOf(sizeQuery)).toEqual({
      path: 'flight/live.mcap', offset: 0, length: 4096, from_end: true,
    })
    answer(sizeQuery, 10000, tail)
    await expect(size).resolves.toBe(10000)
    await expect(source.read(10000 - 22, 22)).resolves.toEqual(tail.subarray(4096 - 22))

    const read = source.read(10, 4)
    const readQuery = await transport.nextQuery()
    expect(requestOf(readQuery)).toEqual({
      path: 'flight/live.mcap', offset: 10, length: 4, from_end: false,
    })
    answer(readQuery, 12000, new Uint8Array([1, 2, 3, 4]))

    await expect(read).resolves.toEqual(new Uint8Array([1, 2, 3, 4]))
    await expect(source.size()).resolves.toBe(10000)
    expect(source.bytesRead).toBe(4096 + 4)
  })

  it('splits a large read into 1 MiB queries, four in flight, and does not queue another read behind it', async () => {
    const transport = new FakeTransport()
    let inFlight = 0
    let mostInFlight = 0
    const counting: Transport = {
      subscribe: transport.subscribe.bind(transport),
      subscribeLiveliness: transport.subscribeLiveliness.bind(transport),
      async get(key, body) {
        inFlight += 1
        mostInFlight = Math.max(mostInFlight, inFlight)
        try {
          return await transport.get(key, body)
        } finally {
          inFlight -= 1
        }
      },
    }
    const source = new ZenohByteSource(counting, 'live.mcap')

    const read = source.read(0, 5 * MEBIBYTE + 3)
    const other = source.read(9 * MEBIBYTE, 2)
    await answerRanges(transport, 7, 10 * MEBIBYTE)
    const data = await read

    expect(mostInFlight).toBe(5)
    expect(data.byteLength).toBe(5 * MEBIBYTE + 3)
    expect([0, 1, 2, 3, 4, 5].map((piece) => data[piece * MEBIBYTE])).toEqual([0, 1, 2, 3, 4, 5])
    await expect(other).resolves.toEqual(new Uint8Array([9, 9]))
    expect(source.bytesRead).toBe(5 * MEBIBYTE + 5)
  })

  it('rejects with AbortError when its signal aborts, without waiting for the reply', async () => {
    const transport = new FakeTransport()
    const source = new ZenohByteSource(transport, 'live.mcap')
    const controller = new AbortController()

    const read = source.read(0, 10, controller.signal)
    await transport.nextQuery()
    controller.abort()

    await expect(read).rejects.toMatchObject({ name: 'AbortError' })
    await expect(source.read(0, 10, controller.signal)).rejects.toMatchObject({ name: 'AbortError' })
  })

  it('rejects with the reason the recorder refused the read', async () => {
    const transport = new FakeTransport()
    const source = new ZenohByteSource(transport, 'missing.mcap')

    const read = source.read(0, 10)
    const query = await transport.nextQuery()
    query.reply({ kind: 'error', payload: new TextEncoder().encode('Recording not found.'), encoding: 'text/plain' })

    await expect(read).rejects.toBeInstanceOf(QueryFailedError)
    await expect(read).rejects.toThrow('Recording not found.')
  })
})

describe('recordingByteSource', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('reads HTTP ranges from nginx by default', async () => {
    const source = await recordingByteSource(recordingDownloadUrl('flight/live.mcap', DEFAULT_RECORDING_HTTP_PREFIX))

    expect(source).toBeInstanceOf(HttpByteSource)
  })

  it('reads the same recording with the bytes Query when the byte source is set to zenoh', async () => {
    vi.stubGlobal('window', { localStorage: { getItem: () => 'zenoh' } })

    const source = await recordingByteSource(recordingDownloadUrl('flight 2/a#b.mcap', DEFAULT_RECORDING_HTTP_PREFIX))

    expect(source).toBeInstanceOf(ZenohByteSource)
    expect((source as ZenohByteSource).path).toBe('flight 2/a#b.mcap')
  })
})
