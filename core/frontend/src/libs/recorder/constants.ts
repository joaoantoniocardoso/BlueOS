export const RECORDER_SERVICE = 'recorder'

export const RECORDING_LIBRARY_SCHEMA = 'blueos_recorder_msgs/msg/RecordingLibrary'
export const RECORDING_STATE_SCHEMA = 'blueos_recorder_msgs/msg/RecordingState'
export const RECORDING_OPERATION_SCHEMA = 'blueos_recorder_msgs/msg/RecordingOperation'
export const RECORDING_INDEX_SCHEMA = 'blueos_recorder_msgs/msg/RecordingIndex'
export const RECORDING_INDEX_REQUEST_SCHEMA = 'blueos_recorder_msgs/msg/RecordingIndexRequest'
export const REPAIR_RECORDING_COMMAND_SCHEMA = 'blueos_recorder_msgs/msg/RepairRecordingCommand'
export const CANCEL_REPAIR_COMMAND_SCHEMA = 'blueos_recorder_msgs/msg/CancelRepairCommand'
export const DELETE_RECORDING_COMMAND_SCHEMA = 'blueos_recorder_msgs/msg/DeleteRecordingCommand'
export const SNAPSHOT_RECORDING_COMMAND_SCHEMA = 'blueos_recorder_msgs/msg/SnapshotRecordingCommand'

export const STATE_RECORDING = 0
export const STATE_READY = 1
export const STATE_NEEDS_REPAIR = 2
export const STATE_REPAIRING = 3

export const OPERATION_REPAIR = 0
export const OPERATION_SNAPSHOT = 1
export const OPERATION_DELETE = 2

export const DEFAULT_RECORDING_HTTP_PREFIX = '/userdata/recorder'

/** Rewrite speed seen on a Pi 4 SD card, only used to estimate a repair before one is running. */
export const REPAIR_BYTES_PER_SECOND_ESTIMATE = 25 * 1024 * 1024

export const SNAPSHOT_WAIT_TIMEOUT_MS = 600_000

export const INDEX_PAGE_LIMIT = 2000
