import {
  beforeEach, describe, expect, it, vi,
} from 'vitest'

import { extractMcapThumbnail } from '@/libs/mcap/adapters/thumbnail'
import { loadRecordingThumbnail } from '@/libs/recorder/thumbnail-loader'

import MemoryByteSource from '../mcap/memory-byte-source'

vi.mock('@/libs/mcap/adapters/thumbnail-cache', () => ({
  getCachedThumbnail: vi.fn(async () => null),
  setCachedThumbnail: vi.fn(async () => undefined),
}))
vi.mock('@/libs/mcap/adapters/thumbnail', () => ({ extractMcapThumbnail: vi.fn() }))

const extract = vi.mocked(extractMcapThumbnail)
const sources = new Map<string, MemoryByteSource>()

function load(name: string, signal?: AbortSignal): Promise<Blob | null> {
  const source = new MemoryByteSource(new Uint8Array())
  sources.set(name, source)
  return loadRecordingThumbnail({
    source,
    cacheKey: { path: name, sizeBytes: 1, created: 1 },
    signal,
  })
}

async function settle(): Promise<void> {
  await new Promise((resolve) => { setTimeout(resolve, 0) })
}

describe('loadRecordingThumbnail', () => {
  beforeEach(() => {
    extract.mockReset()
  })

  it('pulls two thumbnails over the link at once, and drops a waiting one that is canceled', async () => {
    const finishes: ((blob: Blob | null) => void)[] = []
    extract.mockImplementation(() => new Promise((resolve) => { finishes.push(resolve) }))
    const canceled = new AbortController()

    const loads = [load('a.mcap'), load('b.mcap'), load('c.mcap', canceled.signal), load('d.mcap')]
    await settle()
    expect(extract).toHaveBeenCalledTimes(2)

    canceled.abort()
    await expect(loads[2]).rejects.toMatchObject({ name: 'AbortError' })
    finishes[0](null)
    await settle()
    expect(extract).toHaveBeenCalledTimes(3)
    expect(extract.mock.calls[2][0]).toBe(sources.get('d.mcap'))

    finishes[1](null)
    finishes[2](null)
    await expect(Promise.all([loads[0], loads[1], loads[3]])).resolves.toEqual([null, null, null])
  })
})
