/* eslint-disable import/no-extraneous-dependencies */
import { parse as parseMessageDefinition } from '@foxglove/rosmsg'
import { MessageReader } from '@foxglove/rosmsg2-serialization'

import { VideoFormat } from '@/libs/mcap/logic/codec'
import { inlineRos2Time, parseCompressedVideo } from '@/libs/mcap/logic/video-track'

export interface CompressedVideoFrame {
  data: Uint8Array
  format: VideoFormat
  timestampSeconds: number
}

/** Reads CompressedVideo samples; a sample it cannot read gives null so the picture stays on the last good frame. */
export function createVideoFrameReader(schemaText: string): (payload: Uint8Array) => CompressedVideoFrame | null {
  const messageReader = new MessageReader(parseMessageDefinition(inlineRos2Time(schemaText), { ros2: true }))
  return (payload) => {
    try {
      return parseCompressedVideo(messageReader, payload)
    } catch {
      return null
    }
  }
}
