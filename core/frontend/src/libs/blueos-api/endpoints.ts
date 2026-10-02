import {
  commandKey, jobsKey, settingsKey, statusStateKey,
} from './keys'
import type { SchemaName } from './types'

/** A State endpoint, such as the generated `tank.tank`: published on change and answered to a query. */
export interface StateEndpoint<Schema extends SchemaName> {
  key: string
  messageSchema: Schema
}

/** A Command endpoint, such as the generated `tank.Drain`. It replies with a `CommandAck`. */
export interface CommandEndpoint<Schema extends SchemaName> {
  key: string
  requestSchema: Schema
}

/** A Query or IO query endpoint, such as the generated `tank.Level`. */
export interface QueryEndpoint<RequestSchema extends SchemaName, ResponseSchema extends SchemaName> {
  key: string
  requestSchema: RequestSchema
  responseSchema: ResponseSchema
}

/** The `status` State of a Service (D-12). A `service` of `*` matches every Service. */
export function statusState(service: string): StateEndpoint<'blueos_msgs/msg/ServiceStatus'> {
  return { key: statusStateKey(service), messageSchema: 'blueos_msgs/msg/ServiceStatus' }
}

/** The `settings` State of a Service, with its pending restart fields (D-11). */
export function settingsState(service: string): StateEndpoint<'blueos_msgs/msg/SettingsEnvelope'> {
  return { key: settingsKey(service), messageSchema: 'blueos_msgs/msg/SettingsEnvelope' }
}

/** The `jobs` State of a Service whose Domain has Jobs (D-12). */
export function jobsState(service: string): StateEndpoint<'blueos_msgs/msg/JobList'> {
  return { key: jobsKey(service), messageSchema: 'blueos_msgs/msg/JobList' }
}

/** The `UpdateSettings` Command of a Service, the only way to change its settings (D-11). */
export function updateSettingsCommand(service: string): CommandEndpoint<'blueos_msgs/msg/SettingsEnvelope'> {
  return { key: commandKey(service, 'UpdateSettings'), requestSchema: 'blueos_msgs/msg/SettingsEnvelope' }
}
