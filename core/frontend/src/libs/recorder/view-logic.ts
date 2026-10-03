import { JobStatusStatus } from '@blueos-idl'
import type { JobStatus } from '@blueos-idl/messages'

import type { RepairRecordingFeedback } from '@/libs/blueos-api/services/recorder'
import { prettifySize } from '@/utils/helper_functions'

import {
  CANCEL_JOB,
  DELETE_RECORDING,
  REPAIR_RECORDING,
  SNAPSHOT_RECORDING,
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

export const RECORDING_OPERATION_UI: Record<string, { label: string, icon: string, color: string }> = {
  [REPAIR_RECORDING]: { label: 'Repair', icon: 'mdi-wrench', color: 'primary' },
  [CANCEL_JOB]: { label: 'Cancel repair', icon: 'mdi-stop', color: 'primary' },
  [DELETE_RECORDING]: { label: 'Delete', icon: 'mdi-delete', color: 'error' },
  [SNAPSHOT_RECORDING]: { label: 'Download snapshot', icon: 'mdi-download', color: 'primary' },
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

export function sortRecordingsNewestFirst(files: LibraryRecording[]): LibraryRecording[] {
  return [...files].sort((left, right) => {
    if (right.created !== left.created) {
      return right.created - left.created
    }
    return right.path.localeCompare(left.path)
  })
}

export function canPlayRecording(file: LibraryRecording): boolean {
  return file.state === 'ready' || file.state === 'recording'
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

/** Why a Job aborted, for the user, or null when it succeeded or was canceled. */
export function jobFailureMessage({ job, result }: RecordingJobResult): string | null {
  if (job.status !== JobStatusStatus.Aborted) {
    return null
  }
  return `${RECORDING_OPERATION_UI[job.job_type].label} failed for ${result.path}: ${job.reason || 'unknown error'}`
}

/** Which Job the user canceled, or null when the Job ended any other way. */
export function jobCanceledMessage({ job, result }: RecordingJobResult): string | null {
  if (job.status !== JobStatusStatus.Canceled) {
    return null
  }
  return `${RECORDING_OPERATION_UI[job.job_type].label} canceled for ${result.path}`
}
