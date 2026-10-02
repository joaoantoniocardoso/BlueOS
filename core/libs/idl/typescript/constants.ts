// @generated

export const PumpStateSelfTestPhase = {
  Cancelled: 4,
  Failed: 3,
  Idle: 0,
  Passed: 2,
  Running: 1,
} as const;
export type PumpStateSelfTestPhase = typeof PumpStateSelfTestPhase[keyof typeof PumpStateSelfTestPhase] | number;

export const JobStatusStatus = {
  Cancelled: 5,
  Cancelling: 2,
  Failed: 4,
  Interrupted: 6,
  Queued: 0,
  Running: 1,
  Succeeded: 3,
} as const;
export type JobStatusStatus = typeof JobStatusStatus[keyof typeof JobStatusStatus] | number;

export const ServiceStatusStatus = {
  Degraded: 3,
  Ready: 2,
  Starting: 1,
  Stopping: 4,
  StatusUnknown: 0,
} as const;
export type ServiceStatusStatus = typeof ServiceStatusStatus[keyof typeof ServiceStatusStatus] | number;

export const RecordingFileState = {
  Ready: 1,
  Recording: 0,
  Repairing: 3,
} as const;
export type RecordingFileState = typeof RecordingFileState[keyof typeof RecordingFileState] | number;

export const RecordingOperationOperation = {
  Delete: 2,
  Repair: 0,
  Snapshot: 1,
} as const;
export type RecordingOperationOperation = typeof RecordingOperationOperation[keyof typeof RecordingOperationOperation] | number;

