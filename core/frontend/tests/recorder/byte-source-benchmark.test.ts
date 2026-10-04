import {
  afterEach, describe, expect, it, vi,
} from 'vitest'

import { benchmarkRecordingByteSource } from '@/libs/recorder/byte-source-benchmark'
import { RECORDS_BYTE_SOURCE_STORAGE_KEY } from '@/libs/recorder/preferences'

import { buildIndexedVideoMcap } from '../mcap/build-mcap'

/** Serves `bytes` to `HttpByteSource` the way nginx answers its range requests, and counts the bytes served. */
function serveOverHttp(bytes: Uint8Array): { served: number } {
  const counter = { served: 0 }
  vi.stubGlobal('fetch', vi.fn(async (_url: string, init?: RequestInit) => {
    const range = (init?.headers as Record<string, string>).Range
    const [, first, last] = /^bytes=(\d*)-(\d*)$/.exec(range) ?? []
    const start = first === '' ? Math.max(0, bytes.length - Number(last)) : Number(first)
    const end = first === '' || last === '' ? bytes.length - 1 : Math.min(Number(last), bytes.length - 1)
    counter.served += end + 1 - start
    return new Response(bytes.slice(start, end + 1), {
      status: 206,
      headers: { 'Content-Range': `bytes ${start}-${end}/${bytes.length}` },
    })
  }))
  return counter
}

describe('benchmarkRecordingByteSource', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('times the player reading a recording through the chosen source, then restores the stored source', async () => {
    const items: Record<string, string> = { [RECORDS_BYTE_SOURCE_STORAGE_KEY]: 'zenoh' }
    vi.stubGlobal('window', {
      localStorage: {
        getItem: (key: string) => items[key] ?? null,
        setItem: (key: string, value: string) => { items[key] = value },
        removeItem: (key: string) => { delete items[key] },
      },
    })
    const recording = await buildIndexedVideoMcap(40)
    const http = serveOverHttp(recording)

    const result = await benchmarkRecordingByteSource('indexed.mcap', 'http', 1)

    expect(result.source).toBe('http')
    expect(result.path).toBe('indexed.mcap')
    expect(result.sequentialBytes).toBeGreaterThan(0)
    expect(result.sequentialBytes).toBeLessThanOrEqual(recording.length)
    expect(http.served).toBeGreaterThan(result.sequentialBytes)
    expect(items[RECORDS_BYTE_SOURCE_STORAGE_KEY]).toBe('zenoh')
  })
})
