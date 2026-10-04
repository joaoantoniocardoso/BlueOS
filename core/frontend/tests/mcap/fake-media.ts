/* eslint-disable class-methods-use-this, max-classes-per-file */
import { vi } from 'vitest'

/** `HTMLMediaElement.readyState` values the fake reports. */
export const HAVE_NOTHING = 0
export const HAVE_METADATA = 1
export const HAVE_ENOUGH_DATA = 4

/** How far a playing element moves per `advance()`, as a browser moves between two `timeupdate` events (~250 ms). */
export const TIME_UPDATE_SECONDS = 0.25

const NO_RANGES: TimeRanges = { length: 0, start: () => 0, end: () => 0 }

let installed: FakeMedia | null = null

function media(): FakeMedia {
  if (!installed) {
    throw new Error('Call installFakeMedia() before using the fake media classes.')
  }
  return installed
}

/** HTML "queue a media element task": events reach listeners from a later task, never from inside the call. */
function queueTask(task: () => void): void {
  setTimeout(task, 0)
}

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

/** What `installFakeMedia` set up: the sources and buffers the code under test created, and control of `sourceopen`. */
export class FakeMedia {
  /** Every media source created, by the object URL it was given. */
  readonly mediaSources = new Map<string, FakeMediaSource>()

  /** Every source buffer created, in creation order. */
  readonly sourceBuffers: FakeSourceBuffer[] = []

  /** While true, a media source attached to an element stays `closed` until `openHeldSources()`. */
  holdSourceOpen = false

  /** What `MediaSource.isTypeSupported()` answers. */
  typeSupported = true

  private heldSources: FakeMediaSource[] = []

  constructor(readonly options: { frameSeconds: number, appendMilliseconds: number }) {}

  hold(source: FakeMediaSource): void {
    this.heldSources.push(source)
  }

  openHeldSources(): void {
    this.heldSources.splice(0).forEach((source) => source.open())
  }
}

/** Replaces `MediaSource` and the object URL functions; the fake elements need this to find their sources. */
export function installFakeMedia(
  options: { frameSeconds: number, appendMilliseconds?: number },
): FakeMedia {
  const fake = new FakeMedia({ appendMilliseconds: 0, ...options })
  installed = fake
  vi.stubGlobal('MediaSource', FakeMediaSource)
  vi.spyOn(URL, 'createObjectURL').mockImplementation((source) => {
    const url = `blob:fake-media-${fake.mediaSources.size}`
    fake.mediaSources.set(url, source as unknown as FakeMediaSource)
    return url
  })
  vi.spyOn(URL, 'revokeObjectURL').mockReturnValue(undefined)
  return fake
}

/**
 * Holds the fragments appended so far as one range, from the first `tfdt` decode time to one frame past the last.
 * That range is a simplification: fragments are taken to be contiguous.
 */
export class FakeSourceBuffer extends EventTarget {
  mode = 'segments'

  /** MSE: true from `appendBuffer()`/`remove()` until their `updateend`. */
  updating = false

  /** Start of every fragment appended, in append order, whatever was removed since. */
  appendedStarts: number[] = []

  /** Every range `remove()` was asked for, in order. */
  removed: Array<[number, number]> = []

  private fragmentStarts: number[] = []

  private timescale = 1

  constructor(private readonly source: FakeMediaSource) {
    super()
  }

  get buffered(): TimeRanges {
    if (this.fragmentStarts.length === 0) {
      return NO_RANGES
    }
    const start = Math.min(...this.fragmentStarts)
    const end = Math.max(...this.fragmentStarts) + media().options.frameSeconds
    return { length: 1, start: () => start, end: () => end }
  }

  /** MSE "buffer append": throws `InvalidStateError` while updating, and the data lands, with `updateend`, later. */
  appendBuffer(data: Uint8Array): void {
    this.startUpdate()
    const view = new DataView(data.buffer, data.byteOffset, data.byteLength)
    const mdhd = boxPayload(data, 'mdhd')
    if (mdhd >= 0) {
      this.timescale = view.getUint32(mdhd + 4 + (data[mdhd] === 1 ? 16 : 8))
    }
    const tfdt = boxPayload(data, 'tfdt')
    const decodeTime = tfdt < 0
      ? null
      : Number(data[tfdt] === 1 ? view.getBigUint64(tfdt + 4) : view.getUint32(tfdt + 4)) / this.timescale
    if (decodeTime !== null) {
      this.appendedStarts.push(decodeTime)
    }
    setTimeout(() => {
      if (decodeTime !== null) {
        this.fragmentStarts.push(decodeTime)
      }
      this.endUpdate(mdhd >= 0)
    }, media().options.appendMilliseconds)
  }

  /** MSE "range removal": drops the fragments that start in `[start, end)`; throws while updating. */
  remove(start: number, end: number): void {
    this.startUpdate()
    this.removed.push([start, end])
    queueTask(() => {
      this.fragmentStarts = this.fragmentStarts.filter((fragmentStart) => fragmentStart < start || fragmentStart >= end)
      this.endUpdate(false)
    })
  }

  private startUpdate(): void {
    if (this.updating) {
      throw new DOMException('The source buffer is still processing an earlier operation.', 'InvalidStateError')
    }
    this.updating = true
  }

  private endUpdate(initSegment: boolean): void {
    this.updating = false
    this.source.video?.onBufferChanged(initSegment)
    this.dispatchEvent(new Event('updateend'))
  }
}

export class FakeMediaSource extends EventTarget {
  static isTypeSupported(): boolean {
    return media().typeSupported
  }

  /** MSE: `closed` until attached to an element, `open` once it fires `sourceopen`, `ended` after `endOfStream()`. */
  readyState = 'closed'

  duration = Number.NaN

  sourceBuffer: FakeSourceBuffer | null = null

  /** The element this source is attached to. */
  video: FakeVideo | null = null

  addSourceBuffer(): FakeSourceBuffer {
    this.sourceBuffer = new FakeSourceBuffer(this)
    media().sourceBuffers.push(this.sourceBuffer)
    return this.sourceBuffer
  }

  endOfStream(): void {
    this.readyState = 'ended'
  }

  open(): void {
    this.readyState = 'open'
    this.dispatchEvent(new Event('sourceopen'))
  }
}

/** A `<video>` playing the media of its own source, one `advance()` at a time. */
export class FakeVideo extends EventTarget {
  paused = true

  muted = false

  /** HTML: the rate the playback position advances at, which the controller copies from leader to followers. */
  playbackRate = 1

  /** HTML seek algorithm: true from setting `currentTime` until the data at the new position is available. */
  seeking = false

  private srcAttribute = ''

  private time = 0

  private defaultPlaybackStart = 0

  private metadataLoaded = false

  private networkEmpty = true

  private stalled = false

  private seekAnnounced = false

  private source: FakeMediaSource | null = null

  /** HTML "remove pending tasks": the load algorithm cancels every event task queued before it. */
  private generation = 0

  get src(): string {
    return this.srcAttribute
  }

  /** HTML: setting `src` runs the load algorithm. */
  set src(url: string) {
    this.srcAttribute = url
    this.load()
  }

  /** HTML: the default playback start position, when set and nonzero, is the current playback position. */
  get currentTime(): number {
    return this.defaultPlaybackStart || this.time
  }

  /**
   * HTML: with no media (`HAVE_NOTHING`) the time is kept as the default playback start position and applied
   * when the metadata loads; otherwise it starts a seek, which fires `seeking`.
   */
  set currentTime(seconds: number) {
    if (this.readyState === HAVE_NOTHING) {
      this.defaultPlaybackStart = seconds
      return
    }
    this.seek(seconds)
  }

  get buffered(): TimeRanges {
    return this.source?.sourceBuffer?.buffered ?? NO_RANGES
  }

  /** HTML ready states: `HAVE_NOTHING` until the metadata loads, then enough data only inside a buffered range. */
  get readyState(): number {
    if (!this.metadataLoaded) {
      return HAVE_NOTHING
    }
    const { buffered } = this
    const covered = buffered.length > 0 && this.time >= buffered.start(0) && this.time < buffered.end(0)
    return covered ? HAVE_ENOUGH_DATA : HAVE_METADATA
  }

  /** HTML play(): fires `play` when it unpauses, then `playing`, or `waiting` when there is no data to play. */
  play(): Promise<void> {
    if (this.paused) {
      this.paused = false
      this.fire('play')
      this.stalled = this.readyState < HAVE_ENOUGH_DATA
      this.fire(this.stalled ? 'waiting' : 'playing')
    }
    return Promise.resolve()
  }

  /** HTML pause(): fires `timeupdate` then `pause` when it pauses. */
  pause(): void {
    if (!this.paused) {
      this.paused = true
      this.fire('timeupdate')
      this.fire('pause')
    }
  }

  /** HTML: removing the `src` attribute changes nothing until the next `load()`. */
  removeAttribute(): void {
    this.srcAttribute = ''
  }

  /**
   * HTML load algorithm. An element that had media queues `emptied`, drops its metadata, pauses, and goes back to
   * time 0 with a `timeupdate` if that changed it; the default playback start position survives. A source set on
   * `src` attaches and fires `sourceopen` from a later task.
   */
  load(): void {
    this.generation += 1
    if (this.source) {
      this.source.video = null
      this.source.readyState = 'closed'
      this.source = null
    }
    if (!this.networkEmpty) {
      this.fire('emptied')
      this.metadataLoaded = false
      this.seeking = false
      this.paused = true
      this.stalled = false
      if (this.time !== 0) {
        this.time = 0
        this.fire('timeupdate')
      }
    }
    this.networkEmpty = this.srcAttribute === ''
    const source = media().mediaSources.get(this.srcAttribute)
    if (source) {
      source.video = this
      this.source = source
      queueTask(() => {
        if (media().holdSourceOpen) {
          media().hold(source)
        } else {
          source.open()
        }
      })
    }
  }

  /** The source buffer appended or removed data: an init segment loads the metadata, and covered seeks finish. */
  onBufferChanged(initSegment: boolean): void {
    if (initSegment && !this.metadataLoaded) {
      this.metadataLoaded = true
      this.fire('loadedmetadata')
      const start = this.defaultPlaybackStart
      this.defaultPlaybackStart = 0
      if (start > 0) {
        this.seek(start)
      }
    }
    const { generation } = this
    queueTask(() => {
      if (generation === this.generation) {
        this.finishSeekIfCovered()
      }
    })
  }

  /** Plays on until the next `timeupdate`, stalling where nothing is buffered, as a browser does. */
  advance(): void {
    if (this.paused || this.seeking) {
      return
    }
    if (this.readyState < HAVE_ENOUGH_DATA) {
      if (!this.stalled) {
        this.stalled = true
        this.fire('waiting')
      }
      return
    }
    this.stalled = false
    this.time = Math.min(this.time + TIME_UPDATE_SECONDS * this.playbackRate, this.buffered.end(0))
    this.fire('timeupdate')
  }

  /** HTML "queue a media element task to fire an event", dropped if the element reloaded meanwhile. */
  private fire(type: string): void {
    const { generation } = this
    queueTask(() => {
      if (generation === this.generation) {
        this.dispatchEvent(new Event(type))
      }
    })
  }

  /** HTML seek algorithm: moves at once, fires `seeking` and `timeupdate`, and `seeked` once data covers it. */
  private seek(seconds: number): void {
    this.time = seconds
    this.seeking = true
    this.seekAnnounced = false
    const { generation } = this
    queueTask(() => {
      if (generation !== this.generation) {
        return
      }
      this.seekAnnounced = true
      this.dispatchEvent(new Event('seeking'))
      this.dispatchEvent(new Event('timeupdate'))
      this.finishSeekIfCovered()
    })
  }

  private finishSeekIfCovered(): void {
    if (this.seeking && this.seekAnnounced && this.readyState === HAVE_ENOUGH_DATA) {
      this.seeking = false
      this.dispatchEvent(new Event('seeked'))
    }
  }
}
