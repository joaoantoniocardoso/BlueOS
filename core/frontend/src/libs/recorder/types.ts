export type RecordingState = 'recording' | 'ready' | 'needs_repair' | 'repairing'

export type RecordingOperationKind = 'repair' | 'snapshot' | 'delete'

export interface LibraryRecording {
  path: string
  name: string
  size_bytes: number
  /** Unix seconds (UTC), from the file name or filesystem time. */
  created: number
  state: RecordingState
  repair_bytes_processed: number
  repair_total_bytes: number
  repair_bytes_per_second: number
  repair_error: string
}

export interface RecordingOperationEvent {
  operation: RecordingOperationKind
  path: string
  output_path: string
  succeeded: boolean
  cancelled: boolean
  error: string
}

export interface RecorderCommandResult {
  accepted: boolean
  reason: string
}

export interface RepairProgress {
  percent: number
  label: string
}
