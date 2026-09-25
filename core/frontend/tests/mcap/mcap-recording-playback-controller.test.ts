import {
  describe, expect, it, vi,
} from 'vitest'

import { McapRecordingPlaybackController } from '@/libs/mcap'

describe('McapRecordingPlaybackController', () => {
  it('tracks written size growth for ongoing recordings', async () => {
    const onState = vi.fn()
    const controller = new McapRecordingPlaybackController({
      url: 'http://example/recording.mcap',
      ongoing: true,
      writtenSizeBytes: 100,
      callbacks: {
        onState,
        onBusy: () => undefined,
        onSummary: () => undefined,
        onMp4Saved: () => undefined,
      },
    })
    controller.onWrittenSizeBytes(100)
    controller.onWrittenSizeBytes(200)
    expect(controller.getState().lastKnownWrittenSize).toBe(200)
    controller.destroy()
  })

  it('toggles stream selection', () => {
    const controller = new McapRecordingPlaybackController({
      url: 'http://example/recording.mcap',
      ongoing: false,
      callbacks: {
        onState: () => undefined,
        onBusy: () => undefined,
        onSummary: () => undefined,
        onMp4Saved: () => undefined,
      },
    })
    controller.setSelectedChannelIds([1, 2])
    controller.toggleStream(1)
    expect(controller.getState().selectedChannelIds).toEqual([2])
    controller.destroy()
  })
})
