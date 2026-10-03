import type { LibraryRecording, RecorderCommandResult, RecordingOperationEvent } from './types'
import { operationFailureMessage } from './view-logic'

export interface BulkSubmissionOutcome {
  path: string
  accepted: boolean
  reason: string
  job_id: string
  transportError: boolean
}

export type BulkDeleteRecording = (path: string) => Promise<RecorderCommandResult>
export type BulkRepairRecording = (path: string) => Promise<RecorderCommandResult>

async function submitBulk(
  paths: readonly string[],
  submit: (path: string) => Promise<RecorderCommandResult>,
): Promise<BulkSubmissionOutcome[]> {
  const outcomes: BulkSubmissionOutcome[] = []
  for (const path of paths) {
    try {
      // eslint-disable-next-line no-await-in-loop
      const result = await submit(path)
      outcomes.push({
        path,
        accepted: result.accepted,
        reason: result.reason,
        job_id: result.job_id,
        transportError: false,
      })
    } catch (error) {
      outcomes.push({
        path,
        accepted: false,
        reason: error instanceof Error ? error.message : String(error),
        job_id: '',
        transportError: true,
      })
    }
  }
  return outcomes
}

export function submitBulkDelete(
  paths: readonly string[],
  deleteRecording: BulkDeleteRecording,
): Promise<BulkSubmissionOutcome[]> {
  return submitBulk(paths, deleteRecording)
}

export function submitBulkRepair(
  paths: readonly string[],
  repairRecording: BulkRepairRecording,
): Promise<BulkSubmissionOutcome[]> {
  return submitBulk(paths, repairRecording)
}

export function bulkActionTargets(
  recordings: readonly LibraryRecording[],
  operationName: string,
): LibraryRecording[] {
  return recordings.filter((file) => file.allowed_operations.includes(operationName))
}

function submissionFailureMessages(
  outcomes: readonly BulkSubmissionOutcome[],
  verb: 'Delete' | 'Repair',
): string[] {
  return outcomes.flatMap((outcome) => {
    if (outcome.accepted) {
      return []
    }
    if (outcome.transportError) {
      return [`${verb} failed for ${outcome.path}: ${outcome.reason}`]
    }
    return [`${verb} rejected for ${outcome.path}: ${outcome.reason}`]
  })
}

export function bulkDeleteSubmissionFailureMessages(outcomes: readonly BulkSubmissionOutcome[]): string[] {
  return submissionFailureMessages(outcomes, 'Delete')
}

export function bulkRepairSubmissionFailureMessages(outcomes: readonly BulkSubmissionOutcome[]): string[] {
  return submissionFailureMessages(outcomes, 'Repair')
}

export function bulkOperationFailureMessages(
  events: readonly RecordingOperationEvent[],
  bulkPaths: ReadonlySet<string>,
): string[] {
  return events.flatMap((event) => {
    if (!bulkPaths.has(event.path)) {
      return []
    }
    const message = operationFailureMessage(event, event.path)
    return message ? [message] : []
  })
}

export function appendBulkOperationFailure(
  messages: readonly string[],
  event: RecordingOperationEvent,
  bulkPaths: ReadonlySet<string>,
): string[] {
  const added = bulkOperationFailureMessages([event], bulkPaths)
  if (added.length === 0) {
    return [...messages]
  }
  return [...messages, added[0]]
}

export function formatBulkFailureMessages(messages: readonly string[]): string {
  return messages.join('\n')
}
