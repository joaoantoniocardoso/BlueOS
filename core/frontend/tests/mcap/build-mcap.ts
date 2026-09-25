import { McapWriter, TempBuffer } from '@mcap/core'

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

function encodeCompressedVideo(format: string, data: Uint8Array): Uint8Array {
  const formatBytes = new TextEncoder().encode(format)
  const size = 4 + 4 + 4 + formatBytes.length + 1 + 4 + data.length
  const out = new Uint8Array(size)
  const view = new DataView(out.buffer)
  let offset = 0
  view.setInt32(offset, 0, true)
  offset += 4
  view.setUint32(offset, 0, true)
  offset += 4
  view.setUint32(offset, formatBytes.length, true)
  offset += 4
  out.set(formatBytes, offset)
  offset += formatBytes.length
  out[offset] = 0
  offset += 1
  view.setUint32(offset, data.length, true)
  offset += 4
  out.set(data, offset)
  return out
}
