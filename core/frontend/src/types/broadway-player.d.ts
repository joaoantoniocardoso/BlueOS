declare module 'broadway-player' {
  export interface BroadwayDecoder {
    decode(data: Uint8Array): void
    onPictureDecoded?: (buffer: Uint8Array, width: number, height: number, infos: unknown) => void
  }

  export class Decoder implements BroadwayDecoder {
    constructor()

    decode(data: Uint8Array): void

    onPictureDecoded?: (buffer: Uint8Array, width: number, height: number, infos: unknown) => void
  }
}
