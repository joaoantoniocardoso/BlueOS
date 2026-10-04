import { McapWriter, TempBuffer } from '@mcap/core'

import { encodeCdrWithSchema } from '@/libs/blueos-api/cdr'
import type { ByteSource } from '@/libs/mcap/logic/byte-source'
import { type McapChunkIndex, McapIndexedReader } from '@/libs/mcap/logic/reader'
import type { RecordingIndexSource } from '@/libs/mcap/logic/recording-index'

import MemoryByteSource from './memory-byte-source'

/** Minimal Annex-B IDR slice (SPS + PPS + IDR) for mux/codec probes. */
export const SAMPLE_H264_KEYFRAME = Uint8Array.from([
  0, 0, 0, 1, 0x67, 0x42, 0x00, 0x0a, 0xf8, 0x41, 0xa2,
  0, 0, 0, 1, 0x68, 0xce, 0x38, 0x80,
  0, 0, 0, 1, 0x65, 0x88, 0x84, 0x00, 0x10, 0xff, 0xfe, 0x00,
])

const COMPRESSED_VIDEO_SCHEMA = `int32 sec
uint32 nanosec
string format
uint8[] data`

/** Tiny Annex-B non-IDR slice for keyframe size contrast in tests. */
export const SAMPLE_H264_DELTA = Uint8Array.from([
  0, 0, 0, 1, 0x41, 0x9a, 0x24, 0x0c, 0x0f, 0x40,
])

export async function buildIndexedVideoMcap(messageCount = 4): Promise<Uint8Array> {
  const buffer = new TempBuffer()
  const writer = new McapWriter({ writable: buffer, chunkSize: 256 })
  await writer.start({ profile: '', library: 'blueos-test' })
  const schemaId = await writer.registerSchema({
    name: 'foxglove.CompressedVideo',
    encoding: 'ros2msg',
    data: new TextEncoder().encode(COMPRESSED_VIDEO_SCHEMA),
  })
  const channelId = await writer.registerChannel({
    schemaId,
    topic: 'video/camera/stream',
    messageEncoding: 'cdr',
    metadata: new Map(),
  })
  const baseTime = 1_000_000_000n
  for (let index = 0; index < messageCount; index += 1) {
    const logTime = baseTime + BigInt(index) * 33_000_000n
    const payload = encodeCompressedVideo('h264', SAMPLE_H264_KEYFRAME)
    await writer.addMessage({
      channelId,
      sequence: index,
      logTime,
      publishTime: logTime,
      data: payload,
    })
  }
  await writer.end()
  return buffer.get()
}

/** Alternates large keyframe payloads with small deltas so KeyframeLocator can learn a median. */
export async function buildSizedVideoMcap(messageCount = 16): Promise<Uint8Array> {
  const buffer = new TempBuffer()
  const writer = new McapWriter({ writable: buffer, chunkSize: 4096 })
  await writer.start({ profile: '', library: 'blueos-test' })
  const schemaId = await writer.registerSchema({
    name: 'foxglove.CompressedVideo',
    encoding: 'ros2msg',
    data: new TextEncoder().encode(COMPRESSED_VIDEO_SCHEMA),
  })
  const channelId = await writer.registerChannel({
    schemaId,
    topic: 'video/camera/stream',
    messageEncoding: 'cdr',
    metadata: new Map(),
  })
  const baseTime = 1_000_000_000n
  for (let index = 0; index < messageCount; index += 1) {
    const logTime = baseTime + BigInt(index) * 33_000_000n
    const videoData = index % 3 === 0
      ? Uint8Array.from([...SAMPLE_H264_KEYFRAME, ...new Uint8Array(128)])
      : SAMPLE_H264_DELTA
    const payload = encodeCompressedVideo('h264', videoData)
    await writer.addMessage({
      channelId,
      sequence: index,
      logTime,
      publishTime: logTime,
      data: payload,
    })
  }
  await writer.end()
  return buffer.get()
}

/** Frame spacing of `buildTwoTrackVideoMcap`; camera_b frames sit half way between two camera_a frames. */
export const TWO_TRACK_FRAME_NS = 500_000_000n

/**
 * Two keyframe-only video streams sharing chunks: camera_a has 41 frames over 20 s, and camera_b has
 * 21 frames that start 5 s after camera_a and stop 5 s before it. The first `cameraBDeltaFrames` frames of
 * camera_b are deltas, so its first keyframe comes that many frames after its first frame. Without
 * `useMessageIndex`, nothing tells which chunks hold which stream, so both cover the whole recording.
 */
export async function buildTwoTrackVideoMcap(cameraBDeltaFrames = 0, useMessageIndex = true): Promise<Uint8Array> {
  const buffer = new TempBuffer()
  const writer = new McapWriter({ writable: buffer, chunkSize: 1024, useMessageIndex })
  await writer.start({ profile: '', library: 'blueos-test' })
  const schemaId = await writer.registerSchema({
    name: 'foxglove.CompressedVideo',
    encoding: 'ros2msg',
    data: new TextEncoder().encode(COMPRESSED_VIDEO_SCHEMA),
  })
  const cameraA = await writer.registerChannel({
    schemaId, topic: 'video/camera_a/stream', messageEncoding: 'cdr', metadata: new Map(),
  })
  const cameraB = await writer.registerChannel({
    schemaId, topic: 'video/camera_b/stream', messageEncoding: 'cdr', metadata: new Map(),
  })
  const payload = encodeCompressedVideo('h264', SAMPLE_H264_KEYFRAME)
  const baseTime = 1_000_000_000n
  for (let index = 0; index <= 40; index += 1) {
    const logTime = baseTime + BigInt(index) * TWO_TRACK_FRAME_NS
    await writer.addMessage({
      channelId: cameraA, sequence: index, logTime, publishTime: logTime, data: payload,
    })
    if (index >= 10 && index <= 30) {
      const cameraBTime = logTime + TWO_TRACK_FRAME_NS / 2n
      const data = index - 10 < cameraBDeltaFrames ? encodeCompressedVideo('h264', SAMPLE_H264_DELTA) : payload
      await writer.addMessage({
        channelId: cameraB, sequence: index - 10, logTime: cameraBTime, publishTime: cameraBTime, data,
      })
    }
  }
  await writer.end()
  return buffer.get()
}

/**
 * A telemetry topic every 100 ms for 4 s and a video stream that only starts 2 s in, so the first chunks hold no
 * video and the video Channel record sits in a later chunk, as when a camera starts during a recording.
 */
export async function buildLateVideoMcap(): Promise<Uint8Array> {
  const buffer = new TempBuffer()
  const writer = new McapWriter({ writable: buffer, chunkSize: 256 })
  await writer.start({ profile: '', library: 'blueos-test' })
  const telemetrySchema = await writer.registerSchema({
    name: 'blueos_msgs/msg/TelemetrySample',
    encoding: 'jsonschema',
    data: new TextEncoder().encode('{"type":"object"}'),
  })
  const telemetry = await writer.registerChannel({
    schemaId: telemetrySchema, topic: '/telemetry/depth', messageEncoding: 'json', metadata: new Map(),
  })
  const videoSchema = await writer.registerSchema({
    name: 'foxglove.CompressedVideo', encoding: 'ros2msg', data: new TextEncoder().encode(COMPRESSED_VIDEO_SCHEMA),
  })
  const camera = await writer.registerChannel({
    schemaId: videoSchema, topic: 'video/camera/stream', messageEncoding: 'cdr', metadata: new Map(),
  })
  const payload = encodeCompressedVideo('h264', SAMPLE_H264_KEYFRAME)
  for (let index = 0; index < 40; index += 1) {
    const logTime = 1_000_000_000n + BigInt(index) * 100_000_000n
    await writer.addMessage({
      channelId: telemetry,
      sequence: index,
      logTime,
      publishTime: logTime,
      data: new TextEncoder().encode(JSON.stringify({ depth_m: index })),
    })
    if (index >= 20) {
      await writer.addMessage({
        channelId: camera, sequence: index - 20, logTime, publishTime: logTime, data: payload,
      })
    }
  }
  await writer.end()
  return buffer.get()
}

/** A finished recording served as if it were still being written, `writtenChunks` chunks at a time. */
export interface LiveRecording {
  /** Chunks of the finished recording, in file order. */
  chunks: McapChunkIndex[]
  /** Chunks on disk so far; raise it to write more. */
  writtenChunks: number
  /** The bytes on disk so far. */
  bytes: () => Uint8Array
  source: ByteSource & { bytesRead: number }
  /** Pages through the written chunks the way the recorder's index service does. */
  indexSource: RecordingIndexSource
}

export async function asLiveRecording(finished: Uint8Array, writtenChunks: number): Promise<LiveRecording> {
  const reader = await McapIndexedReader.open(new MemoryByteSource(finished))
  const { chunkIndexes } = reader.summary
  const counts = await Promise.all(chunkIndexes.map((chunk, index) => Promise.all(
    chunk.channelIds.map(async (channelId) => ({
      channel_id: channelId,
      count: (await reader.readChunkMessageEntries(index, channelId))?.length ?? 0,
    })),
  )))
  const live: LiveRecording = {
    chunks: chunkIndexes,
    writtenChunks,
    bytes: () => {
      const last = chunkIndexes[live.writtenChunks - 1]
      return finished.subarray(0, last.offset + last.length + last.messageIndexLength)
    },
    source: {
      bytesRead: 0,
      size: async () => live.bytes().length,
      read: async (offset, length) => {
        const slice = live.bytes().subarray(offset, offset + length)
        live.source.bytesRead += slice.length
        return slice
      },
    },
    indexSource: {
      page: async (fromOffset) => {
        const size = live.bytes().length
        const written = chunkIndexes
          .slice(0, live.writtenChunks)
          .map((chunk, index) => ({ chunk, index }))
          .filter(({ chunk }) => chunk.offset >= fromOffset)
        return {
          size,
          offset: size,
          closed: false,
          chunks: written.map(({ chunk }) => ({
            start_time: Number(chunk.startTime),
            end_time: Number(chunk.endTime),
            offset: chunk.offset,
            length: chunk.length,
            compression: chunk.compression,
            compressed_size: chunk.compressedSize,
            uncompressed_size: chunk.uncompressedSize,
            channel_ids: chunk.channelIds,
            message_index_length: chunk.messageIndexLength,
          })),
          message_counts: written.flatMap(({ index }) => counts[index]),
          records: new Uint8Array(),
        }
      },
    },
  }
  return live
}

/** Simple JSON channel for CSV export tests (avoids ROS2 CDR layout in the video helper). */
export async function buildJsonTelemetryMcap(messageCount = 3, chunkSize = 512): Promise<Uint8Array> {
  const buffer = new TempBuffer()
  const writer = new McapWriter({ writable: buffer, chunkSize })
  await writer.start({ profile: '', library: 'blueos-test' })
  const schemaId = await writer.registerSchema({
    name: 'blueos_msgs/msg/TelemetrySample',
    encoding: 'jsonschema',
    data: new TextEncoder().encode(JSON.stringify({
      type: 'object',
      properties: { depth_m: { type: 'number' }, label: { type: 'string' } },
    })),
  })
  const channelId = await writer.registerChannel({
    schemaId,
    topic: '/telemetry/depth',
    messageEncoding: 'json',
    metadata: new Map(),
  })
  const baseTime = 2_000_000_000n
  for (let index = 0; index < messageCount; index += 1) {
    const logTime = baseTime + BigInt(index) * 100_000_000n
    const payload = new TextEncoder().encode(JSON.stringify({ depth_m: index * 1.5, label: `sample-${index}` }))
    await writer.addMessage({
      channelId,
      sequence: index,
      logTime,
      publishTime: logTime,
      data: payload,
    })
  }
  await writer.end()
  return buffer.get()
}

/**
 * Two JSON topics interleaved: `/depth` at 0, 200 and 400 ms with `depth_m`, and `/attitude` at 100 and 300 ms, whose
 * second message adds `pitch` to `yaw`.
 */
export async function buildTwoJsonTopicsMcap(): Promise<Uint8Array> {
  const buffer = new TempBuffer()
  const writer = new McapWriter({ writable: buffer, chunkSize: 512 })
  await writer.start({ profile: '', library: 'blueos-test' })
  const schemaId = await writer.registerSchema({
    name: 'blueos_msgs/msg/TelemetrySample',
    encoding: 'jsonschema',
    data: new TextEncoder().encode('{"type":"object"}'),
  })
  const depth = await writer.registerChannel({
    schemaId, topic: '/depth', messageEncoding: 'json', metadata: new Map(),
  })
  const attitude = await writer.registerChannel({
    schemaId, topic: '/attitude', messageEncoding: 'json', metadata: new Map(),
  })
  const messages: [number, number, number, object][] = [
    [depth, 0, 0, { depth_m: 1 }],
    [attitude, 0, 100, { yaw: 90 }],
    [depth, 1, 200, { depth_m: 2 }],
    [attitude, 1, 300, { yaw: 91, pitch: 5 }],
    [depth, 2, 400, { depth_m: 3 }],
  ]
  for (const [channelId, sequence, milliseconds, value] of messages) {
    const logTime = 2_000_000_000n + BigInt(milliseconds) * 1_000_000n
    await writer.addMessage({
      channelId, sequence, logTime, publishTime: logTime, data: new TextEncoder().encode(JSON.stringify(value)),
    })
  }
  await writer.end()
  return buffer.get()
}

/**
 * One CDR topic, `/stamps`, whose messages carry a list of samples, each with a `uint64[]` holding a value beyond
 * 2^53.
 */
export async function buildUint64ArrayMcap(): Promise<Uint8Array> {
  const schema = [
    'blueos_msgs/Sample[] samples',
    '================================================================================',
    'MSG: blueos_msgs/Sample',
    'uint64[] stamps',
  ].join('\n')
  const buffer = new TempBuffer()
  const writer = new McapWriter({ writable: buffer, chunkSize: 512 })
  await writer.start({ profile: '', library: 'blueos-test' })
  const schemaId = await writer.registerSchema({
    name: 'blueos_msgs/msg/Stamps', encoding: 'ros2msg', data: new TextEncoder().encode(schema),
  })
  const channelId = await writer.registerChannel({
    schemaId, topic: '/stamps', messageEncoding: 'cdr', metadata: new Map(),
  })
  for (let index = 0; index < 3; index += 1) {
    const logTime = 3_000_000_000n + BigInt(index) * 100_000_000n
    await writer.addMessage({
      channelId,
      sequence: index,
      logTime,
      publishTime: logTime,
      data: encodeCdrWithSchema('blueos_msgs/msg/Stamps', schema, {
        samples: [{ stamps: [BigInt(index), 18446744073709551615n] }],
      }),
    })
  }
  await writer.end()
  return buffer.get()
}

function encodeCompressedVideo(format: string, data: Uint8Array): Uint8Array {
  return encodeCdrWithSchema('foxglove.CompressedVideo', COMPRESSED_VIDEO_SCHEMA, {
    sec: 0,
    nanosec: 0,
    format,
    data,
  })
}
