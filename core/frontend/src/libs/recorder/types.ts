import type { CommandAck, RecordingFile } from '@blueos-idl/messages'

export type RecordingState = 'recording' | 'ready' | 'needs_repair' | 'repairing'

export type RecordingOperationKind = 'repair' | 'snapshot' | 'delete'

/** A `RecordingFile` of the library State, with its time in Unix seconds (UTC) and its state named. */
export type LibraryRecording = Omit<RecordingFile, 'created' | 'state'> & {
  created: number
  state: RecordingState
}

export interface RecordingOperationEvent {
  operation: RecordingOperationKind
  path: string
  output_path: string
  succeeded: boolean
  cancelled: boolean
  error: string
}

/** The verdict of a submitted Job; `job_id` names the Job to watch or cancel, `status` is its status in the ack. */
export type RecorderCommandResult = Pick<CommandAck, 'accepted' | 'job_id' | 'reason' | 'status'>
