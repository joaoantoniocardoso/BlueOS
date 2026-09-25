/* eslint-disable import/no-extraneous-dependencies */
import type {
  CancelRepairCommand,
  CommandAck,
  DeleteRecordingCommand,
  EmptyRequest,
  Header,
  JobList,
  JobStatus,
  LevelQueryResponse,
  Log,
  PumpState,
  RecordingIndex,
  RecordingIndexRequest,
  RecordingLibrary,
  RecordingOperation,
  RecordingState,
  RepairRecordingCommand,
  RestartRequired,
  SelfTestCompleted,
  ServiceInfo,
  ServiceStatus,
  SetLevelRequest,
  SettingField,
  SettingsEnvelope,
  SnapshotRecordingCommand,
  Time,
} from '@blueos-idl/messages'
import { SCHEMAS } from '@blueos-idl/schemas'

export type SchemaName = keyof typeof SCHEMAS

export interface MessageBySchema {
  'blueos_example_msgs/msg/EmptyRequest': EmptyRequest
  'blueos_example_msgs/msg/LevelQueryResponse': LevelQueryResponse
  'blueos_example_msgs/msg/PumpState': PumpState
  'blueos_example_msgs/msg/SelfTestCompleted': SelfTestCompleted
  'blueos_example_msgs/msg/SetLevelRequest': SetLevelRequest
  'blueos_msgs/msg/CommandAck': CommandAck
  'blueos_msgs/msg/JobList': JobList
  'blueos_msgs/msg/JobStatus': JobStatus
  'blueos_msgs/msg/RestartRequired': RestartRequired
  'blueos_msgs/msg/ServiceInfo': ServiceInfo
  'blueos_msgs/msg/ServiceStatus': ServiceStatus
  'blueos_msgs/msg/SettingField': SettingField
  'blueos_msgs/msg/SettingsEnvelope': SettingsEnvelope
  'builtin_interfaces/msg/Time': Time
  'builtin_interfaces/msg/Duration': Time
  'blueos_recorder_msgs/msg/CancelRepairCommand': CancelRepairCommand
  'blueos_recorder_msgs/msg/DeleteRecordingCommand': DeleteRecordingCommand
  'blueos_recorder_msgs/msg/RecordingIndex': RecordingIndex
  'blueos_recorder_msgs/msg/RecordingIndexRequest': RecordingIndexRequest
  'blueos_recorder_msgs/msg/RecordingLibrary': RecordingLibrary
  'blueos_recorder_msgs/msg/RecordingOperation': RecordingOperation
  'blueos_recorder_msgs/msg/RecordingState': RecordingState
  'blueos_recorder_msgs/msg/RepairRecordingCommand': RepairRecordingCommand
  'blueos_recorder_msgs/msg/SnapshotRecordingCommand': SnapshotRecordingCommand
  'foxglove_msgs/msg/Log': Log
  'std_msgs/msg/Header': Header
}

export type MessageForSchema<Schema extends SchemaName> = MessageBySchema[Schema]

export const COMMAND_ACK_SCHEMA: SchemaName = 'blueos_msgs/msg/CommandAck'
export const JOB_LIST_SCHEMA: SchemaName = 'blueos_msgs/msg/JobList'
export const SERVICE_INFO_SCHEMA: SchemaName = 'blueos_msgs/msg/ServiceInfo'
export const LOG_SCHEMA: SchemaName = 'foxglove_msgs/msg/Log'
