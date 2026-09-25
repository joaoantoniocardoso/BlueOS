import { describe, expect, it } from 'vitest'

import { isKeyframe, iterateNalUnits } from '@/libs/mcap/logic/codec'

import { SAMPLE_H264_KEYFRAME } from './build-mcap'

describe('mcap codec helpers', () => {
  it('detects H.264 keyframes in Annex-B buffers', () => {
    const units = iterateNalUnits(SAMPLE_H264_KEYFRAME, false)
    expect(units.length).toBeGreaterThan(0)
    expect(isKeyframe(SAMPLE_H264_KEYFRAME, 'h264')).toBe(true)
  })
})
