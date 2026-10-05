import type { CommandAck, RecordingFile } from '@blueos-idl/messages'

import type { JobResultEntry } from '@/libs/blueos-api/job'
import type {
  DeleteRecordingResult,
  RepairRecordingResult,
  SnapshotRecordingResult,
} from '@/libs/blueos-api/services/recorder'

export type RecordingState = 'recording' | 'ready' | 'needs_repair' | 'repairing'

/**
 * A `RecordingFile` of the library State, with its time in Unix seconds (UTC), its state named, and its
 * `RecordingContents`: the duration in seconds, the topics of its video channels and how many other topics it has.
 * All three are `null` when the library does not know them; an empty `video_topics` means no video.
 */
export type LibraryRecording = Omit<RecordingFile, 'created' | 'state'> & {
  created: number
  state: RecordingState
  duration_seconds: number | null
  video_topics: string[] | null
  other_topic_count: number | null
}

/** How a delete, repair or snapshot Job of one recording ended; `job.job_type` says which it was. */
export type RecordingJobResult = JobResultEntry<DeleteRecordingResult | RepairRecordingResult | SnapshotRecordingResult>

/** The verdict of a submitted Job; `job_id` names the Job to watch or cancel, `status` is its status in the ack. */
export type RecorderCommandResult = Pick<CommandAck, 'accepted' | 'job_id' | 'reason' | 'status'>
