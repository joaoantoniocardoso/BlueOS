/** Paged chunk index walk; mirrors `blueos_recorder_msgs/msg/RecordingIndex` with JS number fields. */

export interface ChunkIndexEntry {
  start_time: number
  end_time: number
  offset: number
  length: number
  compression: string
  compressed_size: number
  uncompressed_size: number
  channel_ids: number[]
  message_index_length: number
}

export interface ChannelMessageCount {
  channel_id: number
  count: number
}

export interface RecordingIndexPage {
  size: number
  offset: number
  closed: boolean
  chunks: ChunkIndexEntry[]
  message_counts: ChannelMessageCount[]
  records: Uint8Array
}

export interface RecordingIndexSource {
  page(fromOffset: number, limit: number, signal?: AbortSignal): Promise<RecordingIndexPage>
}
