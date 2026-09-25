import { describe, expect, it } from 'vitest'

import type { RecordingIndexPage } from '@/libs/mcap/logic/recording-index'

describe('RecordingIndexPage', () => {
  it('accepts empty prefix pages with number fields', () => {
    const page: RecordingIndexPage = {
      size: 1024,
      offset: 512,
      closed: false,
      chunks: [{
        start_time: 0,
        end_time: 1_000_000_000,
        offset: 200,
        length: 80,
        compression: '',
        compressed_size: 80,
        uncompressed_size: 80,
        channel_ids: [1],
        message_index_length: 0,
      }],
      message_counts: [{ channel_id: 1, count: 3 }],
      records: new Uint8Array(),
    }
    expect(page.chunks[0].channel_ids).toEqual([1])
    expect(page.message_counts[0].count).toBe(3)
  })
})
