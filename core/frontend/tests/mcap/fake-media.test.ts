import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import {
  FakeMediaSource, FakeSourceBuffer, FakeVideo, HAVE_ENOUGH_DATA, HAVE_METADATA, HAVE_NOTHING, installFakeMedia,
} from './fake-media'

const TIMESCALE = 1000

/** An ISO BMFF box of this type holding the bytes, with the size field a parser would skip. */
function box(type: string, payload: number[]): Uint8Array {
  const data = new Uint8Array(8 + payload.length)
  new DataView(data.buffer).setUint32(0, data.length)
  data.set([...type].map((character) => character.charCodeAt(0)), 4)
  data.set(payload, 8)
  return data
}

function uint32(value: number): number[] {
  const view = new DataView(new ArrayBuffer(4))
  view.setUint32(0, value)
  return [...new Uint8Array(view.buffer)]
}

/** The init segment: the track timescale is all the fake reads of it. */
function initSegment(): Uint8Array {
  return box('mdhd', [0, 0, 0, 0, ...uint32(0), ...uint32(0), ...uint32(TIMESCALE)])
}

/** A fragment whose first frame decodes at this time, which is all the fake reads of it. */
function fragmentAt(seconds: number): Uint8Array {
  return box('tfdt', [0, 0, 0, 0, ...uint32(seconds * TIMESCALE)])
}

function settle(): Promise<void> {
  return new Promise((resolve) => { setTimeout(resolve, 5) })
}

function record(video: FakeVideo, ...types: string[]): string[] {
  const events: string[] = []
  types.forEach((type) => video.addEventListener(type, () => events.push(type)))
  return events
}

async function append(buffer: FakeSourceBuffer, data: Uint8Array): Promise<void> {
  const updated = new Promise((resolve) => { buffer.addEventListener('updateend', resolve, { once: true }) })
  buffer.appendBuffer(data)
  await updated
}

/** A video attached to a source buffer that holds no data yet. */
async function attachedVideo(): Promise<{ video: FakeVideo, source: FakeMediaSource, buffer: FakeSourceBuffer }> {
  const source = new MediaSource() as unknown as FakeMediaSource
  const video = new FakeVideo()
  const opened = new Promise((resolve) => { source.addEventListener('sourceopen', resolve, { once: true }) })
  video.src = URL.createObjectURL(source as unknown as MediaSource)
  await opened
  return { video, source, buffer: source.addSourceBuffer() }
}

describe('the fake media element', () => {
  beforeEach(() => {
    installFakeMedia({ frameSeconds: 1 })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('opens a media source from a later task once an element attaches it', async () => {
    const source = new MediaSource() as unknown as FakeMediaSource
    const video = new FakeVideo()
    expect(source.readyState).toBe('closed')

    video.src = URL.createObjectURL(source as unknown as MediaSource)
    expect(source.readyState).toBe('closed')

    await settle()
    expect(source.readyState).toBe('open')
  })

  it('fires seeking for a time set, and seeked only once data covers it', async () => {
    const { video, buffer } = await attachedVideo()
    await append(buffer, initSegment())
    await append(buffer, fragmentAt(0))
    const events = record(video, 'seeking', 'seeked')

    video.currentTime = 0.5
    expect(video.currentTime).toBe(0.5)
    expect(video.seeking).toBe(true)
    expect(events).toEqual([])
    await settle()
    expect(events).toEqual(['seeking', 'seeked'])
    expect(video.seeking).toBe(false)

    events.length = 0
    video.currentTime = 5
    await settle()
    expect(events).toEqual(['seeking'])
    expect(video.seeking).toBe(true)

    await append(buffer, fragmentAt(5))
    await settle()
    expect(events).toEqual(['seeking', 'seeked'])
  })

  it('reports the reset to 0 that load() makes with emptied and timeupdate', async () => {
    const { video, buffer } = await attachedVideo()
    await append(buffer, initSegment())
    await append(buffer, fragmentAt(0))
    video.currentTime = 0.5
    await settle()
    const events = record(video, 'emptied', 'timeupdate')

    video.removeAttribute()
    video.load()
    expect(video.currentTime).toBe(0)
    expect(video.readyState).toBe(HAVE_NOTHING)
    expect(video.buffered.length).toBe(0)
    await settle()
    expect(events).toEqual(['emptied', 'timeupdate'])

    events.length = 0
    video.load()
    await settle()
    expect(events).toEqual([])
  })

  it('keeps a time set before the metadata and seeks to it once the metadata loads', async () => {
    const { video, buffer } = await attachedVideo()
    const events = record(video, 'seeking', 'loadedmetadata')

    video.currentTime = 4
    expect(video.currentTime).toBe(4)
    expect(video.readyState).toBe(HAVE_NOTHING)
    await settle()
    expect(events).toEqual([])

    await append(buffer, initSegment())
    expect(video.readyState).toBe(HAVE_METADATA)
    await settle()
    expect(events).toEqual(['loadedmetadata', 'seeking'])
    expect(video.currentTime).toBe(4)

    await append(buffer, fragmentAt(4))
    expect(video.readyState).toBe(HAVE_ENOUGH_DATA)
  })

  it('keeps a time set on an element with no media through a load()', () => {
    const video = new FakeVideo()
    video.currentTime = 14
    video.load()
    expect(video.currentTime).toBe(14)
  })

  it('pauses with timeupdate before pause, and plays into waiting where nothing is buffered', async () => {
    const { video, buffer } = await attachedVideo()
    await append(buffer, initSegment())
    const events = record(video, 'play', 'playing', 'waiting', 'timeupdate', 'pause')

    await video.play()
    video.advance()
    video.advance()
    await settle()
    expect(events).toEqual(['play', 'waiting'])

    await append(buffer, fragmentAt(0))
    events.length = 0
    video.advance()
    video.pause()
    await settle()
    expect(video.currentTime).toBeGreaterThan(0)
    expect(events).toEqual(['timeupdate', 'timeupdate', 'pause'])
  })

  it('refuses an append while the source buffer is still updating, and removes only the range asked', async () => {
    const { video, buffer } = await attachedVideo()
    await append(buffer, initSegment())
    await append(buffer, fragmentAt(0))
    await append(buffer, fragmentAt(2))
    expect(video.buffered.end(0)).toBe(3)

    buffer.appendBuffer(fragmentAt(3))
    expect(() => buffer.appendBuffer(fragmentAt(4))).toThrow(/still processing/)
    await settle()

    const removed = new Promise((resolve) => { buffer.addEventListener('updateend', resolve, { once: true }) })
    buffer.remove(0, 1)
    await removed
    expect(video.buffered.start(0)).toBe(2)
    expect(buffer.appendedStarts).toEqual([0, 2, 3])
  })
})
