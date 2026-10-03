import { JobStatusStatus } from '@blueos-idl'
import type { JobStatus } from '@blueos-idl/messages'

import type { RepairRecordingFeedback } from '@/libs/blueos-api/services/recorder'

import {
  CANCEL_JOB,
  DELETE_RECORDING,
  REPAIR_RECORDING,
  SNAPSHOT_RECORDING,
} from './constants'
import type {
  LibraryRecording,
  RecordingOperationEvent,
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
