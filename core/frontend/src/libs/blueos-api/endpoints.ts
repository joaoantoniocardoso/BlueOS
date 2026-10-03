import {
  commandKey, jobFeedbackKey, jobResultKey, jobsKey, settingsKey, statusStateKey,
} from './keys'
import type { SchemaName } from './types'
import type { EventEndpoint } from './watch-event'

/** A State endpoint, such as the generated `example.pump`: published on change and answered to a query. */
export interface StateEndpoint<Schema extends SchemaName> {
  key: string
  messageSchema: Schema
}

/** A Command endpoint, such as the generated `example.SetLevel`. It replies with a `CommandAck`. */
export interface CommandEndpoint<Schema extends SchemaName> {
  key: string
  requestSchema: Schema
}

/** A Query or IO query endpoint, such as the generated `example.Level`. */
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

/** The `jobs` State of a Service: its active Jobs, those waiting for permission too, and the last ended ones. */
export function jobsState(service: string): StateEndpoint<'blueos_msgs/msg/JobList'> {
  return { key: jobsKey(service), messageSchema: 'blueos_msgs/msg/JobList' }
}

/** The Feedback State of a Job type: the latest Feedback of each of its active Jobs (D-12). */
export function jobFeedbackState(service: string, jobType: string): StateEndpoint<'blueos_msgs/msg/JobFeedbackList'> {
  return { key: jobFeedbackKey(service, jobType), messageSchema: 'blueos_msgs/msg/JobFeedbackList' }
}

/** The Job result Event of a Job type, published when one of its Jobs ends (D-12). */
export function jobResultEvent(service: string, jobType: string): EventEndpoint<'blueos_msgs/msg/JobResult'> {
  return { key: jobResultKey(service, jobType), messageSchema: 'blueos_msgs/msg/JobResult' }
}

/** The `UpdateSettings` instant Job type of a Service, the only way to change its settings (D-11, D-12). */
export function updateSettingsCommand(service: string): CommandEndpoint<'blueos_msgs/msg/SettingsEnvelope'> {
  return { key: commandKey(service, 'UpdateSettings'), requestSchema: 'blueos_msgs/msg/SettingsEnvelope' }
}
