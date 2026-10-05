/**
 * Writes a video stream of an MCAP recording, or a part of one, out as an MP4 file in the browser.
 *
 * The frames stored in the recording are already H264 or H265, so saving is a matter of wrapping them
 * in a container: nothing is decoded or encoded, and the picture is exactly what was recorded. The
 * file is not fragmented, so players and editors that only understand ordinary MP4 read it too.
 *
 * Saving a range is what keeps the transfer down. Frames live inside compressed MCAP chunks that also
 * carry every other topic recorded at the same time, so a chunk can only be read whole: saving a
 * minute of it only downloads the chunks that minute falls in.
 */
import { throwIfAborted } from '../logic/abort'
import type { Mp4ExportRange } from '../logic/csv'
import VideoFrameStream from '../logic/frame-stream'
import { DecodableFrameCursor } from '../logic/frames'
import {
  AnnexBFrame, clampSampleDuration, Mp4FileStream,
} from '../logic/mux'
import { createProgressGate } from '../logic/progress'
import { VideoTrack } from '../logic/video-track'
import { McapVideoRecording } from './player'

export type { Mp4ExportRange }

export interface Mp4ExportProgress {
  seconds: number
  durationSeconds: number
  bytes: number
}

export interface Mp4ExportOptions {
  range?: Mp4ExportRange
  onProgress?: (progress: Mp4ExportProgress) => void
  signal?: AbortSignal
}

/** The stream, or the part of it asked for, holds no keyframe, so there is nothing that can be saved. */
export class NoKeyframeError extends Error {
  constructor(ranged: boolean) {
    super(ranged
      ? 'The selected part of this video stream holds no keyframe, so there is nothing that can be saved.'
      : 'This video stream holds no keyframe, so there is nothing that can be saved.')
    this.name = 'NoKeyframeError'
  }
}

interface HeldSample {
  data: Uint8Array
  logTime: bigint
  isKeyframe: boolean
}

/** Reads a video stream, or the requested range of it, and returns it as an MP4 file ready to save. */
export async function exportTrackAsMp4(
  recording: McapVideoRecording,
  track: VideoTrack,
  options: Mp4ExportOptions = {},
): Promise<Blob> {
  const { onProgress, signal, range } = options
  const stream = new VideoFrameStream(recording.reader, track)
  const startSeconds = Math.max(0, range?.startSeconds ?? 0)
  const endSeconds = Math.min(range?.endSeconds ?? Infinity, stream.durationSeconds)
  if (startSeconds > 0) {
    await stream.seekToKeyframe(startSeconds, signal)
  } else {
    stream.seekToStart()
  }
  const endLogTime = range && Number.isFinite(range.endSeconds) ? stream.toLogTime(range.endSeconds) : null

  const cursor = new DecodableFrameCursor(stream, recording.reader, track, false)
  // Assigned inside closures, which narrowing cannot see, so the declared null must not narrow the type.
  let writer = null as Mp4FileStream | null
  let held = null as HeldSample | null
  // The frames before the start from the last keyframe there on: the least that decodes the start.
  let preRoll: HeldSample[] = []
  let reachedStart = false
  let duration = 1 / 30
  let bytes = 0
  const shouldReport = createProgressGate()

  async function emit(sample: HeldSample, sampleDuration: number): Promise<void> {
    const frame: AnnexBFrame = {
      data: sample.data,
      timestamp: Math.max(0, stream.toSeconds(sample.logTime) - startSeconds),
      duration: sampleDuration,
      isKeyframe: sample.isKeyframe,
    }
    if (!writer && cursor.decoderInfo) {
      writer = await Mp4FileStream.open(cursor.decoderInfo, (written) => {
        bytes = written
      })
    }
    await writer?.add(frame)
    if (shouldReport()) {
      onProgress?.({
        seconds: Math.max(0, stream.toSeconds(sample.logTime) - startSeconds),
        durationSeconds: Math.max(endSeconds - startSeconds, 0),
        bytes,
      })
    }
  }

  async function hold(sample: HeldSample): Promise<void> {
    if (held) {
      duration = clampSampleDuration(Number(sample.logTime - held.logTime) / 1e9)
      await emit(held, duration)
    }
    held = sample
  }

  for (;;) {
    throwIfAborted(signal, 'The export was cancelled.')
    // eslint-disable-next-line no-await-in-loop
    const frame = await cursor.next(signal)
    if (!frame) {
      break
    }
    const sample = { data: frame.annexB, logTime: frame.logTime, isKeyframe: frame.isKeyframe }
    if (!reachedStart && stream.toSeconds(frame.logTime) < startSeconds) {
      if (frame.isKeyframe) {
        preRoll = []
      }
      preRoll.push(sample)
      continue
    }
    if (endLogTime !== null && frame.logTime > endLogTime) {
      if (held) {
        // eslint-disable-next-line no-await-in-loop
        await emit(held, clampSampleDuration(Number(frame.logTime - held.logTime) / 1e9))
        held = null
      }
      break
    }
    // A stream with no frame inside the cut never gets here, so nothing of it is written.
    if (!reachedStart) {
      reachedStart = true
      for (const preRollSample of preRoll) {
        // eslint-disable-next-line no-await-in-loop
        await hold(preRollSample)
      }
    }
    // eslint-disable-next-line no-await-in-loop
    await hold(sample)
  }

  if (!cursor.decoderInfo || !held && !writer) {
    throw new NoKeyframeError(range !== undefined)
  }
  if (held) {
    await emit(held, duration)
  }
  if (!writer) {
    throw new NoKeyframeError(range !== undefined)
  }
  return writer.finalize()
}
