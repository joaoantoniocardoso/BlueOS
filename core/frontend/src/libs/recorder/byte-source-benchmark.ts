import { openMcapVideoRecording } from '@/libs/mcap/adapters/player'
import { extractMcapThumbnail } from '@/libs/mcap/adapters/thumbnail'
import VideoFrameStream from '@/libs/mcap/logic/frame-stream'
import { DecodableFrameCursor } from '@/libs/mcap/logic/frames'

import { DEFAULT_RECORDING_HTTP_PREFIX } from './constants'
import { RECORDS_BYTE_SOURCE_STORAGE_KEY, type RecordsByteSource } from './preferences'
import { recordingDownloadUrl } from './url'

const MEBIBYTE = 1024 * 1024

/** What reading one recording through one byte source cost, as the Records page reads it. */
export interface ByteSourceBenchmark {
  source: RecordsByteSource
  path: string
  /** From opening the reader (size, footer, summary, chunk index) to the first video frame read. */
  openToFirstFrameMs: number
  /** Bytes the player's frame reader fetched after the first frame, in its own range sizes, and how long it took. */
  sequentialBytes: number
  sequentialMs: number
  sequentialMebibytesPerSecond: number
  /** One thumbnail, as a card builds it: open, seek to the middle, one keyframe. */
  thumbnailMs: number
}

/**
 * Times the Records page reading the finished recording at `path` through `source`: opening the player, reading
 * `mebibytes` of video frames in order, and one thumbnail. The stored byte source is restored afterwards.
 */
export async function benchmarkRecordingByteSource(
  path: string,
  source: RecordsByteSource,
  mebibytes = 64,
): Promise<ByteSourceBenchmark> {
  const storage = window.localStorage
  const stored = storage.getItem(RECORDS_BYTE_SOURCE_STORAGE_KEY)
  storage.setItem(RECORDS_BYTE_SOURCE_STORAGE_KEY, source)
  try {
    const url = recordingDownloadUrl(path, DEFAULT_RECORDING_HTTP_PREFIX)
    const opening = performance.now()
    const recording = await openMcapVideoRecording(url)
    const track = [...recording.tracks].sort((left, right) => right.frameCount - left.frameCount)[0]
    if (!track) {
      throw new Error('This recording has no video track to read.')
    }
    const cursor = new DecodableFrameCursor(new VideoFrameStream(recording.reader, track), recording.reader, track)
    let frame = await cursor.next()
    const firstFrame = performance.now()
    const bytesBefore = recording.reader.source.bytesRead
    while (frame && recording.reader.source.bytesRead - bytesBefore < mebibytes * MEBIBYTE) {
      // eslint-disable-next-line no-await-in-loop
      frame = await cursor.next()
    }
    const sequentialBytes = recording.reader.source.bytesRead - bytesBefore
    const sequentialMs = performance.now() - firstFrame
    const thumbnailStart = performance.now()
    await extractMcapThumbnail(url)
    return {
      source,
      path,
      openToFirstFrameMs: firstFrame - opening,
      sequentialBytes,
      sequentialMs,
      sequentialMebibytesPerSecond: sequentialBytes / MEBIBYTE / (sequentialMs / 1000),
      thumbnailMs: performance.now() - thumbnailStart,
    }
  } finally {
    if (stored === null) {
      storage.removeItem(RECORDS_BYTE_SOURCE_STORAGE_KEY)
    } else {
      storage.setItem(RECORDS_BYTE_SOURCE_STORAGE_KEY, stored)
    }
  }
}

/**
 * Puts `blueosRecordsBenchmark(path, source, mebibytes)` on `window` in development builds, or once a byte source
 * was stored, so it can be run from the browser console on the vehicle.
 */
export function exposeByteSourceBenchmark(): void {
  let stored: string | null = null
  try {
    stored = window.localStorage.getItem(RECORDS_BYTE_SOURCE_STORAGE_KEY)
  } catch {
    // The browser refuses storage, so no byte source can be chosen either.
  }
  if (import.meta.env.DEV || stored !== null) {
    Object.assign(window, { blueosRecordsBenchmark: benchmarkRecordingByteSource })
  }
}
