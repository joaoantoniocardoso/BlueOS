/* eslint-disable class-methods-use-this, max-classes-per-file */
import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import { McapVideoPlayer } from '@/libs/mcap/adapters/player'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'
import { listVideoTracks } from '@/libs/mcap/logic/video-track'

import { buildIndexedVideoMcap, buildTwoTrackVideoMcap } from './build-mcap'
import MemoryByteSource from './memory-byte-source'

/** A browser decodes appended media asynchronously; the player must not count on it being instant. */
const APPEND_MS = 20

/** Frame spacing of `buildIndexedVideoMcap`; the fake buffers one frame past each fragment start. */
const FRAME_SECONDS = 0.033

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
    const { timescale } = this
    if (decodeTime !== null) {
      this.appendedStarts.push(decodeTime / timescale)
    }
    setTimeout(() => {
      if (decodeTime !== null) {
        this.fragmentStarts.push(decodeTime / timescale)
      }
      if (mdhd >= 0) {
        onMetadata?.()
      }
      this.dispatchEvent(new Event('updateend'))
    }, APPEND_MS)
  }

  remove(): void {
    this.fragmentStarts = []
    setTimeout(() => this.dispatchEvent(new Event('updateend')))
  }
}

let sourceBuffer: FakeSourceBuffer | null = null

/** What the `<video>` does once the source buffer holds the init segment, which is when it leaves `HAVE_NOTHING`. */
let onMetadata: (() => void) | null = null

class FakeMediaSource extends EventTarget {
  static isTypeSupported(): boolean {
    return true
  }

  readyState = 'open'

  duration = Number.NaN

  addSourceBuffer(): FakeSourceBuffer {
    sourceBuffer = new FakeSourceBuffer()
    return sourceBuffer
  }

  endOfStream(): void {
    this.readyState = 'ended'
  }
}

/** A media source the browser has not opened yet, so `start()` waits until the test opens it. */
class UnopenedMediaSource extends FakeMediaSource {
  static last: UnopenedMediaSource | null = null

  readyState = 'closed'

  constructor() {
    super()
    UnopenedMediaSource.last = this
  }

  open(): void {
    this.readyState = 'open'
    this.dispatchEvent(new Event('sourceopen'))
  }
}

class FakeVideo extends EventTarget {
  paused = true

  src = ''

  private time = 0

  get currentTime(): number {
    return this.time
  }

  set currentTime(seconds: number) {
    this.time = seconds
    this.dispatchEvent(new Event('seeking'))
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

/**
 * A `<video>` with the HTML default playback start position: a time set while it has no media is kept, reported as
 * its current time, and seeked to once its metadata loads. An element that a closed player left can hold one.
 */
class ReusedVideo extends FakeVideo {
  private defaultPlaybackStart = 0

  private metadataLoaded = false

  constructor() {
    super()
    onMetadata = () => {
      this.metadataLoaded = true
      const start = this.defaultPlaybackStart
      this.defaultPlaybackStart = 0
      if (start > 0) {
        super.currentTime = start
      }
    }
  }

  get currentTime(): number {
    return this.defaultPlaybackStart || super.currentTime
  }

  set currentTime(seconds: number) {
    if (this.metadataLoaded) {
      super.currentTime = seconds
    } else {
      this.defaultPlaybackStart = seconds
    }
  }
}

describe('McapVideoPlayer', () => {
  beforeEach(() => {
    vi.stubGlobal('MediaSource', FakeMediaSource)
    vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:recording')
    vi.spyOn(URL, 'revokeObjectURL').mockReturnValue(undefined)
  })

  afterEach(() => {
    sourceBuffer = null
    onMetadata = null
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('plays a followed recording from its latest frames', async () => {
    const reader = await McapIndexedReader.open(new MemoryByteSource(await buildIndexedVideoMcap(40)))
    const [track] = listVideoTracks(reader)
    const { startTime, endTime } = reader.summary
    const recording = {
      reader, tracks: [track], channels: [], durationSeconds: Number(endTime - startTime) / 1e9, startTime,
    }
    const video = new FakeVideo()
    const player = new McapVideoPlayer(video as unknown as HTMLVideoElement, recording, track, {
      startAtEnd: true,
      follow: true,
    })

    await player.start()

    await vi.waitFor(() => {
      expect(video.buffered.length).toBe(1)
      expect(video.buffered.start(0)).toBeGreaterThan(1.2)
      expect(video.currentTime).toBeGreaterThanOrEqual(video.buffered.start(0))
    })
    player.destroy()
  })

  it('plays frames of a followed recording that another caller indexed while it waited', async () => {
    const reader = await McapIndexedReader.open(new MemoryByteSource(await buildIndexedVideoMcap(40)))
    const [track] = listVideoTracks(reader)
    const { chunkIndexes, startTime, endTime } = reader.summary
    const unwritten = chunkIndexes.splice(chunkIndexes.length - 3)
    reader.summary.endTime = chunkIndexes[chunkIndexes.length - 1].endTime
    const recording = {
      reader,
      tracks: [track],
      channels: [],
      durationSeconds: Number(reader.summary.endTime - startTime) / 1e9,
      startTime,
    }
    const video = new FakeVideo()
    const player = new McapVideoPlayer(video as unknown as HTMLVideoElement, recording, track, { follow: true })
    await player.start()
    await vi.waitFor(() => expect(player.stats.waiting).toBe(true))

    chunkIndexes.push(...unwritten)
    reader.summary.endTime = endTime

    await vi.waitFor(() => {
      expect(video.buffered.end(0)).toBeGreaterThan(recording.durationSeconds + FRAME_SECONDS)
    }, { timeout: 3000 })
    player.destroy()
  })

  it('reads a stream seeked while it was still starting once, in order, from the seek target only', async () => {
    vi.stubGlobal('MediaSource', UnopenedMediaSource)
    const reader = await McapIndexedReader.open(new MemoryByteSource(await buildTwoTrackVideoMcap()))
    const [, cameraB] = listVideoTracks(reader)
    const { startTime, endTime } = reader.summary
    const recording = {
      reader, tracks: [cameraB], channels: [], durationSeconds: Number(endTime - startTime) / 1e9, startTime,
    }
    const startSeconds = 6
    const target = 12
    const video = new FakeVideo()
    const onError = vi.fn()
    const player = new McapVideoPlayer(video as unknown as HTMLVideoElement, recording, cameraB, {
      startSeconds,
      onError,
    })

    const started = player.start()
    player.seek(target)
    await new Promise((resolve) => { setTimeout(resolve) })
    UnopenedMediaSource.last?.open()
    await started

    const lastCameraBFrame = 15.25
    await vi.waitFor(() => {
      expect(onError.mock.calls.length > 0 || video.buffered.end(0) > lastCameraBFrame).toBe(true)
    }, { timeout: 3000 })
    expect(onError).not.toHaveBeenCalled()
    const appended = sourceBuffer?.appendedStarts ?? []
    expect(appended).toEqual([...new Set(appended)].sort((left, right) => left - right))
    expect(appended[0]).toBeGreaterThan(startSeconds + 1)
    expect(appended[0]).toBeLessThanOrEqual(target)
    expect(video.currentTime).toBeGreaterThanOrEqual(appended[0])
    expect(video.currentTime).toBeLessThanOrEqual(target)
    player.destroy()
  })

  it('reads from its start time on a video element left at a later time', async () => {
    const reader = await McapIndexedReader.open(new MemoryByteSource(await buildTwoTrackVideoMcap()))
    const [, cameraB] = listVideoTracks(reader)
    const { startTime, endTime } = reader.summary
    const recording = {
      reader, tracks: [cameraB], channels: [], durationSeconds: Number(endTime - startTime) / 1e9, startTime,
    }
    const startSeconds = 6
    const video = new ReusedVideo()
    video.currentTime = 14
    const player = new McapVideoPlayer(video as unknown as HTMLVideoElement, recording, cameraB, {
      startSeconds,
      bufferAheadSeconds: 2,
    })

    await player.start()

    await vi.waitFor(() => expect(Math.max(...sourceBuffer?.appendedStarts ?? [])).toBeGreaterThan(startSeconds + 1))
    expect(Math.max(...sourceBuffer?.appendedStarts ?? [])).toBeLessThan(startSeconds + 4)
    expect(video.currentTime).toBeGreaterThan(startSeconds - 1)
    expect(video.currentTime).toBeLessThan(startSeconds + 1)
    player.destroy()
  })
})
