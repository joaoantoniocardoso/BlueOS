import { JobStatusStatus } from '@blueos-idl'
import type { JobStatus } from '@blueos-idl/messages'

import type { RepairRecordingFeedback } from '@/libs/blueos-api/services/recorder'
import { prettifySize } from '@/utils/helper_functions'

import {
  CANCEL_JOB,
  DELETE_RECORDING,
  REPAIR_BYTES_PER_SECOND_ESTIMATE,
  REPAIR_RECORDING,
} from './constants'
import type {
  LibraryRecording,
  RecordingJobResult,
  RecordingState,
} from './types'

export const RECORDING_STATE_UI: Record<RecordingState, { label: string, color: string }> = {
  recording: { label: 'Recording', color: 'warning' },
  needs_repair: { label: 'Needs repair', color: 'error' },
  repairing: { label: 'Repairing', color: 'primary' },
  ready: { label: 'Ready', color: 'success' },
}

/** The operations a row shows as buttons. A snapshot is not one: the Download of the file being written takes it. */
export const RECORDING_OPERATION_UI: Record<string, { label: string, icon: string, color: string }> = {
  [REPAIR_RECORDING]: { label: 'Repair', icon: 'mdi-wrench', color: 'primary' },
  [CANCEL_JOB]: { label: 'Cancel repair', icon: 'mdi-stop', color: 'primary' },
  [DELETE_RECORDING]: { label: 'Delete', icon: 'mdi-delete', color: 'error' },
}

/**
 * The operation buttons of `file`: Delete always and Repair whenever the file needs it, so a refused one stays visible
 * and says why, plus any other allowed operation that has a button.
 */
export function operationButtons(file: LibraryRecording): string[] {
  return Object.keys(RECORDING_OPERATION_UI).filter((operationName) => file.allowed_operations.includes(operationName)
    || operationName === DELETE_RECORDING
    || operationName === REPAIR_RECORDING && file.state === 'needs_repair')
}

/** Why the recorder refuses `operationName` for `file`, or null when it allows it. */
export function operationDisabledReason(file: LibraryRecording, operationName: string): string | null {
  if (file.allowed_operations.includes(operationName)) {
    return null
  }
  if (operationName === DELETE_RECORDING && file.state === 'recording') {
    return 'Cannot delete while the vehicle is still recording'
  }
  if (operationName === DELETE_RECORDING && file.state === 'repairing') {
    return 'Cannot delete while the recording is being repaired'
  }
  if (operationName === REPAIR_RECORDING) {
    return file.repair_error || 'Wait until the file is finished before repairing'
  }
  return 'The recorder does not allow this right now'
}

/**
 * How a recording downloads: a finished file as it is, even without an index, and the file being written through a
 * snapshot, the indexed copy of what has been written so far. A file being repaired does not download.
 */
export function recordingDownload(file: LibraryRecording): 'direct' | 'snapshot' | null {
  if (file.state === 'recording') {
    return 'snapshot'
  }
  if (file.state === 'repairing') {
    return null
  }
  return 'direct'
}

export function canDownloadRecording(file: LibraryRecording): boolean {
  return recordingDownload(file) !== null
}

export function downloadTooltip(file: LibraryRecording): string {
  if (file.state === 'needs_repair') {
    return `Download ${file.name}. It has no index: Repair makes it seekable`
  }
  if (file.state === 'repairing') {
    return 'Wait until repair finishes before downloading'
  }
  if (file.state === 'recording') {
    return 'Download what has been written so far'
  }
  return `Download ${file.name}`
}

/** The latest repair Feedback of each active repair Job, by Job id. */
export type RepairProgress = Record<string, RepairRecordingFeedback>

/**
 * Lays the live repair Jobs over the library rows: a repairing row shows the Feedback of its Job (also for a page
 * opened mid-repair), and offers no second cancel once its Job is canceling.
 */
export function withRepairJobs(
  files: LibraryRecording[],
  progress: RepairProgress,
  jobs: JobStatus[],
): LibraryRecording[] {
  return files.map((file) => {
    const feedback = progress[file.repair_job_id]
    const canceling = jobs.some(
      (job) => job.job_id === file.repair_job_id && job.status === JobStatusStatus.Canceling,
    )
    if (file.repair_job_id === '' || (feedback === undefined && !canceling)) {
      return file
    }
    return {
      ...file,
      repair_bytes_processed: feedback?.bytes_processed ?? file.repair_bytes_processed,
      repair_total_bytes: feedback?.total_bytes ?? file.repair_total_bytes,
      allowed_operations: canceling
        ? file.allowed_operations.filter((operationName) => operationName !== CANCEL_JOB)
        : file.allowed_operations,
    }
  })
}

/** How far a repairing row is, for the bar and the text under it; null when it is not repairing or its size is unknown. */
export function repairProgress(file: LibraryRecording): { percent: number, label: string } | null {
  if (file.state !== 'repairing' || file.repair_total_bytes <= 0) {
    return null
  }
  const processed = prettifySize(file.repair_bytes_processed / 1024)
  const total = prettifySize(file.repair_total_bytes / 1024)
  const bytesLeft = file.repair_total_bytes - file.repair_bytes_processed
  const timeLeft = file.repair_bytes_per_second > 0 && bytesLeft > 0
    ? ` \u00B7 ${formatDuration(bytesLeft / file.repair_bytes_per_second)} left`
    : ''
  return {
    percent: Math.min(100, file.repair_bytes_processed / file.repair_total_bytes * 100),
    label: `${processed} of ${total}${timeLeft}`,
  }
}

/** What repairing `targets` rewrites, how long it should take, and what it costs the vehicle meanwhile. */
export function repairEstimateMessage(targets: LibraryRecording[]): string {
  const bytes = targets.reduce((total, file) => total + file.size_bytes, 0)
  const seconds = bytes / REPAIR_BYTES_PER_SECOND_ESTIMATE
  const estimate = seconds < 60 ? 'less than a minute' : `around ${formatDuration(seconds)}`
  const what = targets.length === 1 ? targets[0].name : `${targets.length} recordings`
  return `Repairing ${what} rewrites ${prettifySize(bytes / 1024)} on the vehicle and should take ${estimate},`
    + ' keeping its disk and processor busy the whole time. Recording and streaming will be slower while it'
    + ' runs. You can already play this recording without repairing it, and you can stop the repair at any'
    + ' time.'
}

/** `seconds` as `2m 05s`, `3h 07m 09s` or `2d 5h 01m`. */
export function formatDuration(seconds: number): string {
  const total = Math.max(0, Math.round(seconds))
  const days = Math.floor(total / 86400)
  const hours = Math.floor(total % 86400 / 3600)
  const minutes = Math.floor(total % 3600 / 60)
  const paddedMinutes = String(minutes).padStart(2, '0')
  const paddedSeconds = String(total % 60).padStart(2, '0')
  if (days > 0) {
    return `${days}d ${hours}h ${paddedMinutes}m`
  }
  if (hours > 0) {
    return `${hours}h ${paddedMinutes}m ${paddedSeconds}s`
  }
  return `${minutes}m ${paddedSeconds}s`
}

/** The library row for `path`, or null when nothing is open or the path left the library. */
export function recordingByPath(
  recordings: LibraryRecording[],
  path: string | null,
): LibraryRecording | null {
  if (path === null) {
    return null
  }
  return recordings.find((file) => file.path === path) ?? null
}

/** `files` with the file being written lasting from its start until `nowSeconds`; `files` itself when none is. */
export function withLiveDuration(files: LibraryRecording[], nowSeconds: number): LibraryRecording[] {
  if (!files.some((file) => file.state === 'recording')) {
    return files
  }
  return files.map((file) => {
    if (file.state !== 'recording') {
      return file
    }
    return { ...file, duration_seconds: Math.max(0, nowSeconds - file.created) }
  })
}

/** The duration of `file` as text; "-" when unknown. */
export function durationLabel(file: LibraryRecording): string {
  return file.duration_seconds === null ? '-' : formatDuration(file.duration_seconds)
}

/** The video topics of `file` and how many other topics it has, as text. */
export function tracksLabel(file: LibraryRecording): string {
  if (file.video_topics === null || file.other_topic_count === null) {
    return 'unknown'
  }
  const parts = file.video_topics.length > 0 ? [file.video_topics.join(', ')] : []
  if (file.other_topic_count > 0) {
    parts.push(`${file.other_topic_count} other topic${file.other_topic_count === 1 ? '' : 's'}`)
  }
  return parts.length > 0 ? parts.join(' \u00B7 ') : 'no topics'
}

/** When a finished recording of known duration ended, in Unix seconds; null otherwise. */
export function recordingEndSeconds(file: LibraryRecording): number | null {
  if (file.state === 'recording' || file.duration_seconds === null) {
    return null
  }
  return file.created + file.duration_seconds
}

/** What a card says under its duration: that the file is being written, or why it needs repair. */
export function recordingCaption(file: LibraryRecording): string | null {
  if (file.state === 'recording') {
    return 'recording...'
  }
  if (file.state === 'needs_repair') {
    return file.repair_error || 'Recording index is missing'
  }
  return null
}

/** A missing index is not a missing recording: the vehicle indexes what was written on demand. */
export function canPlayRecording(file: LibraryRecording): boolean {
  return file.state === 'ready' || file.state === 'recording' || file.state === 'needs_repair'
}

/** A thumbnail seeks to the middle of the video, which needs the summary only a finished, indexed file has. */
export function canLoadThumbnail(file: LibraryRecording): boolean {
  return file.state === 'ready'
    && file.video_topics?.length !== 0
}

/** The snapshot a snapshot Job wrote, or null when the Job did not succeed. */
export function snapshotDownloadPath({ job, result }: RecordingJobResult): string | null {
  if (job.status !== JobStatusStatus.Succeeded || !('output_path' in result) || !result.output_path) {
    return null
  }
  return result.output_path
}

function snapshotPathPrefix(sourcePath: string): string {
  const segments = sourcePath.split('/')
  const fileName = segments.pop() ?? sourcePath
  const parent = segments.join('/')
  const stem = fileName.replace(/\.mcap$/i, '')
  return parent ? `${parent}/${stem}.snapshot-` : `${stem}.snapshot-`
}

function isReadySnapshotMcapForPrefix(file: LibraryRecording, prefix: string): boolean {
  return file.state === 'ready'
    && file.path.startsWith(prefix)
    && file.path.toLowerCase().endsWith('.mcap')
}

/** Ready snapshot MCAP paths for one source recording, newest first. */
export function snapshotPathsForSource(
  sourcePath: string,
  files: LibraryRecording[],
): string[] {
  const prefix = snapshotPathPrefix(sourcePath)
  return files
    .filter((file) => isReadySnapshotMcapForPrefix(file, prefix))
    .sort((left, right) => right.created - left.created)
    .map((file) => file.path)
}

export function readySnapshotDownloadPath(
  sourcePath: string,
  files: LibraryRecording[],
  excludePaths: ReadonlySet<string> = new Set(),
): string | null {
  const prefix = snapshotPathPrefix(sourcePath)
  const match = files
    .filter(
      (file) => isReadySnapshotMcapForPrefix(file, prefix) && !excludePaths.has(file.path),
    )
    .sort((left, right) => right.created - left.created)[0]
  return match?.path ?? null
}

export function deleteConfirmationMessage(targets: LibraryRecording[]): string {
  if (targets.length === 1) {
    return `Delete ${targets[0].name}? This cannot be undone.`
  }
  return `Delete ${targets.length} recordings? This cannot be undone.`
}

/**
 * Why a Job aborted, for the user, or null when it succeeded or was canceled. A snapshot's failure is null too: the
 * Download that took it reports it.
 */
export function jobFailureMessage({ job, result }: RecordingJobResult): string | null {
  if (job.status !== JobStatusStatus.Aborted || !(job.job_type in RECORDING_OPERATION_UI)) {
    return null
  }
  return `${RECORDING_OPERATION_UI[job.job_type].label} failed for ${result.path}: ${job.reason || 'unknown error'}`
}

/** Which Job the user canceled, or null when the Job ended any other way. */
export function jobCanceledMessage({ job, result }: RecordingJobResult): string | null {
  if (job.status !== JobStatusStatus.Canceled || !(job.job_type in RECORDING_OPERATION_UI)) {
    return null
  }
  return `${RECORDING_OPERATION_UI[job.job_type].label} canceled for ${result.path}`
}
