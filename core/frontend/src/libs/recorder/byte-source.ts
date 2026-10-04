import { query } from '@/libs/blueos-api/command'
import { bytes } from '@/libs/blueos-api/services/recorder'
import type { Transport } from '@/libs/blueos-api/transport'
import zenohTransport from '@/libs/blueos-api/zenoh-transport'
import { HttpByteSource } from '@/libs/mcap/adapters/http-byte-source'
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

  constructor(private readonly transport: Transport, public readonly path: string) {}

  async size(signal?: AbortSignal): Promise<number> {
    if (this.total === null) {
      await this.readPiece(0, 0, signal)
    }
    return this.total ?? 0
  }

  async read(offset: number, length: number, signal?: AbortSignal): Promise<Uint8Array> {
    if (length <= 0) {
      return new Uint8Array()
    }
    const pieces = await Promise.all(Array.from(
      { length: Math.ceil(length / RECORDING_BYTES_MAX_LENGTH) },
      (_, piece) => {
        const start = piece * RECORDING_BYTES_MAX_LENGTH
        return this.readPiece(offset + start, Math.min(RECORDING_BYTES_MAX_LENGTH, length - start), signal)
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

  private readPiece(offset: number, length: number, signal?: AbortSignal): Promise<Uint8Array> {
    const piece = this.queue.then(() => this.queryPiece(offset, length, signal))
    this.queue = piece.catch(() => undefined)
    return piece
  }

  private async queryPiece(offset: number, length: number, signal?: AbortSignal): Promise<Uint8Array> {
    throwIfAborted(signal)
    const response = await whileWaiting(query(this.transport, bytes, { path: this.path, offset, length }), signal)
    if (this.total === null) {
      this.total = response.size
    }
    // CDR decodes `uint8[]` as a Uint8Array, although the generated type says `number[]`.
    const data = response.data instanceof Uint8Array ? response.data : Uint8Array.from(response.data)
    this.bytesRead += data.byteLength
    return data
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
