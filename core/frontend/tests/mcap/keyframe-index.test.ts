import { describe, expect, it } from 'vitest'

import { KeyframeLocator } from '@/libs/mcap/logic/keyframe-index'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'
import { listVideoTracks } from '@/libs/mcap/logic/video-track'

import { buildSizedVideoMcap } from './build-mcap'
import MemoryByteSource from './memory-byte-source'

describe('KeyframeLocator', () => {
  it('finds a forward keyframe hint from message index sizes', async () => {
    const bytes = await buildSizedVideoMcap(18)
    const reader = await McapIndexedReader.open(new MemoryByteSource(bytes))
    const track = listVideoTracks(reader)[0]
    function chunkPositions(): number[] {
      return reader.chunkIndexesForChannel(track.channelId)
    }
    const locator = new KeyframeLocator(reader, track.channelId, chunkPositions)
    const hint = await locator.findForward(0, 4)
    expect(hint).not.toBeNull()
    expect(hint?.logTime).toBeGreaterThanOrEqual(reader.summary.startTime)
  })
})
