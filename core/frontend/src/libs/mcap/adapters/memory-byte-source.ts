import type { ByteSource } from './byte-source'

/** In-memory byte source for tests and harness-style probes. */
export default class MemoryByteSource implements ByteSource {
  bytesRead = 0

  constructor(private readonly data: Uint8Array) {}

  async size(): Promise<number> {
    return this.data.byteLength
  }

  async read(offset: number, length: number): Promise<Uint8Array> {
    const end = Math.min(offset + length, this.data.byteLength)
    const slice = this.data.subarray(offset, end)
    this.bytesRead += slice.byteLength
    return slice
  }
}
