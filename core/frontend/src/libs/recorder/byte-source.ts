import { query } from '@/libs/blueos-api/command'
import { bytes, type bytesRequest, type bytesResponse } from '@/libs/blueos-api/services/recorder'
import type { Transport } from '@/libs/blueos-api/transport'
import zenohTransport from '@/libs/blueos-api/zenoh-transport'
import { HttpByteSource, TAIL_SIZE } from '@/libs/mcap/adapters/http-byte-source'
import { throwIfAborted, whileWaiting } from '@/libs/mcap/logic/abort'
import type { ByteSource } from '@/libs/mcap/logic/byte-source'
import zenoh from '@/libs/zenoh'

import { DEFAULT_RECORDING_HTTP_PREFIX } from './constants'
import { browserStorage, storedByteSource } from './preferences'

/** The most bytes one reply of the recorder `bytes` Query carries. */
export const RECORDING_BYTES_MAX_LENGTH = 1024 * 1024

/** Reads a recording with the recorder `bytes` Query over the backbone instead of HTTP ranges from nginx. */
export class ZenohByteSource implements ByteSource {
  bytesRead = 0

  private total: number | null = null

  /**
   * Queries go one at a time. The recorder answers them in turn anyway, and a reply that overtakes no request
   * waits up to 40 ms on the remote_api websocket: the plugin leaves Nagle's algorithm on, and nginx delays the
   * ACK that a next request would otherwise carry.
   */
  private queue: Promise<unknown> = Promise.resolve()

  private tail: { offset: number, data: Uint8Array } | null = null

  constructor(private readonly transport: Transport, public readonly path: string) {}

  /** Reads the end of the recording with its size, like the suffix range of `HttpByteSource`, for the footer. */
  async size(signal?: AbortSignal): Promise<number> {
    if (this.total === null) {
      const { size, data } = await this.request({ offset: 0, length: TAIL_SIZE, from_end: true }, signal)
      this.tail = { offset: size - data.byteLength, data }
    }
    return this.total ?? 0
  }

  async read(offset: number, length: number, signal?: AbortSignal): Promise<Uint8Array> {
    if (length <= 0) {
      return new Uint8Array()
    }
    const { tail } = this
    if (tail && offset >= tail.offset && offset + length <= tail.offset + tail.data.byteLength) {
      return tail.data.subarray(offset - tail.offset, offset - tail.offset + length)
    }
    const pieces = await Promise.all(Array.from(
      { length: Math.ceil(length / RECORDING_BYTES_MAX_LENGTH) },
      async (_, piece) => {
        const start = offset + piece * RECORDING_BYTES_MAX_LENGTH
        const pieceLength = Math.min(RECORDING_BYTES_MAX_LENGTH, offset + length - start)
        return (await this.request({ offset: start, length: pieceLength, from_end: false }, signal)).data
      },
    ))
    if (pieces.length === 1) {
      return pieces[0]
    }
    const data = new Uint8Array(pieces.reduce((total, piece) => total + piece.byteLength, 0))
    pieces.reduce((position, piece) => {
      data.set(piece, position)
      return position + piece.byteLength
    }, 0)
    return data
  }

  private request(range: Omit<bytesRequest, 'path'>, signal?: AbortSignal): Promise<bytesResponse> {
    const response = this.queue.then(async () => {
      throwIfAborted(signal)
      const answer = await whileWaiting(query(this.transport, bytes, { path: this.path, ...range }), signal)
      if (this.total === null) {
        this.total = answer.size
      }
      this.bytesRead += answer.data.byteLength
      return answer
    })
    this.queue = response.catch(() => undefined)
    return response
  }
}

/**
 * The source of the recording that nginx serves at `url`: HTTP ranges, or the recorder `bytes` Query when the
 * Records byte source is set to `zenoh`. Read on each call, so switching needs no rebuild.
 */
export async function recordingByteSource(url: string): Promise<ByteSource> {
  if (storedByteSource(browserStorage) !== 'zenoh') {
    return new HttpByteSource(url)
  }
  const path = url.slice(DEFAULT_RECORDING_HTTP_PREFIX.length + 1).split('/').map(decodeURIComponent).join('/')
  return new ZenohByteSource(zenohTransport(await zenoh.getSession()), path)
}
