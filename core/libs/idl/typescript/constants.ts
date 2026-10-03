// @generated

export const PumpStateSelfTestPhase = {
  Cancelled: 4,
  Failed: 3,
  Idle: 0,
  Passed: 2,
  Running: 1,
} as const;
export type PumpStateSelfTestPhase = typeof PumpStateSelfTestPhase[keyof typeof PumpStateSelfTestPhase] | number;

export const CommandAckStatus = {
  Aborted: 6,
  Accepted: 1,
  Canceled: 5,
  Canceling: 3,
  Executing: 2,
  Paused: 9,
  Succeeded: 4,
  StatusUnknown: 0,
  WaitingForPermission: 7,
  WaitingForResource: 8,
} as const;
export type CommandAckStatus = typeof CommandAckStatus[keyof typeof CommandAckStatus] | number;

export const JobStatusStatus = {
  Aborted: 6,
  Accepted: 1,
  Canceled: 5,
  Canceling: 3,
  Executing: 2,
  Paused: 9,
  Succeeded: 4,
  StatusUnknown: 0,
  WaitingForPermission: 7,
  WaitingForResource: 8,
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
  NeedsRepair: 2,
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

