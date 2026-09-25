import type { RecordingFile, RecordingOperation, Time } from '@blueos-idl/messages'

import {
  OPERATION_DELETE,
  OPERATION_REPAIR,
  OPERATION_SNAPSHOT,
  STATE_NEEDS_REPAIR,
  STATE_READY,
  STATE_RECORDING,
  STATE_REPAIRING,
} from './constants'
import type {
  LibraryRecording,
  RecordingOperationEvent,
  RecordingOperationKind,
  RecordingState,
} from './types'

function timeToUnixSeconds(time: Time): number {
  return time.sec + time.nanosec / 1e9
}

export function mapRecordingState(state: number): RecordingState {
  switch (state) {
    case STATE_RECORDING:
      return 'recording'
    case STATE_READY:
      return 'ready'
    case STATE_NEEDS_REPAIR:
      return 'needs_repair'
    case STATE_REPAIRING:
      return 'repairing'
    default:
      return 'needs_repair'
  }
}

export function mapRecordingFile(file: RecordingFile): LibraryRecording {
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
  }
}

function mapOperationKind(operation: number): RecordingOperationKind {
  switch (operation) {
    case OPERATION_REPAIR:
      return 'repair'
    case OPERATION_SNAPSHOT:
      return 'snapshot'
    case OPERATION_DELETE:
      return 'delete'
    default:
      return 'repair'
  }
}

export function mapRecordingOperation(event: RecordingOperation): RecordingOperationEvent {
  return {
    operation: mapOperationKind(event.operation),
    path: event.path,
    output_path: event.output_path,
    succeeded: event.succeeded,
    cancelled: event.cancelled,
    error: event.error,
  }
}
