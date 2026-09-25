import type { Mp4ExportRange } from '../adapters/export'
import type { PrefixScanProgress } from './reader'
import type { TimeRange, VideoTrack } from './video-track'
import { mergeTimeRanges, timeRangesCover } from './video-track'

export interface Mp4ExportProgressView {
  seconds: number
  durationSeconds: number
  bytes: number
}

/** How close to the last recorded frame counts as watching the latest written frames. */
export const LATEST_EDGE_SECONDS = 2
/** How finely the picker cuts. Bounds land on multiples of it, so the end of a recording lies within. */
export const CLIP_STEP_SECONDS = 0.1
/** Room the range slider leaves on each side for its thumbs, which the playhead marker has to match. */
export const SLIDER_THUMB_ROOM = '8px'
/** Cap on grid columns so 64 streams stay readable on a desktop layout. */
export const MAX_GRID_COLUMNS = 8

export function coverageEnd(tracks: VideoTrack[]): number {
  let latest = 0
  for (const track of tracks) {
    for (const range of track.coverage) {
      latest = Math.max(latest, range.end)
    }
  }
  return latest
}

export function coverageStart(tracks: VideoTrack[]): number {
  let earliest = Number.POSITIVE_INFINITY
  for (const track of tracks) {
    for (const range of track.coverage) {
      earliest = Math.min(earliest, range.start)
    }
  }
  return Number.isFinite(earliest) ? earliest : 0
}

export function formatPlaybackPosition(seconds: number): string {
  const total = Math.max(0, Math.round(seconds))
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor(total % 3600 / 60)
  const secs = total % 60
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  }
  return `${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
}

export function formatFrameAge(seconds: number): string {
  const whole = Math.max(0, Math.round(seconds))
  if (whole === 0) {
    return 'Frame from less than a second ago'
  }
  return whole === 1 ? 'Frame from 1 second ago' : `Frame from ${whole} seconds ago`
}

export function filterTracksBySearch(tracks: VideoTrack[], query: string): VideoTrack[] {
  const normalized = query.trim().toLowerCase()
  if (!normalized) {
    return tracks
  }
  return tracks.filter((track) => track.name.toLowerCase().includes(normalized)
    || track.topic.toLowerCase().includes(normalized))
}

export function visibleTracks(tracks: VideoTrack[], selectedChannelIds: number[]): VideoTrack[] {
  const selected = new Set(selectedChannelIds)
  return tracks.filter((track) => selected.has(track.channelId))
}

export function gridColumnCount(visibleCount: number, smallScreen: boolean): number {
  if (visibleCount <= 1 || smallScreen) {
    return 1
  }
  return Math.min(MAX_GRID_COLUMNS, Math.ceil(Math.sqrt(visibleCount)))
}

export function gridRowCount(visibleCount: number, columns: number): number {
  return Math.max(1, Math.ceil(visibleCount / columns))
}

export function gridStyle(columns: number, rows: number): Record<string, string> {
  return {
    '--grid-columns': String(columns),
    '--grid-rows': String(rows),
  }
}

export function clipExportRange(
  cutEnabled: boolean,
  clipRange: [number, number],
  durationSeconds: number,
): Mp4ExportRange | null {
  if (!cutEnabled) {
    return null
  }
  const [start, end] = clipRange
  const toTheEnd = end >= durationSeconds - CLIP_STEP_SECONDS
  const whole = start <= 0 && toTheEnd
  if (whole) {
    return null
  }
  return { startSeconds: start, endSeconds: toTheEnd ? Infinity : end }
}

export function clipDurationLabel(clipRange: [number, number]): string {
  const [start, end] = clipRange
  return formatPlaybackPosition(Math.max(0, Math.round(end) - Math.round(start)))
}

export function mp4SaveLabel(
  trackCount: number,
  clip: Mp4ExportRange | null,
  clipRange: [number, number],
): string {
  if (!clip) {
    return trackCount > 1
      ? `Save ${trackCount} whole streams as MP4`
      : 'Save the whole stream as MP4'
  }
  return `Save ${formatPlaybackPosition(clipRange[0])} – ${formatPlaybackPosition(clipRange[1])} as MP4`
}

export function exportPercentage(
  progress: Mp4ExportProgressView | null,
  trackIndex: number,
  trackCount: number,
): number {
  const { seconds, durationSeconds } = progress ?? { seconds: 0, durationSeconds: 0 }
  const current = durationSeconds > 0 ? Math.min(1, seconds / durationSeconds) : 0
  if (trackCount <= 1) {
    return Math.round(current * 100)
  }
  const done = trackIndex + current
  return Math.min(100, Math.round(done / trackCount * 100))
}

export function playheadStyle(position: number, durationSeconds: number): Record<string, string> {
  const fraction = durationSeconds > 0 ? Math.min(position / durationSeconds, 1) : 0
  return { left: `calc(${SLIDER_THUMB_ROOM} + (100% - 2 * ${SLIDER_THUMB_ROOM}) * ${fraction})` }
}

export function openingMessage(ongoing: boolean): string {
  return ongoing
    ? 'This is an on-going recording. Reading what has been written so far...'
    : 'Reading recording index...'
}

export function openingStatusText(
  progress: PrefixScanProgress | null,
  bytesPerSecond: number,
  formatKilobytes: (kilobytes: number) => string,
): string {
  if (!progress) {
    return ''
  }
  const indexed = formatKilobytes(progress.offset / 1024)
  const total = formatKilobytes(progress.size / 1024)
  const chunks = progress.chunks === 1 ? '1 chunk' : `${progress.chunks} chunks`
  const speed = bytesPerSecond > 0 ? ` · ${formatKilobytes(bytesPerSecond / 1024)}/s` : ''
  return `${indexed} of ${total} · ${chunks}${speed}`
}

export function openingPercent(progress: PrefixScanProgress | null): number | null {
  if (!progress || progress.size <= 0) {
    return null
  }
  return Math.min(100, progress.offset / progress.size * 100)
}

export function namingStatusText(
  progress: { named: number, total: number, bytes: number, bytesPerSecond: number } | null,
  formatKilobytes: (kilobytes: number) => string,
): string {
  if (!progress) {
    return ''
  }
  const speed = progress.bytesPerSecond > 0 ? ` · ${formatKilobytes(progress.bytesPerSecond / 1024)}/s` : ''
  return `${progress.named} of ${progress.total} topics · ${formatKilobytes(progress.bytes / 1024)}${speed}`
}

export function namingPercent(progress: { named: number, total: number } | null): number | null {
  if (!progress || progress.total <= 0) {
    return null
  }
  return Math.min(100, progress.named / progress.total * 100)
}

export function recordingNameFromUrl(url: string): string {
  return decodeURIComponent(url.split('/').pop() ?? 'recording').replace(/\.mcap$/, '')
}

export function atLatestPosition(
  ongoing: boolean,
  lastVideoTime: number,
  position: number,
): boolean {
  return ongoing && lastVideoTime - position <= LATEST_EDGE_SECONDS
}

export function timelinePercent(seconds: number | null, durationSeconds: number): string {
  if (seconds === null || durationSeconds <= 0) {
    return '0%'
  }
  return `${Math.min(100, seconds / durationSeconds * 100)}%`
}

export function timelineRangeStyles(
  ranges: TimeRange[],
  durationSeconds: number,
): { style: { left: string, width: string } }[] {
  if (durationSeconds <= 0) {
    return []
  }
  return ranges.map((range) => ({
    style: {
      left: `${range.start / durationSeconds * 100}%`,
      width: `${Math.max(0, range.end - range.start) / durationSeconds * 100}%`,
    },
  }))
}

export function bufferedRangeStyles(
  bufferedRanges: { start: number, end: number }[],
  durationSeconds: number,
): { left: string, width: string }[] {
  if (durationSeconds <= 0) {
    return []
  }
  return bufferedRanges.map((range) => ({
    left: `${range.start / durationSeconds * 100}%`,
    width: `${Math.max(0, range.end - range.start) / durationSeconds * 100}%`,
  }))
}

export function mergedVideoCoverage(tracks: VideoTrack[]): TimeRange[] {
  return mergeTimeRanges(tracks.flatMap((track) => track.coverage))
}

export function trackCoversAt(track: VideoTrack, seconds: number): boolean {
  return timeRangesCover(track.coverage, seconds)
}

export function playheadHasBufferedMedia(video: HTMLVideoElement): boolean {
  const { buffered, currentTime } = video
  for (let index = 0; index < buffered.length; index += 1) {
    if (currentTime >= buffered.start(index) && currentTime <= buffered.end(index)) {
      return true
    }
  }
  return false
}

export function secondsFromTimelineFraction(fraction: number, durationSeconds: number): number {
  return Math.max(0, Math.min(1, fraction)) * durationSeconds
}

export function toggleSelectedChannel(
  selectedChannelIds: number[],
  tracks: VideoTrack[],
  channelId: number,
): number[] {
  if (selectedChannelIds.includes(channelId)) {
    return selectedChannelIds.filter((selected) => selected !== channelId)
  }
  return tracks
    .map((track) => track.channelId)
    .filter((selected) => selected === channelId || selectedChannelIds.includes(selected))
}

export function mp4FileName(
  recordingName: string,
  track: VideoTrack,
  clip: Mp4ExportRange | null,
): string {
  const base = `${recordingName}-${track.name}`
  if (!clip) {
    return `${base}.mp4`
  }
  const end = Number.isFinite(clip.endSeconds) ? `${Math.round(clip.endSeconds)}s` : 'end'
  return `${base}-${Math.round(clip.startSeconds)}s-${end}.mp4`
}

export function markClipStart(
  clipRange: [number, number],
  position: number,
  durationSeconds: number,
): [number, number] {
  const [, end] = clipRange
  return [position, end > position ? end : durationSeconds]
}

export function markClipEnd(clipRange: [number, number], position: number): [number, number] {
  const [start] = clipRange
  return [start < position ? start : 0, position]
}

export function rangeInputMovedBound(
  previousRange: [number, number],
  nextRange: [number, number],
): number {
  const [previousStart, previousEnd] = previousRange
  const [nextStart, nextEnd] = nextRange
  return Math.abs(nextStart - previousStart) >= Math.abs(nextEnd - previousEnd) ? 0 : 1
}
