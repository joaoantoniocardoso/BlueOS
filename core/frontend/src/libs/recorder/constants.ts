import { CANCEL_JOB } from '@/libs/blueos-api/command'
import {
  DeleteRecording,
  RepairRecording,
  SnapshotRecording,
} from '@/libs/blueos-api/services/recorder'

export const DEFAULT_RECORDING_HTTP_PREFIX = '/userdata/recorder'

export const SNAPSHOT_WAIT_TIMEOUT_MS = 600_000

export const REPAIR_RECORDING = RepairRecording.name
export { CANCEL_JOB }
export const DELETE_RECORDING = DeleteRecording.name
export const SNAPSHOT_RECORDING = SnapshotRecording.name
