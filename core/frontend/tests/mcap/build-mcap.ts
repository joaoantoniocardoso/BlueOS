import { McapWriter, TempBuffer } from '@mcap/core'

import { encodeCdrWithSchema } from '@/libs/blueos-api/cdr'

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
