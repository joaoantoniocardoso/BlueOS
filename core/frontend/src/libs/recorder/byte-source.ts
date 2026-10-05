import { query } from '@/libs/blueos-api/command'
import { bytes, type bytesRequest, type bytesResponse } from '@/libs/blueos-api/services/recorder'
import type { Transport } from '@/libs/blueos-api/transport'
import { throwIfAborted, whileWaiting } from '@/libs/mcap/logic/abort'
import type { ByteSource } from '@/libs/mcap/logic/byte-source'

/** Bytes read from the end of the recording to learn its size and read the MCAP footer at once. */
const TAIL_SIZE = 4096

/** The most bytes one reply of the recorder `bytes` Query carries. */
export const RECORDING_BYTES_MAX_LENGTH = 1024 * 1024

/**
 * Queries one read keeps in flight. The recorder answers them one at a time, so more would only queue on the
 * vehicle; these hide the round trip through nginx, the router and the recorder between two replies.
 */
export const RECORDING_BYTES_IN_FLIGHT = 4

/** Reads a recording with the recorder `bytes` Query over the backbone. */
export class ZenohByteSource implements ByteSource {
  bytesRead = 0

  private total: number | null = null

  private tail: { offset: number, data: Uint8Array } | null = null

  constructor(private readonly transport: Transport, public readonly path: string) {}

  /** Reads the end of the recording with its size, for the footer. */
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
    const pieceCount = Math.ceil(length / RECORDING_BYTES_MAX_LENGTH)
    const pieces: Uint8Array[] = []
    let nextPiece = 0
    const readPieces = async (): Promise<void> => {
      while (nextPiece < pieceCount) {
        const piece = nextPiece
        nextPiece += 1
        const start = offset + piece * RECORDING_BYTES_MAX_LENGTH
        const pieceLength = Math.min(RECORDING_BYTES_MAX_LENGTH, offset + length - start)
        // eslint-disable-next-line no-await-in-loop
        pieces[piece] = (await this.request({ offset: start, length: pieceLength, from_end: false }, signal)).data
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

  private async request(range: Omit<bytesRequest, 'path'>, signal?: AbortSignal): Promise<bytesResponse> {
    throwIfAborted(signal)
    const answer = await whileWaiting(query(this.transport, bytes, { path: this.path, ...range }), signal)
    if (this.total === null) {
      this.total = answer.size
    }
    this.bytesRead += answer.data.byteLength
    return answer
  }
}
