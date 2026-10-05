import {
  RecordingFileState,
} from '@blueos-idl'
import type { RecordingContents, RecordingFile, Time } from '@blueos-idl/messages'

import type {
  LibraryRecording,
  RecordingState,
} from './types'

function timeToUnixSeconds(time: Time): number {
  return time.sec + time.nanosec / 1e9
}

export function mapRecordingState(state: number): RecordingState {
  switch (state) {
    case RecordingFileState.Recording:
      return 'recording'
    case RecordingFileState.Ready:
      return 'ready'
    case RecordingFileState.NeedsRepair:
      return 'needs_repair'
    case RecordingFileState.Repairing:
      return 'repairing'
    default:
      return 'needs_repair'
  }
}

/** Maps `file` with its `contents`, which the library carries only when it knows them. */
export function mapRecordingFile(file: RecordingFile, contents?: RecordingContents): LibraryRecording {
  return {
    path: file.path,
    name: file.name,
    size_bytes: file.size_bytes,
    created: timeToUnixSeconds(file.created),
    state: mapRecordingState(file.state),
    repair_bytes_processed: file.repair_bytes_processed,
    repair_total_bytes: file.repair_total_bytes,
    repair_bytes_per_second: file.repair_bytes_per_second,
    repair_error: file.repair_error,
    repair_job_id: file.repair_job_id,
    allowed_operations: [...file.allowed_operations],
    duration_seconds: contents ? contents.duration.sec + contents.duration.nanosec / 1e9 : null,
    video_topics: contents ? [...contents.video_topics] : null,
    other_topic_count: contents?.other_topic_count ?? null,
  }
}
