import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import { AnnexBMsePlayer, type AnnexBMseStats } from '@/libs/mcap/adapters/annexb-player'

import { SAMPLE_H264_DELTA, SAMPLE_H264_KEYFRAME } from './build-mcap'
import { type FakeMedia, FakeVideo, installFakeMedia } from './fake-media'

const FRAME_SECONDS = 1 / 30

let fakeMedia: FakeMedia

function newPlayer(options: ConstructorParameters<typeof AnnexBMsePlayer>[1] = {}): {
  player: AnnexBMsePlayer
  video: FakeVideo
} {
  const video = new FakeVideo()
  return { player: new AnnexBMsePlayer(video as unknown as HTMLVideoElement, options), video }
}

function appendedCount(): number {
  return fakeMedia.sourceBuffers[0]?.appendedStarts.length ?? 0
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
    fakeMedia = installFakeMedia({ frameSeconds: FRAME_SECONDS })
    vi.stubGlobal('window', globalThis)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('refuses to start in a browser without Media Source Extensions', () => {
    vi.stubGlobal('window', {})

    expect(() => newPlayer()).toThrow('Media Source Extensions')
  })

  it('plays the stream muted from the object URL of its media source', () => {
    const { video } = newPlayer()

    expect(video.src).toBe([...fakeMedia.mediaSources.keys()][0])
    expect(video.muted).toBe(true)
  })

  it('waits for a keyframe before it is ready, and reports the codec and size it found', async () => {
    const onReady = vi.fn()
    const onStats = vi.fn<[AnnexBMseStats], void>()
    const { player } = newPlayer({ onReady, onStats })

    player.push(SAMPLE_H264_DELTA, 'h264')
    await new Promise((resolve) => { setTimeout(resolve, 20) })
    expect(onReady).not.toHaveBeenCalled()
    expect(fakeMedia.sourceBuffers).toEqual([])

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
    await vi.waitFor(() => expect(fakeMedia.sourceBuffers[0]?.appendedStarts).toHaveLength(3))

    const [first, second, third] = fakeMedia.sourceBuffers[0]?.appendedStarts ?? []
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

    await vi.waitFor(() => expect(fakeMedia.sourceBuffers[0]?.appendedStarts).toHaveLength(6))
    await new Promise((resolve) => { setTimeout(resolve, 50) })
    expect(fakeMedia.sourceBuffers[0]?.appendedStarts).toHaveLength(6)
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

    expect(fakeMedia.sourceBuffers[0]?.removed).toEqual([])
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

    await vi.waitFor(() => expect(fakeMedia.sourceBuffers[0]?.removed).toHaveLength(1))
    expect(fakeMedia.sourceBuffers[0]?.removed[0][0]).toBe(0)
    expect(fakeMedia.sourceBuffers[0]?.removed[0][1]).toBeCloseTo(3, 1)
    // The muxer rounds a frame time down to its timescale, so a range that ends at the keyframe time reaches into it.
    expect(fakeMedia.sourceBuffers[0]?.removed[0][1]).toBeLessThan(fakeMedia.sourceBuffers[0]?.appendedStarts[90] ?? 0)
    player.destroy()
  })

  it('reports the codec as unsupported when the browser cannot play it', async () => {
    fakeMedia.typeSupported = false
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

    const [objectUrl] = fakeMedia.mediaSources.keys()
    // Destroyed before the source opens, the player's wait for `sourceopen` rejects with nothing listening: a product
    // bug the shared fake exposes, as the browser opens the source from a later task.
    await vi.waitFor(() => expect(fakeMedia.mediaSources.get(objectUrl)?.readyState).toBe('open'))
    player.destroy()
    player.push(SAMPLE_H264_KEYFRAME, 'h264')
    await new Promise((resolve) => { setTimeout(resolve, 20) })

    expect(video.src).toBe('')
    expect(URL.revokeObjectURL).toHaveBeenCalledWith(objectUrl)
    expect(onReady).not.toHaveBeenCalled()
  })
})
