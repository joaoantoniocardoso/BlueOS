<template>
  <div />
</template>

<script lang="ts">
import Vue from 'vue'

interface VideoDecoderConfig {
  codec: string
  optimizeForLatency: boolean
  description?: ArrayBuffer
}

interface VideoDecoderInit {
  output: (frame: VideoFrame) => void
  error: (error: Error) => void
}

declare global {
  interface Window {
    VideoDecoder: {
      new (init: VideoDecoderInit): {
        configure: (config: VideoDecoderConfig) => Promise<void>
        decode: (chunk: EncodedVideoChunk) => void
        close: () => void
        decodeQueueSize: number
        ondequeue: ((event: Event) => void) | null
        state: 'unconfigured' | 'configured' | 'closed'
        flush: () => Promise<void>
        reset: () => void
        addEventListener: (type: string, listener: EventListenerOrEventListenerObject) => void
        removeEventListener: (type: string, listener: EventListenerOrEventListenerObject) => void
        dispatchEvent: (event: Event) => boolean
      }
    }

    EncodedVideoChunk: {
      new (init: {
        type: 'key' | 'delta'
        timestamp: number
        duration?: number
        data: Uint8Array
      }): EncodedVideoChunk
    }
  }

  interface EncodedVideoChunk {
    readonly type: 'key' | 'delta'
    readonly timestamp: number
    readonly duration: number | null
    readonly byteLength: number
    readonly data: Uint8Array
  }
}

export default Vue.extend({
  name: 'VideoDecoder',
  props: {
    videoData: {
      type: Uint8Array,
      required: true,
    },
    canvas: {
      type: [Object, HTMLCanvasElement],
      required: false,
      default: null,
    },
  },
  data() {
    return {
      decoder: null,
      isDecoderReady: false,
      hasReceivedKeyframe: false, // <-- Add this flag
    }
  },
  watch: {
    videoData: {
      handler(newData: Uint8Array) {
        if (!newData || !this.decoder) {
          return
        }
        this.processVideoChunk(newData)
      },
      immediate: true,
    },
  },
  async mounted() {
    await this.setupVideoDecoder()
    if (this.videoData) {
      this.processVideoChunk(this.videoData)
    }
  },
  beforeDestroy() {
    this.cleanupVideoDecoder()
  },
  methods: {
  // Convert base64 to Uint8Array (you already have this, so no change)
    base64ToUint8Array(b64: string): Uint8Array {
      const binary = atob(b64)
      const array = new Uint8Array(binary.length)
      for (let i = 0; i < binary.length; i++) {
        array[i] = binary.charCodeAt(i)
      }
      return array
    },

    // Find NAL start codes in Annex B stream (you already have this)
    findStartCodeIndex(data: Uint8Array, start: number): number {
      for (let i = start; i < data.length - 3; i++) {
        if (data[i] === 0 && data[i + 1] === 0) {
          if (data[i + 2] === 1) return i
          if (data[i + 2] === 0 && data[i + 3] === 1) return i
        }
      }
      return -1
    },

    // Extract SPS and PPS NALUs (you already have this)
    extractSpsPps(data: Uint8Array): { sps: Uint8Array | null; pps: Uint8Array | null } {
      let sps: Uint8Array | null = null
      let pps: Uint8Array | null = null
      let i = 0

      while (i < data.length) {
        const start = this.findStartCodeIndex(data, i)
        if (start === -1) break

        let nextStart = this.findStartCodeIndex(data, start + 4)
        if (nextStart === -1) nextStart = data.length

        const nalHeaderOffset = data[start + 2] === 1 ? 3 : 4
        const nalType = data[start + nalHeaderOffset] & 0x1f

        const nal = data.subarray(start + nalHeaderOffset, nextStart)

        if (nalType === 7 && !sps) sps = nal
        if (nalType === 8 && !pps) pps = nal

        if (sps && pps) break
        i = nextStart
      }

      return { sps, pps }
    },

    // Build avcC box from SPS and PPS, required by VideoDecoder.configure()
    createAvcCFromSpsPps(sps: Uint8Array, pps: Uint8Array): Uint8Array {
      const profile = sps[1]
      const compatibility = sps[2]
      const level = sps[3]
      const lengthSizeMinusOne = 3 // 4 bytes length size

      const spsCount = 1
      const ppsCount = 1

      const size = 7 + 2 + sps.length + 2 + pps.length
      const avcC = new Uint8Array(size)
      let offset = 0

      avcC[offset++] = 1 // configurationVersion
      avcC[offset++] = profile // AVCProfileIndication
      avcC[offset++] = compatibility // profile_compatibility
      avcC[offset++] = level // AVCLevelIndication
      avcC[offset++] = 0xfc | lengthSizeMinusOne // lengthSizeMinusOne + reserved bits

      avcC[offset++] = 0xe0 | spsCount // numOfSequenceParameterSets + reserved bits
      avcC[offset++] = sps.length >>> 8 & 0xff // SPS length high byte
      avcC[offset++] = sps.length & 0xff // SPS length low byte
      avcC.set(sps, offset)
      offset += sps.length

      avcC[offset++] = ppsCount // numOfPictureParameterSets
      avcC[offset++] = pps.length >>> 8 & 0xff // PPS length high byte
      avcC[offset++] = pps.length & 0xff // PPS length low byte
      avcC.set(pps, offset)
      offset += pps.length

      return avcC
    },

    async setupVideoDecoder() {
      try {
        if (!this.canvas) {
          console.error('Canvas element not found')
          return
        }

        // Extract SPS and PPS from initial video data
        const { sps, pps } = this.extractSpsPps(this.videoData)

        if (!sps || !pps) {
          throw new Error('SPS or PPS not found in video stream')
        }

        console.log(`SPS length: ${sps.length}, PPS length: ${pps.length}`)

        const avcC = this.createAvcCFromSpsPps(sps, pps)
        console.log('avcC length:', avcC.length)

        this.decoder = new window.VideoDecoder({
          output: (frame) => {
            this.renderFrame(frame)
            frame.close()
          },
          error: (error) => {
            console.error('VideoDecoder error:', error)
            // If error occurs, close decoder to reset state
            if (this.decoder && this.decoder.state !== 'closed') {
              this.decoder.close()
            }
            this.isDecoderReady = false
          },
        })

        await this.decoder.configure({
          codec: 'avc1.42E01E',
          optimizeForLatency: true,
          description: avcC.buffer,
        })

        this.isDecoderReady = true
      } catch (error) {
        console.error('Error setting up VideoDecoder:', error)
        this.decoder = null
        this.isDecoderReady = false
      }
    },

    processVideoChunk(chunkData: Uint8Array) {
      if (!this.decoder || !this.isDecoderReady) {
        console.warn('Decoder not ready yet')
        return
      }

      if (this.decoder.state === 'closed') {
        console.warn('Decoder closed, resetting...')
        this.setupVideoDecoder()
        return
      }

      // Helper to check for IDR (keyframe)
      function containsIdrFrame(data: Uint8Array): boolean {
        for (let i = 0; i < data.length - 4; i++) {
          if (
            data[i] === 0
          && data[i + 1] === 0
          && (data[i + 2] === 1 || data[i + 2] === 0 && data[i + 3] === 1)
          ) {
            const offset = data[i + 2] === 1 ? 3 : 4
            const nalType = data[i + offset] & 0x1f
            if (nalType === 5) return true
          }
        }
        return false
      }

      try {
      // Decide if this chunk has a keyframe
        const isKeyframe = containsIdrFrame(chunkData)

        if (isKeyframe) {
          this.hasReceivedKeyframe = true
        }

        if (!this.hasReceivedKeyframe) {
          console.warn('Waiting for keyframe (IDR) before decoding...')
          return
        }

        // Convert and decode chunk as before
        const nalus = this.splitAnnexBNalus(chunkData)
        if (nalus.length === 0) {
          console.warn('No NALUs found in chunk')
          return
        }

        const avccData = this.convertAnnexBToAvcc(nalus)

        const chunk = new EncodedVideoChunk({
          type: isKeyframe ? 'key' : 'delta',
          timestamp: performance.now(),
          data: avccData,
        })

        this.decoder.decode(chunk)
      } catch (error) {
        console.error('Error decoding video chunk:', error)
        if (this.decoder && this.decoder.state !== 'closed') this.decoder.close()
        this.isDecoderReady = false
        this.hasReceivedKeyframe = false
      }
    },

    // Utility to split Annex B stream into NALUs (no start codes)
    splitAnnexBNalus(data: Uint8Array): Uint8Array[] {
      const nalus: Uint8Array[] = []
      let i = 0
      while (i < data.length) {
        // Find start code
        let start = -1
        for (let j = i; j < data.length - 3; j++) {
          if (data[j] === 0 && data[j + 1] === 0 && (data[j + 2] === 1 || data[j + 2] === 0 && data[j + 3] === 1)) {
            start = j
            break
          }
        }
        if (start === -1) break

        // Find next start code
        let nextStart = -1
        for (let k = start + 3; k < data.length - 3; k++) {
          if (data[k] === 0 && data[k + 1] === 0 && (data[k + 2] === 1 || data[k + 2] === 0 && data[k + 3] === 1)) {
            nextStart = k
            break
          }
        }

        if (nextStart === -1) {
          nalus.push(data.subarray(start + (data[start + 2] === 1 ? 3 : 4)))
          break
        } else {
          nalus.push(data.subarray(start + (data[start + 2] === 1 ? 3 : 4), nextStart))
          i = nextStart
        }
      }
      return nalus
    },

    // Convert Annex B NALUs to AVCC format with 4-byte length prefixes
    convertAnnexBToAvcc(nalus: Uint8Array[]): Uint8Array {
      let totalLength = 0
      for (const nalu of nalus) {
        totalLength += 4 + nalu.length
      }
      const result = new Uint8Array(totalLength)
      let offset = 0
      for (const nalu of nalus) {
        const len = nalu.length
        result[offset++] = len >>> 24 & 0xff
        result[offset++] = len >>> 16 & 0xff
        result[offset++] = len >>> 8 & 0xff
        result[offset++] = len & 0xff
        result.set(nalu, offset)
        offset += len
      }
      return result
    },

    // Check if the chunk contains an IDR frame (NAL type 5)
    containsIdrFrame(data: Uint8Array): boolean {
      for (let i = 0; i < data.length - 4; i++) {
        if (
          data[i] === 0
          && data[i + 1] === 0
          && (data[i + 2] === 1 || data[i + 2] === 0 && data[i + 3] === 1)
        ) {
          const offset = data[i + 2] === 1 ? 3 : 4
          const nalType = data[i + offset] & 0x1f
          if (nalType === 5) return true
        }
      }
      return false
    },

    renderFrame(frame: VideoFrame) {
      if (!this.canvas) return

      const canvas = this.canvas as HTMLCanvasElement
      canvas.width = frame.displayWidth
      canvas.height = frame.displayHeight

      const ctx = canvas.getContext('2d')
      if (!ctx) return

      // Draw VideoFrame to canvas
      try {
        ctx.drawImage(frame, 0, 0, canvas.width, canvas.height)
      } catch (e) {
        console.error('Error drawing frame:', e)
      }
    },

    cleanupVideoDecoder() {
      if (this.decoder) {
        this.decoder.close()
      }
      this.decoder = null
      this.framesDecoded = 0
      this.isDecoderReady = false
      this.pendingChunks = []
    },
  },
})
</script>
