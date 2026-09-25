import { describe, expect, it } from 'vitest'

import MemoryByteSource from '@/libs/mcap/adapters/memory-byte-source'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'
import { listVideoTracks } from '@/libs/mcap/logic/video-track'

import { buildIndexedVideoMcap } from './build-mcap'

describe('video frame sequences', () => {
  it('reads monotonic sequence numbers from chunk messages', async () => {
    const bytes = await buildIndexedVideoMcap(5)
    const reader = await McapIndexedReader.open(new MemoryByteSource(bytes))
    const track = listVideoTracks(reader)[0]
    const chunkIndexes = reader.chunkIndexesForChannel(track.channelId)
    expect(chunkIndexes.length).toBeGreaterThan(0)
    const messages = (
      await Promise.all(chunkIndexes.map((index) => reader.readChunkMessages(index, track.channelId)))
    ).flat()
    const sequences = messages.map((message) => message.sequence)
    expect(sequences.length).toBe(5)
    expect(sequences).toEqual([0, 1, 2, 3, 4])
  })
})
