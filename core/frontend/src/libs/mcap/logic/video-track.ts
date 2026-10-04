/** Discovery and decoding of `foxglove.CompressedVideo` streams stored in an MCAP recording. */
import { parse as parseMessageDefinition } from '@foxglove/rosmsg'
import { MessageReader } from '@foxglove/rosmsg2-serialization'

import { VideoFormat } from './codec'
import {
  McapChannel, McapChunkIndex, McapIndexedReader, McapMessage,
} from './reader'

export const COMPRESSED_VIDEO_SCHEMA = 'foxglove.CompressedVideo'

const SUPPORTED_FORMATS: VideoFormat[] = ['h264', 'h265']

// The published schema refers to builtin_interfaces/Time, which is not included in the file. Since
// the CDR layout of that message is just the two fields, inlining them keeps the reader self
// contained.
const TIME_DEFINITION = /^\s*builtin_interfaces\/Time\s+timestamp\s*$/m
export const INLINE_ROS2_TIME_FIELDS = 'int32 sec\nuint32 nanosec'

export interface TimeRange {
  start: number
  end: number
}

export interface VideoTrack {
  channelId: number
  topic: string
  /** Stream name as configured in the video manager, derived from `video/<name>/stream`. */
  name: string
  frameCount: number
  /** Seconds from the start of the recording where this stream has frames. */
  coverage: TimeRange[]
}

/** Adjacent chunks closer than this are treated as one stretch of video. */
const COVERAGE_MERGE_SECONDS = 2

export function mergeTimeRanges(ranges: TimeRange[]): TimeRange[] {
  if (ranges.length === 0) {
    return []
  }
  const ordered = [...ranges].sort((left, right) => left.start - right.start)
  const merged: TimeRange[] = [{ ...ordered[0] }]
  for (const range of ordered.slice(1)) {
    const last = merged[merged.length - 1]
    if (range.start <= last.end + COVERAGE_MERGE_SECONDS) {
      last.end = Math.max(last.end, range.end)
    } else {
      merged.push({ ...range })
    }
  }
  return merged
}

export function timeRangesCover(ranges: TimeRange[], seconds: number): boolean {
  return ranges.some((range) => seconds >= range.start && seconds <= range.end)
}

/** Chunks that may hold the channel: all of them when the recording has no message index to tell. */
function chunksHolding(reader: McapIndexedReader, channelId: number): McapChunkIndex[] {
  return reader.summary.chunkIndexes.filter((chunk) => (chunk.channelIds.length === 0
    || chunk.channelIds.includes(channelId)) && (reader.channelSpansIn(chunk)?.has(channelId) ?? true))
}

/**
 * Where the channel has messages. Chunks whose message index was read (see `loadFrameAccurateCoverage`) count from
 * the channel's first message in them to the end of its last; the others count whole.
 */
export function coverageForChannel(reader: McapIndexedReader, channelId: number): TimeRange[] {
  const { startTime: origin, endTime } = reader.summary
  const ranges: TimeRange[] = []
  for (const chunk of chunksHolding(reader, channelId)) {
    const span = reader.channelSpansIn(chunk)?.get(channelId)
    let { startTime: start, endTime: end } = chunk
    if (span) {
      start = span.firstLogTime
      // A frame stays on screen until the next one, so the last one lasts as long as the interval before it.
      const shownUntil = span.lastLogTime + span.lastInterval
      end = shownUntil < endTime ? shownUntil : endTime
    }
    ranges.push({ start: Number(start - origin) / 1e9, end: Number(end - origin) / 1e9 })
  }
  const merged = mergeTimeRanges(ranges)
  if (merged.length > 0) {
    return merged
  }
  if ((reader.summary.messageCountByChannel.get(channelId) ?? 0n) <= 0n) {
    return []
  }
  const duration = Number(endTime - origin) / 1e9
  return duration > 0 ? [{ start: 0, end: duration }] : []
}

/**
 * Reads the message index of the chunks where a video stream's coverage starts or ends, so that it runs from the
 * stream's first frame to the end of its last instead of to the edges of those chunks. That is two small reads per
 * stretch of video, each done once. Returns true when it read anything.
 */
export async function loadFrameAccurateCoverage(reader: McapIndexedReader, signal?: AbortSignal): Promise<boolean> {
  let read = false
  // A chunk listed for a channel can turn out to hold none of its messages, which makes the next one an edge.
  for (;;) {
    const edges = new Set<McapChunkIndex>()
    for (const channel of reader.channelsBySchemaName(COMPRESSED_VIDEO_SCHEMA)) {
      const holding = chunksHolding(reader, channel.id)
      holding.forEach((chunk, index) => {
        const previous = holding[index - 1]
        const next = holding[index + 1]
        if (!previous || Number(chunk.startTime - previous.endTime) / 1e9 > COVERAGE_MERGE_SECONDS
          || !next || Number(next.startTime - chunk.endTime) / 1e9 > COVERAGE_MERGE_SECONDS) {
          edges.add(chunk)
        }
      })
    }
    const unread = [...edges].filter((chunk) => chunk.messageIndexLength > 0 && !reader.channelSpansIn(chunk))
    if (unread.length === 0) {
      return read
    }
    // eslint-disable-next-line no-await-in-loop
    await Promise.all(unread.map((chunk) => reader.loadChannelSpans(chunk, signal)))
    read = true
  }
}

export interface VideoFrame {
  /** MCAP log time in nanoseconds, which is also the time base used by the chunk index. */
  logTime: bigint
  /** Recorder counter for this channel. Gaps mean frames that never reached the recording. */
  sequence: number
  format: VideoFormat
  data: Uint8Array
}

interface CompressedVideoMessage {
  data: Uint8Array
  format: string
  sec?: number
  nanosec?: number
}

export function parseCompressedVideo(
  messageReader: MessageReader,
  bytes: Uint8Array,
): { data: Uint8Array, format: VideoFormat, timestampSeconds: number } {
  const message = messageReader.readMessage(bytes) as CompressedVideoMessage
  if (!SUPPORTED_FORMATS.includes(message.format as VideoFormat)) {
    throw new Error(
      `Unsupported video format '${message.format}'. Only ${SUPPORTED_FORMATS.join(' and ')} can be played.`,
    )
  }
  return {
    data: message.data,
    format: message.format as VideoFormat,
    timestampSeconds: (message.sec ?? 0) + (message.nanosec ?? 0) / 1e9,
  }
}

export function decodeSchemaDefinition(data: Uint8Array): string {
  return new TextDecoder().decode(data).replace(TIME_DEFINITION, INLINE_ROS2_TIME_FIELDS)
}

export function listVideoTracks(reader: McapIndexedReader): VideoTrack[] {
  return reader.channelsBySchemaName(COMPRESSED_VIDEO_SCHEMA).map((channel) => ({
    channelId: channel.id,
    topic: channel.topic,
    name: channel.topic.replace(/^video\//, '').replace(/\/stream$/, ''),
    frameCount: Number(reader.summary.messageCountByChannel.get(channel.id) ?? 0n),
    coverage: coverageForChannel(reader, channel.id),
  }))
}

export class VideoFrameDecoder {
  private constructor(private messageReader: MessageReader) {}

  static create(reader: McapIndexedReader, channel: McapChannel): VideoFrameDecoder {
    if (channel.messageEncoding !== 'cdr') {
      throw new Error(`Unsupported video message encoding: '${channel.messageEncoding}'.`)
    }
    const schema = reader.summary.schemas.get(channel.schemaId)
    if (!schema) {
      throw new Error(`Recording is missing the schema for ${channel.topic}.`)
    }
    const text = decodeSchemaDefinition(schema.data)
    return new VideoFrameDecoder(new MessageReader(parseMessageDefinition(text, { ros2: true })))
  }

  decode(message: McapMessage): VideoFrame {
    const parsed = parseCompressedVideo(this.messageReader, message.data)
    return {
      logTime: message.logTime, sequence: message.sequence, format: parsed.format, data: parsed.data,
    }
  }
}
