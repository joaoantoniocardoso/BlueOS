import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import { McapVideoPlayer } from '@/libs/mcap/adapters/player'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'
import { listVideoTracks } from '@/libs/mcap/logic/video-track'

import { buildGopVideoMcap, buildIndexedVideoMcap, buildTwoTrackVideoMcap } from './build-mcap'
import { type FakeMedia, FakeVideo, installFakeMedia } from './fake-media'
import MemoryByteSource from './memory-byte-source'

/** A browser decodes appended media asynchronously; the player must not count on it being instant. */
const APPEND_MS = 20

/** Frame spacing of `buildIndexedVideoMcap`; the fake buffers one frame past each fragment start. */
const FRAME_SECONDS = 0.033

describe('McapVideoPlayer', () => {
  let fakeMedia: FakeMedia

  beforeEach(() => {
    fakeMedia = installFakeMedia({ frameSeconds: FRAME_SECONDS, appendMilliseconds: APPEND_MS })
  })

  afterEach(() => {
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
    fakeMedia.holdSourceOpen = true
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
    fakeMedia.openHeldSources()
    await started

    const lastCameraBFrame = 15.25
    await vi.waitFor(() => {
      expect(onError.mock.calls.length > 0 || video.buffered.end(0) > lastCameraBFrame).toBe(true)
    }, { timeout: 3000 })
    expect(onError).not.toHaveBeenCalled()
    const appended = fakeMedia.sourceBuffers[0]?.appendedStarts ?? []
    expect(appended).toEqual([...new Set(appended)].sort((left, right) => left - right))
    expect(appended[0]).toBeGreaterThan(startSeconds + 1)
    expect(appended[0]).toBeLessThanOrEqual(target)
    expect(video.currentTime).toBeGreaterThanOrEqual(appended[0])
    expect(video.currentTime).toBeLessThanOrEqual(target)
    player.destroy()
  })

  it('starts from the keyframe at or before its start time and plays from that time', async () => {
    const reader = await McapIndexedReader.open(new MemoryByteSource(await buildGopVideoMcap()))
    const [track] = listVideoTracks(reader)
    const { startTime, endTime } = reader.summary
    const recording = {
      reader, tracks: [track], channels: [], durationSeconds: Number(endTime - startTime) / 1e9, startTime,
    }
    const startSeconds = 15
    const video = new FakeVideo()
    const player = new McapVideoPlayer(video as unknown as HTMLVideoElement, recording, track, { startSeconds })

    await player.start()

    await vi.waitFor(() => expect(video.buffered.end(0)).toBeGreaterThan(startSeconds))
    // Keyframes come every 2 s, and the chunk holding 15 s starts at 8.5 s, so its first keyframe is at 10 s.
    const lastKeyframeBefore = 14
    expect(fakeMedia.sourceBuffers[0]?.appendedStarts[0]).toBe(lastKeyframeBefore)
    expect(video.currentTime).toBe(startSeconds)
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
    const video = new FakeVideo()
    video.currentTime = 14
    const player = new McapVideoPlayer(video as unknown as HTMLVideoElement, recording, cameraB, {
      startSeconds,
      bufferAheadSeconds: 2,
    })

    await player.start()

    function latestAppendedStart(): number {
      return Math.max(...fakeMedia.sourceBuffers[0]?.appendedStarts ?? [])
    }
    await vi.waitFor(() => expect(latestAppendedStart()).toBeGreaterThan(startSeconds + 1))
    expect(latestAppendedStart()).toBeLessThan(startSeconds + 4)
    expect(video.currentTime).toBeGreaterThan(startSeconds - 1)
    expect(video.currentTime).toBeLessThan(startSeconds + 1)
    player.destroy()
  })
})
