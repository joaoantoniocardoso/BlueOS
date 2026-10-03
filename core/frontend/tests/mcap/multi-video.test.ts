import {
  afterEach, describe, expect, it, vi,
} from 'vitest'

import { exportTrackAsMp4 } from '@/libs/mcap/adapters/export'
import {
  McapRecordingPlaybackController,
  type StreamPlaybackControl,
} from '@/libs/mcap/adapters/mcap-recording-playback-controller'
import type { McapVideoRecording } from '@/libs/mcap/adapters/player'
import VideoFrameStream from '@/libs/mcap/logic/frame-stream'
import { mergedVideoCoverage, trackTimelineLanes } from '@/libs/mcap/logic/playback-ui'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'
import { listVideoTracks } from '@/libs/mcap/logic/video-track'

import {
  asLiveRecording, buildLateVideoMcap, buildTwoTrackVideoMcap, TWO_TRACK_FRAME_NS,
} from './build-mcap'
import MemoryByteSource from './memory-byte-source'

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
