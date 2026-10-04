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

/**
 * Queries one read keeps in flight. The recorder answers them one at a time, so more would only queue on the
 * vehicle; these hide the round trip through nginx, the router and the recorder between two replies.
 */
export const RECORDING_BYTES_IN_FLIGHT = 4

/** Reads a recording with the recorder `bytes` Query over the backbone instead of HTTP ranges from nginx. */
export class ZenohByteSource implements ByteSource {
  bytesRead = 0

  private total: number | null = null

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
    const pieceCount = Math.ceil(length / RECORDING_BYTES_MAX_LENGTH)
    const pieces: Uint8Array[] = []
    let nextPiece = 0
    const readPieces = async (): Promise<void> => {
      while (nextPiece < pieceCount) {
        const start = nextPiece * RECORDING_BYTES_MAX_LENGTH
        nextPiece += 1
        // eslint-disable-next-line no-await-in-loop
        pieces[start / RECORDING_BYTES_MAX_LENGTH] = await this.readPiece(
          offset + start,
          Math.min(RECORDING_BYTES_MAX_LENGTH, length - start),
          signal,
        )
      }
    }
    await Promise.all(Array.from({ length: Math.min(RECORDING_BYTES_IN_FLIGHT, pieceCount) }, readPieces))
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

  private async readPiece(offset: number, length: number, signal?: AbortSignal): Promise<Uint8Array> {
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
