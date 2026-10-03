import { JobStatusStatus } from '@blueos-idl'
import type { JobStatus } from '@blueos-idl/messages'

import type { RepairRecordingFeedback } from '@/libs/blueos-api/services/recorder'
import { prettifySize } from '@/utils/helper_functions'

import {
  CANCEL_JOB,
  DELETE_RECORDING,
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
  return {
    percent: Math.min(100, file.repair_bytes_processed / file.repair_total_bytes * 100),
    label: `${processed} of ${total}`,
  }
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

/** A missing index is not a missing recording: the vehicle indexes what was written on demand. */
export function canPlayRecording(file: LibraryRecording): boolean {
  return file.state === 'ready' || file.state === 'recording' || file.state === 'needs_repair'
}

/** A thumbnail seeks to the middle of the video, which needs the summary only a finished, indexed file has. */
export function canLoadThumbnail(file: LibraryRecording): boolean {
  return file.state === 'ready'
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
