import { describe, expect, it } from 'vitest'

import { decodeCdr, encodeCdr } from '@/libs/blueos-api/cdr'
import { COMMAND_ACK_SCHEMA, LOG_SCHEMA } from '@/libs/blueos-api/types'

// Produced by blueos-idl Rust codec for CommandAck { accepted: true, job_id: 42, reason: "queued" }.
const RUST_COMMAND_ACK_ROUND_TRIP = Uint8Array.from([
  0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x2a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00,
  0x71, 0x75, 0x65, 0x75, 0x65, 0x64, 0x00,
])

// Old writer (bool + job_id only) from blueos-idl tests/messages.rs command_ack_decode_old_writer_new_reader.
const RUST_COMMAND_ACK_OLD_WRITER = Uint8Array.from([
  0x00, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
])

describe('blueos-api CDR codec', () => {
  it('round-trips CommandAck', () => {
    const message = {
      accepted: true,
      job_id: 42,
      reason: 'queued',
    }
    const encoded = encodeCdr(COMMAND_ACK_SCHEMA, message)
    expect(encoded).toEqual(RUST_COMMAND_ACK_ROUND_TRIP)
    expect(decodeCdr(COMMAND_ACK_SCHEMA, encoded)).toEqual(message)
  })

  it('decodes Rust CommandAck bytes', () => {
    expect(decodeCdr(COMMAND_ACK_SCHEMA, RUST_COMMAND_ACK_ROUND_TRIP)).toEqual({
      accepted: true,
      job_id: 42,
      reason: 'queued',
    })
  })

  it('defaults missing trailing CommandAck fields (D-06 old writer)', () => {
    expect(decodeCdr(COMMAND_ACK_SCHEMA, RUST_COMMAND_ACK_OLD_WRITER)).toEqual({
      accepted: true,
      job_id: 7,
      reason: '',
    })
  })

  it('throws on corrupt CDR payload', () => {
    expect(() => decodeCdr(COMMAND_ACK_SCHEMA, new Uint8Array(0))).toThrow()
  })

  it('decodes numeric sequences as arrays (RecordingIndex)', () => {
    const message = {
      size: 4096,
      offset: 0,
      closed: true,
      chunks: [{
        start_time: 1,
        end_time: 2,
        offset: 8,
        length: 100,
        compression: 'zstd',
        compressed_size: 80,
        uncompressed_size: 120,
        channel_ids: [1, 2],
        message_index_length: 30,
      }],
      message_counts: [],
      records: [0x89, 0x4d],
    }
    const schemaName = 'blueos_recorder_msgs/msg/RecordingIndex'
    const decoded = decodeCdr(schemaName, encodeCdr(schemaName, message))
    expect(decoded).toEqual(message)
    expect(Array.isArray(decoded.chunks[0].channel_ids)).toBe(true)
  })

  it('round-trips foxglove Log', () => {
    const message = {
      timestamp: { sec: 1, nanosec: 2 },
      level: 2,
      message: 'hello',
      name: 'example',
      file: 'main.rs',
      line: 10,
    }
    const encoded = encodeCdr(LOG_SCHEMA, message)
    expect(decodeCdr(LOG_SCHEMA, encoded)).toEqual(message)
  })
})
