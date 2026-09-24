/* eslint-disable import/no-extraneous-dependencies */
import type {
  CommandAck,
  Header,
  JobList,
  JobStatus,
  Log,
  RestartRequired,
  ServiceInfo,
  ServiceStatus,
  SettingField,
  SettingsEnvelope,
  Time,
} from '@blueos-idl/messages'
import { SCHEMAS } from '@blueos-idl/schemas'

export type SchemaName = keyof typeof SCHEMAS

export interface MessageBySchema {
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
  'foxglove_msgs/msg/Log': Log
  'std_msgs/msg/Header': Header
}

export type MessageForSchema<Schema extends SchemaName> = MessageBySchema[Schema]

export const COMMAND_ACK_SCHEMA: SchemaName = 'blueos_msgs/msg/CommandAck'
export const JOB_LIST_SCHEMA: SchemaName = 'blueos_msgs/msg/JobList'
export const SERVICE_INFO_SCHEMA: SchemaName = 'blueos_msgs/msg/ServiceInfo'
export const LOG_SCHEMA: SchemaName = 'foxglove_msgs/msg/Log'
