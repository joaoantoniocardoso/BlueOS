/* eslint-disable class-methods-use-this, max-classes-per-file */
import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import { AnnexBMsePlayer, type AnnexBMseStats } from '@/libs/mcap/adapters/annexb-player'

import { SAMPLE_H264_DELTA, SAMPLE_H264_KEYFRAME } from './build-mcap'

const FRAME_SECONDS = 1 / 30

/** Offset of the payload of the first ISO BMFF box of this type, or -1. */
function boxPayload(data: Uint8Array, type: string): number {
  const code = [...type].map((character) => character.charCodeAt(0))
  for (let index = 4; index + 4 <= data.length; index += 1) {
    if (code.every((byte, offset) => data[index + offset] === byte)) {
      return index + 4
    }
  }
  return -1
}

/** Buffers each appended fragment at its `tfdt` decode time, which is all the player reads back. */
class FakeSourceBuffer extends EventTarget {
  mode = 'segments'

  updating = false

  /** Start of every fragment appended, in append order, whatever was removed since. */
  appendedStarts: number[] = []

  private fragmentStarts: number[] = []

  private timescale = 1

  get buffered(): TimeRanges {
    const ranges = this.fragmentStarts.length === 0
      ? []
      : [[Math.min(...this.fragmentStarts), Math.max(...this.fragmentStarts) + FRAME_SECONDS]]
    return {
      length: ranges.length,
      start: (index: number) => ranges[index][0],
      end: (index: number) => ranges[index][1],
    }
  }

  appendBuffer(data: Uint8Array): void {
    const view = new DataView(data.buffer, data.byteOffset, data.byteLength)
    const mdhd = boxPayload(data, 'mdhd')
    if (mdhd >= 0) {
      this.timescale = view.getUint32(mdhd + 4 + (data[mdhd] === 1 ? 16 : 8))
    }
    const tfdt = boxPayload(data, 'tfdt')
    const decodeTime = tfdt < 0
      ? null
      : Number(data[tfdt] === 1 ? view.getBigUint64(tfdt + 4) : view.getUint32(tfdt + 4))
    if (decodeTime !== null) {
      this.appendedStarts.push(decodeTime / this.timescale)
    }
    setTimeout(() => {
      if (decodeTime !== null) {
        this.fragmentStarts.push(decodeTime / this.timescale)
      }
      this.dispatchEvent(new Event('updateend'))
    }, 0)
  }

  /** Every range removed, in order. */
  removed: Array<[number, number]> = []

  remove(start: number, end: number): void {
    this.removed.push([start, end])
    this.fragmentStarts = this.fragmentStarts.filter((fragmentStart) => fragmentStart < start || fragmentStart >= end)
    setTimeout(() => this.dispatchEvent(new Event('updateend')))
  }
}

let sourceBuffer: FakeSourceBuffer | null = null
let typeSupported = true

class FakeMediaSource extends EventTarget {
  static isTypeSupported(): boolean {
    return typeSupported
  }

  readyState = 'open'

  duration = Number.NaN

  addSourceBuffer(): FakeSourceBuffer {
    sourceBuffer = new FakeSourceBuffer()
    return sourceBuffer
  }
}

class FakeVideo extends EventTarget {
  paused = true

  muted = false

  playbackRate = 1

  src = ''

  private time = 0

  get currentTime(): number {
    return this.time
  }

  set currentTime(seconds: number) {
    this.time = seconds
  }

  get buffered(): TimeRanges {
    return sourceBuffer?.buffered ?? { length: 0, start: () => 0, end: () => 0 }
  }

  play(): Promise<void> {
    this.paused = false
    return Promise.resolve()
  }

  pause(): void {
    this.paused = true
  }

  removeAttribute(): void {
    this.src = ''
  }

  load(): void {
    this.time = 0
  }
}

function newPlayer(options: ConstructorParameters<typeof AnnexBMsePlayer>[1] = {}): {
  player: AnnexBMsePlayer
  video: FakeVideo
} {
  const video = new FakeVideo()
  return { player: new AnnexBMsePlayer(video as unknown as HTMLVideoElement, options), video }
}

function appendedCount(): number {
  return sourceBuffer?.appendedStarts.length ?? 0
}

/** Pushes frames at the pace of a camera, each one after the player appended the one before. */
async function pushPaced(player: AnnexBMsePlayer, frames: Array<[Uint8Array, number]>): Promise<void> {
  for (const [frame, timestampSeconds] of frames) {
    const appended = appendedCount()
    player.push(frame, 'h264', timestampSeconds)
    // eslint-disable-next-line no-await-in-loop
    await vi.waitFor(() => expect(appendedCount()).toBeGreaterThan(appended), { interval: 1 })
  }
}

describe('AnnexBMsePlayer', () => {
  beforeEach(() => {
    vi.stubGlobal('MediaSource', FakeMediaSource)
    vi.stubGlobal('window', globalThis)
    vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:live')
    vi.spyOn(URL, 'revokeObjectURL').mockReturnValue(undefined)
  })

  afterEach(() => {
    sourceBuffer = null
    typeSupported = true
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('refuses to start in a browser without Media Source Extensions', () => {
    vi.stubGlobal('window', {})

    expect(() => newPlayer()).toThrow('Media Source Extensions')
  })

  it('plays the stream muted from the object URL of its media source', () => {
    const { video } = newPlayer()

    expect(video.src).toBe('blob:live')
    expect(video.muted).toBe(true)
  })

  it('waits for a keyframe before it is ready, and reports the codec and size it found', async () => {
    const onReady = vi.fn()
    const onStats = vi.fn<[AnnexBMseStats], void>()
    const { player } = newPlayer({ onReady, onStats })

    player.push(SAMPLE_H264_DELTA, 'h264')
    await new Promise((resolve) => { setTimeout(resolve, 20) })
    expect(onReady).not.toHaveBeenCalled()
    expect(sourceBuffer).toBeNull()

    player.push(SAMPLE_H264_KEYFRAME, 'h264')
    await vi.waitFor(() => expect(onReady).toHaveBeenCalledTimes(1))
    const stats = onStats.mock.calls[onStats.mock.calls.length - 1][0]
    expect(stats.codec).toMatch(/^avc1\./)
    expect(stats.width).toBeGreaterThan(0)
    expect(stats.height).toBeGreaterThan(0)
    player.destroy()
  })

  it('shows each frame as its own fragment, timed from the first frame of the stream', async () => {
    const { player } = newPlayer()

    player.push(SAMPLE_H264_KEYFRAME, 'h264', 100)
    player.push(SAMPLE_H264_DELTA, 'h264', 100.04)
    player.push(SAMPLE_H264_DELTA, 'h264', 100.04)
    await vi.waitFor(() => expect(sourceBuffer?.appendedStarts).toHaveLength(3))

    const [first, second, third] = sourceBuffer?.appendedStarts ?? []
    expect(first).toBeCloseTo(0, 3)
    expect(second).toBeCloseTo(0.04, 3)
    expect(third).toBeGreaterThan(second)
    player.destroy()
  })

  it('drops a backlog down to the latest keyframe instead of playing through it', async () => {
    const { player } = newPlayer()

    player.push(SAMPLE_H264_KEYFRAME, 'h264', 1)
    for (let index = 0; index < 5; index += 1) {
      player.push(SAMPLE_H264_DELTA, 'h264', 1.04 + index * 0.04)
    }
    player.push(SAMPLE_H264_KEYFRAME, 'h264', 1.3)
    for (let index = 0; index < 5; index += 1) {
      player.push(SAMPLE_H264_DELTA, 'h264', 1.34 + index * 0.04)
    }

    await vi.waitFor(() => expect(sourceBuffer?.appendedStarts).toHaveLength(6))
    await new Promise((resolve) => { setTimeout(resolve, 50) })
    expect(sourceBuffer?.appendedStarts).toHaveLength(6)
    player.destroy()
  })

  it('jumps back to the live edge when the picture stalls', async () => {
    const { player, video } = newPlayer()
    for (let index = 0; index < 4; index += 1) {
      player.push(index === 0 ? SAMPLE_H264_KEYFRAME : SAMPLE_H264_DELTA, 'h264', 10 + index * FRAME_SECONDS)
    }
    await vi.waitFor(() => expect(video.buffered.length).toBe(1))
    video.currentTime = 0

    video.dispatchEvent(new Event('waiting'))

    expect(video.currentTime).toBeCloseTo(video.buffered.end(0), 2)
    expect(video.paused).toBe(false)
    player.destroy()
  })

  it('never evicts a keyframe that the frames being played still depend on', async () => {
    const { player, video } = newPlayer()
    await pushPaced(player, Array.from({ length: 101 }, (_, index) => [
      index === 0 ? SAMPLE_H264_KEYFRAME : SAMPLE_H264_DELTA, 100 + index * FRAME_SECONDS,
    ]))
    video.currentTime = 3.3

    await pushPaced(player, [[SAMPLE_H264_DELTA, 100 + 101 * FRAME_SECONDS]])
    await new Promise((resolve) => { setTimeout(resolve, 50) })

    expect(sourceBuffer?.removed).toEqual([])
    player.destroy()
  })

  it('evicts played media only up to the last keyframe that is out of the keep-behind window', async () => {
    const { player, video } = newPlayer()
    await pushPaced(player, Array.from({ length: 121 }, (_, index) => [
      index % 90 === 0 ? SAMPLE_H264_KEYFRAME : SAMPLE_H264_DELTA,
      100 + index * FRAME_SECONDS + (index >= 90 ? 0.0003 : 0),
    ]))
    video.currentTime = 5.2

    await pushPaced(player, [[SAMPLE_H264_DELTA, 100 + 121 * FRAME_SECONDS]])

    await vi.waitFor(() => expect(sourceBuffer?.removed).toHaveLength(1))
    expect(sourceBuffer?.removed[0][0]).toBe(0)
    expect(sourceBuffer?.removed[0][1]).toBeCloseTo(3, 1)
    // The muxer rounds a frame time down to its timescale, so a range that ends at the keyframe time reaches into it.
    expect(sourceBuffer?.removed[0][1]).toBeLessThan(sourceBuffer?.appendedStarts[90] ?? 0)
    player.destroy()
  })

  it('reports the codec as unsupported when the browser cannot play it', async () => {
    typeSupported = false
    const onError = vi.fn<[Error], void>()
    const { player } = newPlayer({ onError })

    player.push(SAMPLE_H264_KEYFRAME, 'h264')

    await vi.waitFor(() => expect(onError).toHaveBeenCalledTimes(1))
    expect(onError.mock.calls[0][0].message).toContain('cannot play H264 video')
    player.destroy()
  })

  it('releases the element and ignores frames once destroyed', async () => {
    const onReady = vi.fn()
    const { player, video } = newPlayer({ onReady })

    player.destroy()
    player.push(SAMPLE_H264_KEYFRAME, 'h264')
    await new Promise((resolve) => { setTimeout(resolve, 20) })

    expect(video.src).toBe('')
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:live')
    expect(onReady).not.toHaveBeenCalled()
  })
})
