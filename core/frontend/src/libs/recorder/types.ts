import type { CommandAck, RecordingFile } from '@blueos-idl/messages'

import type { JobResultEntry } from '@/libs/blueos-api/job'
import type {
  DeleteRecordingResult,
  RepairRecordingResult,
  SnapshotRecordingResult,
} from '@/libs/blueos-api/services/recorder'

export type RecordingState = 'recording' | 'ready' | 'needs_repair' | 'repairing'

/** A `RecordingFile` of the library State, with its time in Unix seconds (UTC) and its state named. */
export type LibraryRecording = Omit<RecordingFile, 'created' | 'state'> & {
  created: number
  state: RecordingState
}

/** How a delete, repair or snapshot Job of one recording ended; `job.job_type` says which it was. */
export type RecordingJobResult = JobResultEntry<DeleteRecordingResult | RepairRecordingResult | SnapshotRecordingResult>

/** The verdict of a submitted Job; `job_id` names the Job to watch or cancel, `status` is its status in the ack. */
export type RecorderCommandResult = Pick<CommandAck, 'accepted' | 'job_id' | 'reason' | 'status'>
