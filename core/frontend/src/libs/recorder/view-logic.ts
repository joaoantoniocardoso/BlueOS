import { REPAIR_BYTES_PER_SECOND_ESTIMATE } from './constants'
import type {
  LibraryRecording,
  RecordingOperationEvent,
  RecordingState,
  RepairProgress,
} from './types'

export function utcCalendarDay(timestampSeconds: number): string {
  const date = new Date(timestampSeconds * 1000)
  const year = date.getUTCFullYear()
  const month = String(date.getUTCMonth() + 1).padStart(2, '0')
  const day = String(date.getUTCDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

export function formatUtcDayLabel(day: string): string {
  const [year, month, date] = day.split('-')
  return `${year}-${month}-${date} UTC`
}

export function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds))
  const days = Math.floor(total / 86400)
  const hours = Math.floor(total % 86400 / 3600)
  const minutes = Math.floor(total % 3600 / 60)
  const rest = total % 60
  const paddedMinutes = String(minutes).padStart(2, '0')
  const paddedSeconds = String(rest).padStart(2, '0')
  if (days > 0) {
    return `${days}d ${hours}h ${paddedMinutes}m`
  }
  if (hours > 0) {
    return `${hours}h ${paddedMinutes}m ${paddedSeconds}s`
  }
  return `${minutes}m ${paddedSeconds}s`
}

export function formatDateFromUnixSeconds(timestampSeconds: number): string {
  return new Date(timestampSeconds * 1000).toLocaleString()
}

export function recordingTitle(file: LibraryRecording): string {
  return file.name.replace(/\.mcap$/i, '')
}

export function stateChipColor(state: RecordingState): string {
  const colors: Record<RecordingState, string> = {
    recording: 'warning',
    needs_repair: 'error',
    repairing: 'primary',
    ready: 'success',
  }
  return colors[state]
}

export function stateChipLabel(
  file: LibraryRecording,
  repairProgress: RepairProgress | null,
): string {
  const labels: Record<RecordingState, string> = {
    recording: 'Recording',
    needs_repair: 'Needs repair',
    repairing: 'Repairing',
    ready: 'Ready',
  }
  const label = labels[file.state]
  return file.state === 'repairing' && repairProgress
    ? `${label} ${Math.round(repairProgress.percent)}%`
    : label
}

export function repairProgressFromFile(
  file: LibraryRecording,
  formatSize: (bytes: number) => string,
): RepairProgress | null {
  if (file.state !== 'repairing' || file.repair_total_bytes <= 0) {
    return null
  }
  const read = file.repair_bytes_processed
  const speed = file.repair_bytes_per_second
  const readLabel = `${formatSize(read)} of ${formatSize(file.repair_total_bytes)}`
  const left = speed > 0 ? formatDuration((file.repair_total_bytes - read) / speed) : null
  return {
    percent: Math.min(100, read / file.repair_total_bytes * 100),
    label: left === null ? readLabel : `${readLabel} · ${left} left`,
  }
}

/** A missing index is not a missing recording: the vehicle indexes what was written on demand. */
export function canPlayRecording(file: LibraryRecording): boolean {
  return file.state === 'ready' || file.state === 'recording' || file.state === 'needs_repair'
}

export function canDeleteRecording(file: LibraryRecording, isSafe: boolean): boolean {
  return isSafe && file.state !== 'recording' && file.state !== 'repairing'
}

export function canRepairRecording(file: LibraryRecording, isSafe: boolean): boolean {
  return isSafe && file.state === 'needs_repair'
}

export function canDownloadRecording(file: LibraryRecording, isSafe: boolean): boolean {
  if (!isSafe) {
    return false
  }
  return file.state === 'ready' || file.state === 'recording'
}

export function needsSnapshotBeforeDownload(file: LibraryRecording): boolean {
  return file.state === 'recording'
}

export function deleteTooltip(file: LibraryRecording): string {
  if (file.state === 'recording') {
    return 'Cannot delete while the vehicle is still recording'
  }
  if (file.state === 'repairing') {
    return 'Cannot delete while the recording is being repaired'
  }
  return `Delete ${file.name}`
}

export function downloadTooltip(file: LibraryRecording): string {
  if (file.state === 'needs_repair') {
    return 'Repair the recording before downloading'
  }
  if (file.state === 'repairing') {
    return 'Wait until repair finishes before downloading'
  }
  if (file.state === 'recording') {
    return 'Download what has been written so far'
  }
  return `Download ${file.name}`
}

export function repairEstimateMessage(
  targets: LibraryRecording[],
  formatSize: (bytes: number) => string,
): string {
  const bytes = targets.reduce((total, file) => total + file.size_bytes, 0)
  const seconds = bytes / REPAIR_BYTES_PER_SECOND_ESTIMATE
  const estimate = seconds < 60 ? 'less than a minute' : `around ${formatDuration(seconds)}`
  const what = targets.length === 1
    ? targets[0].name
    : `${targets.length} recordings`
  return `Repairing ${what} rewrites ${formatSize(bytes)} on the vehicle and should take ${estimate},`
    + ' keeping its disk and processor busy the whole time. Recording and streaming will be slower while it'
    + ' runs. You can already play this recording without repairing it, and you can stop the repair at any'
    + ' time.'
}

export function libraryNeedsFollowUp(files: LibraryRecording[]): boolean {
  return files.some((file) => file.state === 'recording' || file.state === 'repairing')
}

export function sortRecordingsNewestFirst(files: LibraryRecording[]): LibraryRecording[] {
  return [...files].sort((left, right) => {
    if (right.created !== left.created) {
      return right.created - left.created
    }
    return right.path.localeCompare(left.path)
  })
}

export function isSnapshotOperationForPath(
  event: RecordingOperationEvent,
  path: string,
): boolean {
  return event.operation === 'snapshot' && event.path === path
}

export function snapshotDownloadPath(event: RecordingOperationEvent): string | null {
  if (!event.succeeded || event.cancelled || !event.output_path) {
    return null
  }
  return event.output_path
}

export function readySnapshotDownloadPath(
  sourcePath: string,
  files: LibraryRecording[],
): string | null {
  const segments = sourcePath.split('/')
  const fileName = segments.pop() ?? sourcePath
  const parent = segments.join('/')
  const stem = fileName.replace(/\.mcap$/i, '')
  const prefix = parent ? `${parent}/${stem}.snapshot-` : `${stem}.snapshot-`
  const match = files.find(
    (file) => file.state === 'ready'
      && file.path.startsWith(prefix)
      && file.path.toLowerCase().endsWith('.mcap'),
  )
  return match?.path ?? null
}

export const RECORDS_LEAVE_MESSAGE = 'Keep this page open. Downloads and exports run in this browser'
  + ' and will stop if you leave.'
  + ' Repair on the vehicle continues, but you would lose progress shown here.'

export const RECORDS_ALL_DATES = ''

export const RECORDS_LAYOUT_STORAGE_KEY = 'blueos.records.layout'

export function browsingAllowedWhileArmed(vehicleArmed: boolean): boolean {
  return !vehicleArmed
}

export function deleteConfirmationMessage(targets: LibraryRecording[]): string {
  if (targets.length === 1) {
    return `Delete ${targets[0].name}? This cannot be undone.`
  }
  return `Delete ${targets.length} recordings? This cannot be undone.`
}

export function selectionStatsLabel(
  files: LibraryRecording[],
  totalDurationSeconds: number,
  totalSizeBytes: number,
  formatSize: (bytes: number) => string,
): string {
  return `${formatDuration(totalDurationSeconds)} · ${formatSize(totalSizeBytes)}`
}

export function dateFilterOptions(
  recordings: LibraryRecording[],
): { text: string, value: string }[] {
  const days = new Set(recordings.map((file) => utcCalendarDay(file.created)))
  const sorted = Array.from(days).sort((left, right) => right.localeCompare(left))
  return [
    { text: 'All dates', value: RECORDS_ALL_DATES },
    ...sorted.map((day) => ({ text: formatUtcDayLabel(day), value: day })),
  ]
}

export function filterRecordingsByDate(
  recordings: LibraryRecording[],
  selectedDate: string,
): LibraryRecording[] {
  if (!selectedDate) {
    return recordings
  }
  return recordings.filter((file) => utcCalendarDay(file.created) === selectedDate)
}

export function recordingDurationSeconds(
  file: LibraryRecording,
  summaries: Record<string, { durationSeconds: number }>,
  nowSeconds: number,
): number | null {
  if (file.state === 'recording') {
    return Math.max(0, nowSeconds - file.created)
  }
  const summary = summaries[file.path]
  return summary?.durationSeconds ?? null
}

export function tracksSummaryLabel(
  summary: { tracks: { name: string }[], channels: unknown[] } | undefined,
): string | null {
  if (!summary) {
    return null
  }
  const videoNames = summary.tracks.map((track) => track.name)
  const otherCount = summary.channels.length - summary.tracks.length
  if (videoNames.length === 0 && otherCount === 0) {
    return 'no topics'
  }
  const parts: string[] = []
  if (videoNames.length > 0) {
    parts.push(videoNames.join(', '))
  }
  if (otherCount > 0) {
    parts.push(`${otherCount} other topic${otherCount === 1 ? '' : 's'}`)
  }
  return parts.join(' · ')
}

export function activeRecordMetaLabel(
  file: LibraryRecording,
  summary: { tracks: unknown[] } | undefined,
  durationSeconds: number | null,
  formatSize: (bytes: number) => string,
): string | null {
  const parts: string[] = []
  if (durationSeconds !== null) {
    parts.push(formatDuration(durationSeconds))
  }
  if (summary && summary.tracks.length > 0) {
    parts.push(`${summary.tracks.length} stream${summary.tracks.length === 1 ? '' : 's'}`)
  }
  parts.push(formatSize(file.size_bytes))
  return parts.join(' · ')
}

export function operationFailureMessage(
  event: RecordingOperationEvent,
  fileName: string,
): string | null {
  if (event.succeeded || event.cancelled) {
    return null
  }
  let verb = 'Delete'
  if (event.operation === 'repair') {
    verb = 'Repair'
  } else if (event.operation === 'snapshot') {
    verb = 'Snapshot'
  }
  return `${verb} failed for ${fileName}: ${event.error || 'unknown error'}`
}
