import {
  describe, expect, it, vi,
} from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { index, library } from '@/libs/blueos-api/services/recorder'
import { createRecorderClient } from '@/libs/recorder/client'
import { createCachedRecordingIndexSource } from '@/libs/recorder/index-source'

import FakeTransport from '../blueos-api/fake-transport'

function indexResponse(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    size: 500,
    offset: 0,
    closed: true,
    chunks: [],
    message_counts: [],
    records: [],
    ...overrides,
  }
}

describe('recording index source', () => {
  it('queries the recorder index endpoint and maps pages', async () => {
    const transport = new FakeTransport()
    const source = createCachedRecordingIndexSource(transport, 'live.mcap', () => 500)

    const pagePromise = source.page(0, 100)
    const query = await transport.nextQuery()
    expect(query.key).toBe(index.key)
    query.reply({
      kind: 'sample',
      sample: {
        key: index.key,
        payload: encodeCdr(index.responseSchema, indexResponse({ offset: 0 })),
        encoding: cdrEncoding(index.responseSchema),
      },
    })

    const page = await pagePromise
    expect(page.size).toBe(500)
    expect(page.closed).toBe(true)
  })

  it('reuses cached pages until the library size changes', async () => {
    const transport = new FakeTransport()
    let sizeBytes = 100
    const source = createCachedRecordingIndexSource(transport, 'live.mcap', () => sizeBytes)

    const first = source.page(0, 50)
    const firstQuery = await transport.nextQuery()
    firstQuery.reply({
      kind: 'sample',
      sample: {
        key: index.key,
        payload: encodeCdr(index.responseSchema, indexResponse({ size: 100 })),
        encoding: cdrEncoding(index.responseSchema),
      },
    })
    await first

    await expect(source.page(0, 50)).resolves.toMatchObject({ size: 100 })

    sizeBytes = 200
    const second = source.page(0, 50)
    const secondQuery = await transport.nextQuery()
    secondQuery.reply({
      kind: 'sample',
      sample: {
        key: index.key,
        payload: encodeCdr(index.responseSchema, indexResponse({ size: 200 })),
        encoding: cdrEncoding(index.responseSchema),
      },
    })
    await expect(second).resolves.toMatchObject({ size: 200 })
  })

  it('exposes indexSource from the recorder client using library sizes', async () => {
    vi.useFakeTimers()
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const libraryWatch = client.watchLibrary(() => undefined)
    const stateQuery = await transport.nextQuery()
    stateQuery.reply({
      kind: 'sample',
      sample: {
        key: library.key,
        payload: encodeCdr(library.messageSchema, {
          files: [{
            path: 'live.mcap',
            name: 'live.mcap',
            size_bytes: 50,
            created: { sec: 1, nanosec: 0 },
            state: 1,
            repair_bytes_processed: 0,
            repair_total_bytes: 0,
            repair_bytes_per_second: 0,
            repair_error: '',
            allowed_operations: [],
          }],
        }),
        encoding: cdrEncoding(library.messageSchema),
      },
    })
    await libraryWatch

    const source = client.recordingIndexSource('live.mcap')
    const pagePromise = source.page(0, 10)
    const indexQuery = await transport.nextQuery()
    indexQuery.reply({
      kind: 'sample',
      sample: {
        key: index.key,
        payload: encodeCdr(index.responseSchema, indexResponse({ size: 50 })),
        encoding: cdrEncoding(index.responseSchema),
      },
    })
    await expect(pagePromise).resolves.toMatchObject({ size: 50 })
    vi.useRealTimers()
  })
})
