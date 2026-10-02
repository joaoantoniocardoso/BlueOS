import { describe, expect, it } from 'vitest'

import {
  createMemoryThumbnailCache,
  type ThumbnailCacheKey,
} from '@/libs/mcap/adapters/thumbnail-cache'

function cacheKey(overrides: Partial<ThumbnailCacheKey> = {}): ThumbnailCacheKey {
  return {
    path: 'live.mcap',
    sizeBytes: 100,
    created: 1_700_000_000,
    ...overrides,
  }
}

describe('memory thumbnail cache', () => {
  it('returns a hit after set and misses when the file size changes', async () => {
    const cache = createMemoryThumbnailCache(4)
    const blob = new Blob([1, 2, 3], { type: 'image/jpeg' })

    await cache.set(cacheKey(), blob)
    await expect(cache.get(cacheKey())).resolves.toEqual(blob)
    await expect(cache.get(cacheKey({ sizeBytes: 101 }))).resolves.toBeNull()
  })

  it('evicts the least recently used entry when it is full', async () => {
    const cache = createMemoryThumbnailCache(2)
    await cache.set(cacheKey({ path: 'a.mcap' }), new Blob([1]))
    await cache.set(cacheKey({ path: 'b.mcap' }), new Blob([2]))
    await cache.get(cacheKey({ path: 'a.mcap' }))
    await cache.set(cacheKey({ path: 'c.mcap' }), new Blob([3]))

    await expect(cache.get(cacheKey({ path: 'b.mcap' }))).resolves.toBeNull()
    await expect(cache.get(cacheKey({ path: 'a.mcap' }))).resolves.not.toBeNull()
    await expect(cache.get(cacheKey({ path: 'c.mcap' }))).resolves.not.toBeNull()
  })
})
