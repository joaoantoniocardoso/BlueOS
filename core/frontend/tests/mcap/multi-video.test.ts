/* eslint-disable class-methods-use-this, max-classes-per-file */
import {
  afterEach, describe, expect, it, vi,
} from 'vitest'

import { exportTrackAsMp4 } from '@/libs/mcap/adapters/export'
import {
  McapRecordingPlaybackController,
  type StreamPlaybackControl,
} from '@/libs/mcap/adapters/mcap-recording-playback-controller'
import { McapStreamPanelController } from '@/libs/mcap/adapters/mcap-stream-panel-controller'
import type { McapVideoRecording } from '@/libs/mcap/adapters/player'
import VideoFrameStream from '@/libs/mcap/logic/frame-stream'
import { mergedVideoCoverage, trackCoversAt, trackTimelineLanes } from '@/libs/mcap/logic/playback-ui'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'
import { listVideoTracks, type VideoTrack } from '@/libs/mcap/logic/video-track'

import {
  asLiveRecording, buildLateVideoMcap, buildTwoTrackVideoMcap, TWO_TRACK_FRAME_NS,
} from './build-mcap'
import MemoryByteSource from './memory-byte-source'

/** How far apart two streams may play and still count as in step, the controller's sync tolerance. */
const IN_STEP_SECONDS = 0.5

/** How far a playing `<video>` moves between two `timeupdate` events, which browsers fire about every 250 ms. */
const TIME_UPDATE_SECONDS = 0.25

/** `HTMLMediaElement.readyState` values the fake reports. */
const HAVE_METADATA = 1
const HAVE_ENOUGH_DATA = 4

const NO_RANGES: TimeRanges = { length: 0, start: () => 0, end: () => 0 }

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

/** Buffers the fragments appended so far as one range, from the first `tfdt` decode time to one frame past the last. */
class FakeSourceBuffer extends EventTarget {
  mode = 'segments'

  private fragmentStarts: number[] = []

  private timescale = 1

  get buffered(): TimeRanges {
    if (this.fragmentStarts.length === 0) {
      return NO_RANGES
    }
    const start = Math.min(...this.fragmentStarts)
    const end = Math.max(...this.fragmentStarts) + Number(TWO_TRACK_FRAME_NS) / 1e9
    return { length: 1, start: () => start, end: () => end }
  }

  appendBuffer(data: Uint8Array): void {
    const view = new DataView(data.buffer, data.byteOffset, data.byteLength)
    const mdhd = boxPayload(data, 'mdhd')
    if (mdhd >= 0) {
      this.timescale = view.getUint32(mdhd + 4 + (data[mdhd] === 1 ? 16 : 8))
    }
    const tfdt = boxPayload(data, 'tfdt')
    const { timescale } = this
    setTimeout(() => {
      if (tfdt >= 0) {
        this.fragmentStarts.push(Number(data[tfdt] === 1 ? view.getBigUint64(tfdt + 4) : view.getUint32(tfdt + 4))
          / timescale)
      }
      this.dispatchEvent(new Event('updateend'))
    })
  }

  remove(): void {
    this.fragmentStarts = []
    setTimeout(() => this.dispatchEvent(new Event('updateend')))
  }
}

/** Every media source a player created, by the object URL it handed to its `<video>`. */
const mediaSources = new Map<string, FakeMediaSource>()

class FakeMediaSource extends EventTarget {
  static isTypeSupported(): boolean {
    return true
  }

  readyState = 'open'

  duration = Number.NaN

  sourceBuffer: FakeSourceBuffer | null = null

  addSourceBuffer(): FakeSourceBuffer {
    this.sourceBuffer = new FakeSourceBuffer()
    return this.sourceBuffer
  }

  endOfStream(): void {
    this.readyState = 'ended'
  }
}

/** A `<video>` playing the media of its own source, one `advance()` at a time. */
class FakeVideo extends EventTarget {
  paused = true

  seeking = false

  playbackRate = 1

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
    return mediaSources.get(this.src)?.sourceBuffer?.buffered ?? NO_RANGES
  }

  get readyState(): number {
    const { buffered } = this
    if (buffered.length > 0 && this.time >= buffered.start(0) && this.time < buffered.end(0)) {
      return HAVE_ENOUGH_DATA
    }
    return buffered.length > 0 ? HAVE_METADATA : 0
  }

  play(): Promise<void> {
    if (this.paused) {
      this.paused = false
      this.dispatchEvent(new Event('play'))
    }
    return Promise.resolve()
  }

  pause(): void {
    if (!this.paused) {
      this.paused = true
      this.dispatchEvent(new Event('pause'))
    }
  }

  removeAttribute(): void {
    this.src = ''
  }

  load(): void {
    this.time = 0
  }

  /** Plays on until the next `timeupdate`, stalling where nothing is buffered, as a browser does. */
  advance(): void {
    if (this.paused) {
      return
    }
    if (this.readyState < HAVE_ENOUGH_DATA) {
      this.dispatchEvent(new Event('waiting'))
      return
    }
    this.time = Math.min(this.time + TIME_UPDATE_SECONDS, this.buffered.end(0))
    this.dispatchEvent(new Event('timeupdate'))
  }
}

interface PlayingStream {
  video: FakeVideo
  panel: McapStreamPanelController
  control: StreamPlaybackControl
  /** Opens or closes the stream when the playhead enters or leaves its range, as its `available` watcher does. */
  followPosition: () => void
}

/**
 * Plays a finished recording with a panel and a fake `<video>` per stream, wired as `McapVideoPlayer.vue` does.
 * `mountStream` mounts one more, as the page does when a hidden stream is shown again.
 */
async function mountPlayingStreams(bytes: Uint8Array): Promise<{
  controller: McapRecordingPlaybackController,
  streams: PlayingStream[],
  mountStream: (track: VideoTrack) => PlayingStream,
}> {
  serveOverHttp(() => bytes)
  vi.stubGlobal('MediaSource', FakeMediaSource)
  vi.spyOn(URL, 'createObjectURL').mockImplementation((source) => {
    const url = `blob:${mediaSources.size}`
    mediaSources.set(url, source as unknown as FakeMediaSource)
    return url
  })
  vi.spyOn(URL, 'revokeObjectURL').mockReturnValue(undefined)
  const controller = new McapRecordingPlaybackController({
    url: 'http://vehicle/userdata/recorder/two.mcap',
    ongoing: false,
    callbacks: {
      onState: () => undefined,
      onBusy: () => undefined,
      onSummary: () => undefined,
      onMp4Saved: () => undefined,
    },
  })
  await controller.mount()
  const { recording, tracks } = controller.getState()
  function mountStream(track: VideoTrack): PlayingStream {
    const video = new FakeVideo()
    const element = video as unknown as HTMLVideoElement
    let available = trackCoversAt(track, controller.getState().position)
    const panel = new McapStreamPanelController(recording as McapVideoRecording, track, false, {
      onState: () => undefined,
      onStats: (stats) => controller.onStreamStats(track.channelId, stats),
      onTimeUpdate: (seconds) => controller.onStreamTime(track.channelId, seconds),
      onPlay: () => controller.onLeaderPlay(),
      onPause: () => controller.onLeaderPause(),
    })
    video.addEventListener('timeupdate', () => panel.handleTimeUpdate(element))
    video.addEventListener('play', () => panel.handlePlay())
    video.addEventListener('pause', () => panel.handlePause(available))
    controller.registerVideo(track.channelId, element)
    panel.setAvailable(available, element, controller.getState().position)
    return {
      video,
      panel,
      control: {
        channelId: track.channelId,
        seek: (seconds) => panel.seek(seconds),
        play: () => panel.play(element),
        pause: () => panel.pause(element),
      },
      followPosition: () => {
        const { position } = controller.getState()
        if (trackCoversAt(track, position) !== available) {
          available = !available
          panel.setAvailable(available, element, position)
        }
      },
    }
  }
  const streams = tracks.map(mountStream)
  controller.setStreamControls(streams.map(({ control }) => control))
  return { controller, streams, mountStream }
}

async function openTwoTracks(): Promise<McapVideoRecording> {
  const reader = await McapIndexedReader.open(new MemoryByteSource(await buildTwoTrackVideoMcap()))
  const { startTime, endTime } = reader.summary
  return {
    reader,
    tracks: listVideoTracks(reader),
    channels: [],
    durationSeconds: Number(endTime - startTime) / 1e9,
    startTime,
  }
}

/** The `sample_count` of the first `stsz` box: how many frames the MP4 holds. */
async function mp4SampleCount(file: Blob): Promise<number> {
  const data = new Uint8Array(await file.arrayBuffer())
  const view = new DataView(data.buffer)
  const code = [...'stsz'].map((character) => character.charCodeAt(0))
  for (let index = 4; index + 16 <= data.length; index += 1) {
    if (code.every((byte, offset) => data[index + offset] === byte)) {
      return view.getUint32(index + 12)
    }
  }
  throw new Error('No stsz box')
}

/** Serves the file that `written` returns to `HttpByteSource` the way nginx answers its range requests. */
function serveOverHttp(written: () => Uint8Array): void {
  vi.stubGlobal('fetch', vi.fn(async (_url: string, init?: RequestInit) => {
    const bytes = written()
    const range = (init?.headers as Record<string, string>).Range
    const [, first, last] = /^bytes=(\d*)-(\d*)$/.exec(range) ?? []
    const start = first === '' ? Math.max(0, bytes.length - Number(last)) : Number(first)
    const end = first === '' || last === '' ? bytes.length - 1 : Math.min(Number(last), bytes.length - 1)
    return new Response(bytes.slice(start, end + 1), {
      status: 206,
      headers: { 'Content-Range': `bytes ${start}-${end}/${bytes.length}` },
    })
  }))
}

async function mountTwoTrackPlayer(onMp4Saved: (blob: Blob, fileName: string) => void = () => undefined):
  Promise<McapRecordingPlaybackController> {
  const bytes = await buildTwoTrackVideoMcap()
  serveOverHttp(() => bytes)
  const controller = new McapRecordingPlaybackController({
    url: 'http://vehicle/userdata/recorder/two.mcap',
    ongoing: false,
    callbacks: {
      onState: () => undefined,
      onBusy: () => undefined,
      onSummary: () => undefined,
      onMp4Saved,
    },
  })
  await controller.mount()
  return controller
}

function fakeStreamControl(channelId: number): StreamPlaybackControl & { calls: string[] } {
  const calls: string[] = []
  return {
    channelId,
    calls,
    seek: (seconds) => calls.push(`seek ${seconds}`),
    play: () => calls.push('play'),
    pause: () => calls.push('pause'),
  }
}

describe('a recording with two video streams', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('lists both streams, each with its own coverage', async () => {
    const { tracks } = await openTwoTracks()
    expect(tracks.map((track) => [track.name, track.frameCount])).toEqual([['camera_a', 41], ['camera_b', 21]])
    const [cameraA, cameraB] = tracks.map(({ coverage }) => [coverage[0].start, coverage[coverage.length - 1].end])
    expect(cameraA[0]).toBeLessThan(1)
    expect(cameraA[1]).toBeGreaterThan(19)
    expect(cameraB[0]).toBeGreaterThan(cameraA[0] + 3)
    expect(cameraB[1]).toBeLessThan(cameraA[1] - 1)
  })

  it('covers each stream from its first frame to the end of its last one, not to the edges of its chunks', async () => {
    const controller = await mountTwoTrackPlayer()
    const [, cameraB] = controller.getState().tracks
    // camera_b has a frame every 0.5 s from 5.25 to 15.25 s, in chunks that span 5 to 17.5 s.
    expect(cameraB.coverage).toEqual([{ start: 5.25, end: 15.75 }])
    controller.destroy()
  })

  it('never yields a frame of the other stream from a shared chunk', async () => {
    const recording = await openTwoTracks()
    const stream = new VideoFrameStream(recording.reader, recording.tracks[1])
    stream.seekToStart()
    const offsets: bigint[] = []
    for (let frame = await stream.next(); frame; frame = await stream.next()) {
      offsets.push((frame.logTime - recording.startTime) % TWO_TRACK_FRAME_NS)
    }
    expect(offsets).toHaveLength(21)
    expect(new Set(offsets)).toEqual(new Set([TWO_TRACK_FRAME_NS / 2n]))
  })

  it('exports each stream as an MP4 of its own frames only', async () => {
    const recording = await openTwoTracks()
    const [cameraA, cameraB] = recording.tracks
    expect(await mp4SampleCount(await exportTrackAsMp4(recording, cameraA))).toBe(41)
    expect(await mp4SampleCount(await exportTrackAsMp4(recording, cameraB))).toBe(21)
  })

  it('exports only the visible streams, each under its own name', async () => {
    const saved: string[] = []
    const controller = await mountTwoTrackPlayer((_blob, fileName) => saved.push(fileName))
    const [cameraA] = controller.getState().tracks

    controller.setSelectedChannelIds([cameraA.channelId])
    await controller.saveMp4('two', null)
    expect(saved).toEqual(['two-camera_a.mp4'])

    saved.length = 0
    controller.selectAllStreams()
    await controller.saveMp4('two', null)
    expect(saved).toEqual(['two-camera_a.mp4', 'two-camera_b.mp4'])
    controller.destroy()
  })

  it('saves the other streams and names the one with no keyframe in the cut', async () => {
    const saved: string[] = []
    const controller = await mountTwoTrackPlayer((_blob, fileName) => saved.push(fileName))

    await controller.saveMp4('two', { startSeconds: 0, endSeconds: 3 })

    expect(saved).toEqual(['two-camera_a-0s-3s.mp4'])
    expect(controller.getState().exportNotice).toMatch(/camera_b/)
    controller.destroy()
  })

  it('seeks, plays and pauses every stream', async () => {
    const controller = await mountTwoTrackPlayer()
    const streams = controller.getState().tracks.map((track) => fakeStreamControl(track.channelId))
    controller.setStreamControls(streams)

    controller.seekTo(10)
    controller.togglePlayback()

    for (const stream of streams) {
      expect(stream.calls).toEqual(['seek 10', 'play', 'pause'])
    }
    controller.destroy()
  })

  it('moves playback to the next covered time, or stops, when the stream at the playhead is hidden', async () => {
    const controller = await mountTwoTrackPlayer()
    const [cameraA, cameraB] = controller.getState().tracks
    controller.setStreamControls([fakeStreamControl(cameraA.channelId), fakeStreamControl(cameraB.channelId)])

    controller.seekTo(2)
    controller.toggleStream(cameraA.channelId)
    expect(controller.getState()).toMatchObject({ position: cameraB.coverage[0].start, playing: true })

    controller.toggleStream(cameraA.channelId)
    controller.seekTo(18)
    controller.setSelectedChannelIds([cameraB.channelId])
    expect(controller.getState().playing).toBe(false)
    controller.destroy()
  })
})

describe('two streams playing together', () => {
  afterEach(() => {
    mediaSources.clear()
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('holds a stream that comes into range ahead of its first keyframe until playback reaches it', async () => {
    const { controller, streams } = await mountPlayingStreams(await buildTwoTrackVideoMcap(6))
    const [cameraA, cameraB] = streams
    const cameraBFirstKeyframe = 8.25
    const [, cameraBTrack] = controller.getState().tracks
    expect(cameraBTrack.coverage[0].start).toBeLessThan(cameraBFirstKeyframe - 2)

    controller.seekTo(3)
    await vi.waitFor(() => expect(cameraA.video.readyState).toBe(HAVE_ENOUGH_DATA))
    const samples: { cameraA: number, cameraB: number, cameraBPlaying: boolean }[] = []
    for (let step = 0; step < 80 && cameraA.video.currentTime < cameraBFirstKeyframe + 3; step += 1) {
      streams.forEach(({ video }) => video.advance())
      streams.forEach(({ followPosition }) => followPosition())
      samples.push({
        cameraA: cameraA.video.currentTime,
        cameraB: cameraB.video.currentTime,
        cameraBPlaying: !cameraB.video.paused && cameraB.video.readyState === HAVE_ENOUGH_DATA,
      })
      // eslint-disable-next-line no-await-in-loop
      await new Promise((resolve) => { setTimeout(resolve, 10) })
    }

    const playingAhead = samples.filter((sample) => sample.cameraA < cameraBFirstKeyframe && sample.cameraBPlaying)
    expect(playingAhead).toEqual([])
    const playingAfter = samples.filter((sample) => sample.cameraA >= cameraBFirstKeyframe + TIME_UPDATE_SECONDS)
    expect(playingAfter.length).toBeGreaterThan(0)
    for (const sample of playingAfter) {
      expect(sample.cameraBPlaying).toBe(true)
      expect(Math.abs(sample.cameraB - sample.cameraA)).toBeLessThanOrEqual(IN_STEP_SECONDS)
    }
    streams.forEach(({ panel }) => panel.destroy())
    controller.destroy()
  })

  it('shows a hidden stream at the playhead without moving the other one back', async () => {
    const { controller, streams, mountStream } = await mountPlayingStreams(await buildTwoTrackVideoMcap())
    const [cameraA, cameraB] = streams
    const [cameraATrack] = controller.getState().tracks
    controller.seekTo(8)
    streams.forEach(({ followPosition }) => followPosition())
    await vi.waitFor(() => expect(cameraB.video.readyState).toBe(HAVE_ENOUGH_DATA))
    controller.toggleStream(cameraATrack.channelId)
    cameraA.panel.destroy()
    controller.setStreamControls([cameraB.control])
    for (let step = 0; step < 6; step += 1) {
      cameraB.video.advance()
      // eslint-disable-next-line no-await-in-loop
      await new Promise((resolve) => { setTimeout(resolve, 10) })
    }

    controller.toggleStream(cameraATrack.channelId)
    const shown = mountStream(cameraATrack)
    controller.setStreamControls([shown.control, cameraB.control])
    const samples: { position: number, cameraB: number }[] = []
    for (let step = 0; step < 12; step += 1) {
      [shown, cameraB].forEach(({ video }) => video.advance())
      samples.push({ position: controller.getState().position, cameraB: cameraB.video.currentTime })
      // eslint-disable-next-line no-await-in-loop
      await new Promise((resolve) => { setTimeout(resolve, 10) })
    }

    const positions = samples.map((sample) => sample.position)
    expect(positions).toEqual([...positions].sort((left, right) => left - right))
    samples.slice(1).forEach((sample, index) => {
      expect(sample.cameraB).toBeGreaterThan(samples[index].cameraB - IN_STEP_SECONDS)
    })
    expect(Math.abs(shown.video.currentTime - cameraB.video.currentTime)).toBeLessThanOrEqual(IN_STEP_SECONDS)
    shown.panel.destroy()
    cameraB.panel.destroy()
    controller.destroy()
  })

  it('stops once the only visible stream plays its last frame, and plays again from its start', async () => {
    const { controller, streams } = await mountPlayingStreams(await buildTwoTrackVideoMcap())
    const [cameraA, cameraB] = streams
    const [cameraATrack, cameraBTrack] = controller.getState().tracks
    controller.toggleStream(cameraATrack.channelId)
    cameraA.panel.destroy()
    controller.setStreamControls([cameraB.control])
    controller.seekTo(13)
    cameraB.followPosition()
    await vi.waitFor(() => expect(cameraB.video.readyState).toBe(HAVE_ENOUGH_DATA))

    let played = 0
    for (let step = 0; step < 40 && controller.getState().playing; step += 1) {
      cameraB.video.advance()
      played = Math.max(played, cameraB.video.currentTime)
      cameraB.followPosition()
      // eslint-disable-next-line no-await-in-loop
      await new Promise((resolve) => { setTimeout(resolve, 10) })
    }

    const cameraBLastFrame = 15.25
    expect(played).toBeGreaterThan(cameraBLastFrame)
    expect(controller.getState().playing).toBe(false)
    controller.togglePlayback()
    expect(controller.getState().position).toBe(cameraBTrack.coverage[0].start)
    expect(controller.getState().playing).toBe(true)
    cameraB.panel.destroy()
    controller.destroy()
  })

  it('leaves a stream on its last frame while the other plays on, when its coverage runs past it', async () => {
    const { controller, streams } = await mountPlayingStreams(await buildTwoTrackVideoMcap(0, false))
    const [cameraA, cameraB] = streams
    const cameraBLastFrame = 15.25
    expect(trackCoversAt(controller.getState().tracks[1], cameraBLastFrame + 3)).toBe(true)
    let cameraBSeeks = 0
    cameraB.video.addEventListener('seeking', () => { cameraBSeeks += 1 })

    controller.seekTo(12)
    await vi.waitFor(() => expect(cameraA.video.readyState).toBe(HAVE_ENOUGH_DATA))
    const samples: { cameraA: number, cameraB: number, cameraBSeeks: number }[] = []
    for (let step = 0; step < 80 && cameraA.video.currentTime < cameraBLastFrame + 3; step += 1) {
      streams.forEach(({ video }) => video.advance())
      streams.forEach(({ followPosition }) => followPosition())
      samples.push({ cameraA: cameraA.video.currentTime, cameraB: cameraB.video.currentTime, cameraBSeeks })
      // eslint-disable-next-line no-await-in-loop
      await new Promise((resolve) => { setTimeout(resolve, 10) })
    }

    const inStep = samples.filter((sample) => sample.cameraA >= 13).map((sample) => sample.cameraB)
    expect(inStep).toEqual([...inStep].sort((left, right) => left - right))
    const afterLastFrame = samples.filter((sample) => sample.cameraA > cameraBLastFrame + 1)
    expect(afterLastFrame.length).toBeGreaterThan(0)
    expect(new Set(afterLastFrame.map((sample) => sample.cameraBSeeks)).size).toBe(1)
    streams.forEach(({ panel }) => panel.destroy())
    controller.destroy()
  })
})

describe('a live recording', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('adds and selects a stream that started after the player opened, without reopening', async () => {
    const finished = await buildLateVideoMcap()
    const { chunks } = await asLiveRecording(finished, 1)
    const live = await asLiveRecording(finished, chunks.findIndex((chunk) => chunk.channelIds.length === 2))
    serveOverHttp(live.bytes)
    const controller = new McapRecordingPlaybackController({
      url: 'http://vehicle/userdata/recorder/live.mcap',
      indexSource: live.indexSource,
      ongoing: true,
      callbacks: {
        onState: () => undefined,
        onBusy: () => undefined,
        onSummary: () => undefined,
        onMp4Saved: () => undefined,
      },
    })
    await controller.mount()
    expect(controller.getState().tracks).toEqual([])

    live.writtenChunks = chunks.length
    controller.onWrittenSizeBytes(live.bytes().length)

    await vi.waitFor(() => expect(controller.getState().tracks.map((track) => track.name)).toEqual(['camera']))
    const [camera] = controller.getState().tracks
    expect(controller.getState().selectedChannelIds).toEqual([camera.channelId])
    controller.destroy()
  })

  it('covers a stream that started after the player opened from its first frame to its last one', async () => {
    const live = await asLiveRecording(await buildTwoTrackVideoMcap(), 1)
    serveOverHttp(live.bytes)
    const controller = new McapRecordingPlaybackController({
      url: 'http://vehicle/userdata/recorder/live.mcap',
      indexSource: live.indexSource,
      ongoing: true,
      callbacks: {
        onState: () => undefined,
        onBusy: () => undefined,
        onSummary: () => undefined,
        onMp4Saved: () => undefined,
      },
    })
    await controller.mount()
    expect(controller.getState().tracks.map((track) => track.name)).toEqual(['camera_a'])

    live.writtenChunks = live.chunks.length
    controller.onWrittenSizeBytes(live.bytes().length)

    await vi.waitFor(() => {
      expect(controller.getState().tracks[1]?.coverage).toEqual([{ start: 5.25, end: 15.75 }])
    })
    controller.destroy()
  })

  it('plays the first stream added to a player with none from its latest frames', async () => {
    const finished = await buildLateVideoMcap()
    const { chunks } = await asLiveRecording(finished, 1)
    const live = await asLiveRecording(finished, chunks.findIndex((chunk) => chunk.channelIds.length === 2))
    serveOverHttp(live.bytes)
    const controller = new McapRecordingPlaybackController({
      url: 'http://vehicle/userdata/recorder/live.mcap',
      indexSource: live.indexSource,
      ongoing: true,
      callbacks: {
        onState: () => undefined,
        onBusy: () => undefined,
        onSummary: () => undefined,
        onMp4Saved: () => undefined,
      },
    })
    await controller.mount()
    expect(controller.getState().playing).toBe(false)

    live.writtenChunks = chunks.length
    controller.onWrittenSizeBytes(live.bytes().length)

    await vi.waitFor(() => expect(controller.getState().tracks).toHaveLength(1))
    const lastCameraFrame = 3.9
    expect(controller.getState().position).toBeCloseTo(lastCameraFrame)
    expect(controller.getState().playing).toBe(true)
    controller.destroy()
  })
})

describe('timeline lanes', () => {
  it('draws the range of each visible stream on its own lane, not one merged bar', async () => {
    const { tracks, durationSeconds } = await openTwoTracks()

    const lanes = trackTimelineLanes(tracks, durationSeconds)

    expect(lanes.map((lane) => lane.channelId)).toEqual(tracks.map((track) => track.channelId))
    const [cameraA, cameraB] = lanes.map((lane) => lane.ranges.map(({ style }) => ({
      left: parseFloat(style.left), width: parseFloat(style.width),
    })))
    expect(cameraA).toHaveLength(1)
    expect(cameraB).toHaveLength(1)
    expect(cameraA[0].left).toBe(0)
    expect(cameraA[0].width).toBeCloseTo(100, 5)
    expect(cameraB[0].left).toBeGreaterThan(0)
    expect(cameraB[0].left + cameraB[0].width).toBeLessThan(100)
  })

  it('keeps the merged union for playback', async () => {
    const { tracks } = await openTwoTracks()

    expect(mergedVideoCoverage(tracks)).toEqual(tracks[0].coverage)
  })
})
