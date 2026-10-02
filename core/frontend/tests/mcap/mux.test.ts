import { describe, expect, it } from 'vitest'

import { probeDecoderInfo } from '@/libs/mcap/logic/mux'

import { SAMPLE_H264_KEYFRAME } from './build-mcap'

describe('mcap mux', () => {
  it('probes H.264 decoder info from Annex-B keyframes', () => {
    const info = probeDecoderInfo('h264', SAMPLE_H264_KEYFRAME)
    expect(info.format).toBe('h264')
    expect(info.width).toBeGreaterThan(0)
    expect(info.height).toBeGreaterThan(0)
    expect(info.codec.length).toBeGreaterThan(0)
  })
})
