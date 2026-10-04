import { describe, expect, it } from 'vitest'

import VideoFrameStream from '@/libs/mcap/logic/frame-stream'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'
import { listVideoTracks } from '@/libs/mcap/logic/video-track'

import { buildGopVideoMcap } from './build-mcap'
import MemoryByteSource from './memory-byte-source'

async function openLoggingReads(): Promise<{ stream: VideoFrameStream, chunks: number[], chunkReads: () => number[] }> {
  const source = new MemoryByteSource(await buildGopVideoMcap())
  const reader = await McapIndexedReader.open(source)
  const track = listVideoTracks(reader)[0]
  const chunks = reader.chunkIndexesForChannel(track.channelId)
    .map((index) => reader.summary.chunkIndexes[index].offset)
  const offsets: number[] = []
  const read = source.read.bind(source)
  source.read = (offset, length) => {
    offsets.push(offset)
    return read(offset, length)
  }
  return {
    stream: new VideoFrameStream(reader, track),
    chunks,
    chunkReads: () => offsets.filter((offset) => chunks.includes(offset)),
  }
}

describe('VideoFrameStream', () => {
  it('asks for the next chunk while reading on through a chunk', async () => {
    const { stream, chunks, chunkReads } = await openLoggingReads()
    expect(chunks.length).toBeGreaterThanOrEqual(3)

    while (!chunkReads().includes(chunks[1])) {
      // eslint-disable-next-line no-await-in-loop
      await stream.next()
    }

    expect(chunkReads()).toEqual([chunks[0], chunks[1], chunks[2]])
    // eslint-disable-next-line no-await-in-loop
    while (await stream.next()) { /* read to the end */ }
    expect(chunkReads()).toEqual(chunks)
  })

  it('reads only the chunk a seek lands on for one frame', async () => {
    const { stream, chunkReads } = await openLoggingReads()

    await stream.seekToKeyframe(12)
    await stream.next()

    expect(chunkReads()).toHaveLength(1)
  })
})
