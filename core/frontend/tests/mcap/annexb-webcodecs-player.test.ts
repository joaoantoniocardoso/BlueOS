/* eslint-disable class-methods-use-this, max-classes-per-file */
import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import { AnnexBWebCodecsPlayer, isWebCodecsSupported } from '@/libs/mcap/adapters/annexb-webcodecs-player'

import { SAMPLE_H264_DELTA, SAMPLE_H264_KEYFRAME } from './build-mcap'

interface FakeChunk {
  type: 'key' | 'delta'
  timestamp: number
  data: Uint8Array
}

class FakeEncodedVideoChunk {
  type: 'key' | 'delta'

  timestamp: number

  data: Uint8Array

  constructor(init: FakeChunk) {
    this.type = init.type
    this.timestamp = init.timestamp
    this.data = init.data
  }
}

interface FakeFrame {
  displayWidth: number
  displayHeight: number
  timestamp: number
  close: ReturnType<typeof vi.fn>
}

function frameAt(timestamp: number): FakeFrame {
  return {
    displayWidth: 640, displayHeight: 360, timestamp, close: vi.fn(),
  }
}

interface DecoderInit {
  output: (frame: FakeFrame) => void
  error: (error: DOMException) => void
}

class FakeVideoDecoder {
  static instances: FakeVideoDecoder[] = []

  state: 'unconfigured' | 'configured' | 'closed' = 'unconfigured'

  decodeQueueSize = 0

  config: { codec: string, optimizeForLatency: boolean, description?: Uint8Array } | null = null

  decoded: FakeChunk[] = []

  constructor(public init: DecoderInit) {
    FakeVideoDecoder.instances.push(this)
  }

  configure(config: { codec: string, optimizeForLatency: boolean, description?: Uint8Array }): void {
    this.config = config
    this.state = 'configured'
  }

  decode(chunk: FakeChunk): void {
    this.decoded.push(chunk)
  }

  close(): void {
    this.state = 'closed'
  }
}

function lastDecoder(): FakeVideoDecoder {
  return FakeVideoDecoder.instances[FakeVideoDecoder.instances.length - 1]
}

function fakeCanvas(): { canvas: HTMLCanvasElement, drawImage: ReturnType<typeof vi.fn> } {
  const drawImage = vi.fn()
  const canvas = { width: 0, height: 0, getContext: () => ({ drawImage }) }
  return { canvas: canvas as unknown as HTMLCanvasElement, drawImage }
}

describe('AnnexBWebCodecsPlayer', () => {
  beforeEach(() => {
    FakeVideoDecoder.instances = []
    vi.stubGlobal('VideoDecoder', FakeVideoDecoder)
    vi.stubGlobal('EncodedVideoChunk', FakeEncodedVideoChunk)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('detects whether the browser has WebCodecs', () => {
    expect(isWebCodecsSupported()).toBe(true)

    vi.stubGlobal('VideoDecoder', undefined)

    expect(isWebCodecsSupported()).toBe(false)
  })

  it('opens a low-latency decoder on the first keyframe and ignores the frames before it', () => {
    const { canvas } = fakeCanvas()
    const player = new AnnexBWebCodecsPlayer(canvas)

    player.push(SAMPLE_H264_DELTA, 'h264')
    expect(FakeVideoDecoder.instances).toHaveLength(0)

    player.push(SAMPLE_H264_KEYFRAME, 'h264')
    player.push(SAMPLE_H264_DELTA, 'h264')

    const decoder = lastDecoder()
    expect(decoder.config?.codec).toMatch(/^avc1\./)
    expect(decoder.config?.optimizeForLatency).toBe(true)
    expect(decoder.config?.description).toBeUndefined()
    expect(decoder.decoded.map((chunk) => chunk.type)).toEqual(['key', 'delta'])
  })

  it('drops delta frames while the decoder is behind and resumes at the next keyframe', () => {
    const { canvas } = fakeCanvas()
    const player = new AnnexBWebCodecsPlayer(canvas)
    player.push(SAMPLE_H264_KEYFRAME, 'h264')
    const decoder = lastDecoder()

    decoder.decodeQueueSize = 2
    player.push(SAMPLE_H264_DELTA, 'h264')
    decoder.decodeQueueSize = 0
    player.push(SAMPLE_H264_DELTA, 'h264')
    player.push(SAMPLE_H264_KEYFRAME, 'h264')

    expect(decoder.decoded.map((chunk) => chunk.type)).toEqual(['key', 'key'])
  })

  it('draws decoded frames at their size, closes them, and reports ready once with the lag', () => {
    const { canvas, drawImage } = fakeCanvas()
    const onReady = vi.fn()
    const onStats = vi.fn()
    const player = new AnnexBWebCodecsPlayer(canvas, { onReady, onStats })
    player.push(SAMPLE_H264_KEYFRAME, 'h264')
    const decoder = lastDecoder()

    const first = frameAt(performance.now() * 1000 - 50_000)
    const second = frameAt(performance.now() * 1000)
    decoder.init.output(first)
    decoder.init.output(second)

    expect(canvas.width).toBe(640)
    expect(canvas.height).toBe(360)
    expect(drawImage).toHaveBeenCalledTimes(2)
    expect(first.close).toHaveBeenCalledTimes(1)
    expect(second.close).toHaveBeenCalledTimes(1)
    expect(onReady).toHaveBeenCalledTimes(1)
    expect(onStats.mock.calls[0][0].lagSeconds).toBeGreaterThanOrEqual(0.05)
    expect(onStats.mock.calls[0][0].width).toBeGreaterThan(0)
  })

  it('retries a decoder error once with the stream description and length-prefixed frames', () => {
    const { canvas } = fakeCanvas()
    const onError = vi.fn()
    const player = new AnnexBWebCodecsPlayer(canvas, { onError })
    player.push(SAMPLE_H264_KEYFRAME, 'h264')
    const rejecting = lastDecoder()

    rejecting.init.error(new DOMException('unsupported'))

    const retrying = lastDecoder()
    expect(retrying).not.toBe(rejecting)
    expect(rejecting.state).toBe('closed')
    expect(retrying.config?.description).toBeInstanceOf(Uint8Array)
    expect(retrying.decoded).toHaveLength(1)
    expect(Array.from(retrying.decoded[0].data.subarray(0, 4))).toEqual([0, 0, 0, 7])
    expect(onError).not.toHaveBeenCalled()

    retrying.init.error(new DOMException('still unsupported'))

    expect(onError).toHaveBeenCalledTimes(1)
  })

  it('closes the decoder and discards late frames once destroyed', () => {
    const { canvas, drawImage } = fakeCanvas()
    const player = new AnnexBWebCodecsPlayer(canvas)
    player.push(SAMPLE_H264_KEYFRAME, 'h264')
    const decoder = lastDecoder()

    player.destroy()
    const late = frameAt(0)
    decoder.init.output(late)
    player.push(SAMPLE_H264_KEYFRAME, 'h264')

    expect(decoder.state).toBe('closed')
    expect(late.close).toHaveBeenCalledTimes(1)
    expect(drawImage).not.toHaveBeenCalled()
    expect(FakeVideoDecoder.instances).toHaveLength(1)
  })
})
