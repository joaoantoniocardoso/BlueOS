import {
  afterEach, describe, expect, it, vi,
} from 'vitest'

import { HttpByteSource } from '@/libs/mcap/adapters/http-byte-source'

function rangeResponse(
  status: number,
  body: Uint8Array,
  contentRange?: string,
): Response {
  return new Response(body, {
    status,
    headers: contentRange ? { 'Content-Range': contentRange } : {},
  })
}

describe('HttpByteSource', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('uses HTTP Range on reads and accepts 206 responses', async () => {
    const fetchMock = vi.fn(async (_url: string, init?: RequestInit) => {
      const range = (init?.headers as Record<string, string>).Range
      if (range === 'bytes=-4096') {
        return rangeResponse(206, new Uint8Array([1, 2, 3]), 'bytes 997-999/1000')
      }
      if (range === 'bytes=10-19') {
        return rangeResponse(206, new Uint8Array(10).fill(7), 'bytes 10-19/1000')
      }
      throw new Error(`unexpected range ${range}`)
    })
    vi.stubGlobal('fetch', fetchMock)

    const source = new HttpByteSource('http://vehicle/userdata/recorder/live.mcap')
    await expect(source.size()).resolves.toBe(1000)
    const slice = await source.read(10, 10)
    expect(slice).toHaveLength(10)
    expect(slice.every((byte) => byte === 7)).toBe(true)
    expect(source.bytesRead).toBe(13)

    const tailCall = fetchMock.mock.calls.find((call) => {
      const init = call[1] as RequestInit | undefined
      return (init?.headers as Record<string, string>).Range === 'bytes=-4096'
    })
    expect(tailCall).toBeDefined()
    const readCall = fetchMock.mock.calls.find((call) => {
      const init = call[1] as RequestInit | undefined
      return (init?.headers as Record<string, string>).Range === 'bytes=10-19'
    })
    expect(readCall?.[0]).toContain('range=bytes%3D10-19')
  })

  it('rejects servers that respond with 200 instead of 206', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => rangeResponse(200, new Uint8Array([0]))))

    const source = new HttpByteSource('http://vehicle/userdata/recorder/live.mcap')
    await expect(source.read(0, 1)).rejects.toThrow(/range requests/)
  })
})
