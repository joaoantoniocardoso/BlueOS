/* eslint-disable import/no-extraneous-dependencies */
import { CATALOG_SCHEMAS } from '@blueos-idl/catalog'
import { describe, expect, it } from 'vitest'

import { encodeCdrWithSchema } from '@/libs/blueos-api/cdr'
import { createVideoFrameReader } from '@/libs/zenoh-inspector/adapters/video-frame-reader'

import { SAMPLE_H264_KEYFRAME } from '../mcap/build-mcap'

const SCHEMA_NAME = 'foxglove_msgs/msg/CompressedVideo'
const SCHEMA_TEXT = CATALOG_SCHEMAS[SCHEMA_NAME]

function sample(format: string, data: Uint8Array): Uint8Array {
  return encodeCdrWithSchema(SCHEMA_NAME, SCHEMA_TEXT, {
    timestamp: { sec: 12, nanosec: 500_000_000 },
    frame_id: 'camera',
    data,
    format,
  })
}

describe('video frame reader', () => {
  it('reads the Annex-B data and codec of a CompressedVideo sample', () => {
    const readFrame = createVideoFrameReader(SCHEMA_TEXT)

    const frame = readFrame(sample('h264', SAMPLE_H264_KEYFRAME))

    expect(frame?.format).toBe('h264')
    expect(Array.from(frame?.data ?? [])).toEqual(Array.from(SAMPLE_H264_KEYFRAME))
  })

  it('reads the source timestamp of a sample', () => {
    const readFrame = createVideoFrameReader(SCHEMA_TEXT)

    expect(readFrame(sample('h264', SAMPLE_H264_KEYFRAME))?.timestampSeconds).toBe(12.5)
  })

  it('reads H.265 samples too', () => {
    const readFrame = createVideoFrameReader(SCHEMA_TEXT)

    expect(readFrame(sample('h265', SAMPLE_H264_KEYFRAME))?.format).toBe('h265')
  })

  it('skips a codec the players cannot decode', () => {
    const readFrame = createVideoFrameReader(SCHEMA_TEXT)

    expect(readFrame(sample('vp9', SAMPLE_H264_KEYFRAME))).toBeNull()
  })

  it('skips a payload that is not a CompressedVideo sample and keeps reading', () => {
    const readFrame = createVideoFrameReader(SCHEMA_TEXT)

    expect(readFrame(Uint8Array.from([0, 1, 0, 0, 255]))).toBeNull()
    expect(readFrame(sample('h264', SAMPLE_H264_KEYFRAME))?.format).toBe('h264')
  })
})
