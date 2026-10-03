import type { LibraryRecording, RecorderCommandResult, RecordingJobResult } from './types'
import { jobFailureMessage, RECORDING_OPERATION_UI } from './view-logic'

/**
 * One operation on many recordings: why some failed so far, and the paths whose Job may still end. A path leaves
 * `pending` when its submission is refused or its Job ends, in whichever order the ack and the Job result arrive.
 */
export interface BulkAction {
  failures: string[]
  pending: string[]
}

export function bulkActionTargets(
  recordings: readonly LibraryRecording[],
  operationName: string,
): LibraryRecording[] {
  return recordings.filter((file) => file.allowed_operations.includes(operationName))
}

/** Submits `operationName` for each pending path, one at a time: a refused or unreachable path does not stop the rest. */
export async function runBulkAction(
  action: BulkAction,
  operationName: string,
  submit: (path: string) => Promise<RecorderCommandResult>,
): Promise<void> {
  const { label } = RECORDING_OPERATION_UI[operationName]
  for (const path of [...action.pending]) {
    try {
      // eslint-disable-next-line no-await-in-loop
      const { accepted, reason } = await submit(path)
      if (!accepted) {
        settle(action, path, `${label} rejected for ${path}: ${reason}`)
      }
    } catch (error) {
      settle(action, path, `${label} failed for ${path}: ${error instanceof Error ? error.message : String(error)}`)
    }
  }
}

/** Takes the ended Job of `entry` out of the bulk action; false when its path is not part of it. */
export function bulkJobEnded(action: BulkAction, entry: RecordingJobResult): boolean {
  return settle(action, entry.result.path, jobFailureMessage(entry))
}

function settle(action: BulkAction, path: string, failure: string | null): boolean {
  const index = action.pending.indexOf(path)
  if (index === -1) {
    return false
  }
  action.pending.splice(index, 1)
  if (failure !== null) {
    action.failures.push(failure)
  }
  return true
}
