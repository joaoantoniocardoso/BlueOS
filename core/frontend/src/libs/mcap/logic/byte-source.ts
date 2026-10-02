/**
 * Random access like @mcap/core `IReadable`, with AbortSignal and a bytesRead counter. Number
 * offsets match HTTP Range; `@mcap/core` CachedReadable keys exact (offset, size) pairs and never
 * evicts, so it cannot stand in for the decompressed-chunk cache.
 */
export interface ByteSource {
  size(signal?: AbortSignal): Promise<number>
  read(offset: number, length: number, signal?: AbortSignal): Promise<Uint8Array>
  /** Bytes actually transferred so far, used to show the real cost of playback to the user. */
  readonly bytesRead: number
}
