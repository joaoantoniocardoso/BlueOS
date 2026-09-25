import { describe, expect, it } from 'vitest'

import MemoryByteSource from '@/libs/mcap/adapters/memory-byte-source'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'
import { listVideoTracks } from '@/libs/mcap/logic/video-track'

import { buildIndexedVideoMcap } from './build-mcap'

describe('McapIndexedReader', () => {
  it('opens a small indexed recording from memory', async () => {
    const bytes = await buildIndexedVideoMcap(3)
    const source = new MemoryByteSource(bytes)
    const reader = await McapIndexedReader.open(source)
    expect(reader.summary.channels.size).toBe(1)
    const tracks = listVideoTracks(reader)
    expect(tracks).toHaveLength(1)
    expect(tracks[0].frameCount).toBe(3)
    expect(source.bytesRead).toBeGreaterThan(0)
  })

  it('reads metadata only with a smaller byte budget', async () => {
    const bytes = await buildIndexedVideoMcap(2)
    const source = new MemoryByteSource(bytes)
    await McapIndexedReader.open(source, { metadataOnly: true })
    const metadataBytes = source.bytesRead
    source.bytesRead = 0
    await McapIndexedReader.open(source)
    expect(source.bytesRead).toBeGreaterThanOrEqual(metadataBytes)
  })
})
