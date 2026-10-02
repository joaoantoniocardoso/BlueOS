import { describe, expect, it } from 'vitest'

import type { VideoTrack } from '@/libs/mcap'
import {
  clipExportRange,
  formatPlaybackPosition,
  gridColumnCount,
  recordingNameFromUrl,
  visibleTracks,
} from '@/libs/mcap'

function track(channelId: number): VideoTrack {
  return {
    channelId,
    topic: 't',
    name: 'n',
    frameCount: 1,
    coverage: [{ start: 0, end: 10 }],
    codec: 'avc1',
    width: 640,
    height: 480,
  }
}

describe('playback-ui', () => {
  it('formats playback positions', () => {
    expect(formatPlaybackPosition(125)).toBe('02:05')
  })

  it('filters visible tracks', () => {
    const tracks = [track(1), track(2)]
    expect(visibleTracks(tracks, [2]).map((entry) => entry.channelId)).toEqual([2])
  })

  it('builds clip export ranges', () => {
    expect(clipExportRange(false, [0, 5], 10)).toBeNull()
    expect(clipExportRange(true, [1, 9.95], 10)?.endSeconds).toBe(Infinity)
  })

  it('derives recording names from URLs', () => {
    expect(recordingNameFromUrl('/userdata/recorder/folder/file.mcap')).toBe('file')
  })

  it('caps grid columns on small screens', () => {
    expect(gridColumnCount(4, true)).toBe(1)
    expect(gridColumnCount(4, false)).toBe(2)
  })
})
