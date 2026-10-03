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

/** The busy operation of a row while its Download runs in the browser. */
export const DOWNLOAD = 'Download'

export const RECORDS_LEAVE_MESSAGE = 'Keep this page open. Downloads and exports run in this browser'
  + ' and will stop if you leave.'
  + ' Repair on the vehicle continues, but you would lose progress shown here.'
