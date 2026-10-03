/* eslint-disable import/no-extraneous-dependencies */
import type { CommandAck, JobList } from '@blueos-idl/messages'

import { decodeSample, encodeCdr } from './cdr'
import type { CommandEndpoint, QueryEndpoint } from './endpoints'
import { NoReplyError, QueryFailedError } from './errors'
import { cdrEncoding, commandKey, jobHistoryKey } from './keys'
import type { QueryBody, Transport } from './transport'
import {
  COMMAND_ACK_SCHEMA, type MessageForSchema, PERMISSION_ANSWER_SCHEMA, type SchemaName,
} from './types'

/** The control endpoint that cancels a Job; Services list it in `allowed_operations` while a Job can be cancelled. */
export const CANCEL_JOB = 'CancelJob'

/**
 * A new Job id: a random version 4 UUID. Keep it to retry the Command or to control its Job (D-36).
 */
export function newJobId(): string {
  // crypto.randomUUID needs a secure context, and BlueOS is served over plain HTTP.
  const bytes = crypto.getRandomValues(new Uint8Array(16))
  bytes[6] = bytes[6] & 0x0f | 0x40
  bytes[8] = bytes[8] & 0x3f | 0x80
  const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}

/**
 * Submits a Command as the Job `jobId` and returns the Service's verdict: accepted with the Job's status, or rejected
 * with a `reason`. An instant Job acks its final status; a lasting one goes on in the `jobs` State (D-10, D-36).
 * Sending again with the same `jobId` and Goal is a retry that returns the same Job.
 */
export async function sendCommand<Schema extends SchemaName>(
  transport: Transport,
  command: CommandEndpoint<Schema>,
  goal: MessageForSchema<Schema>,
  jobId: string = newJobId(),
): Promise<CommandAck> {
  return ask(transport, command.key, {
    payload: encodeCdr(command.goalSchema, goal),
    encoding: cdrEncoding(command.goalSchema),
    attachment: new TextEncoder().encode(jobId),
  }, COMMAND_ACK_SCHEMA)
}

/** Asks the Service to cancel the Job `jobId`. The ack is rejected when its Job type cannot be cancelled. */
export async function cancelJob(transport: Transport, service: string, jobId: string): Promise<CommandAck> {
  return control(transport, service, CANCEL_JOB, jobId)
}

/** Asks the Service to pause the Job `jobId`. The ack is rejected when its Job type cannot be paused. */
export async function pauseJob(transport: Transport, service: string, jobId: string): Promise<CommandAck> {
  return control(transport, service, 'PauseJob', jobId)
}

/** Asks the Service to resume the paused Job `jobId`. */
export async function resumeJob(transport: Transport, service: string, jobId: string): Promise<CommandAck> {
  return control(transport, service, 'ResumeJob', jobId)
}

/** Grants or denies the permission the Job `jobId` waits for. A denied Job ends Canceled. */
export async function answerPermission(
  transport: Transport,
  service: string,
  jobId: string,
  granted: boolean,
): Promise<CommandAck> {
  const endpoint = { key: commandKey(service, 'AnswerPermission'), goalSchema: PERMISSION_ANSWER_SCHEMA }
  return sendCommand(transport, endpoint, { granted }, jobId)
}

/**
 * Sends a Query and returns its decoded response. Throws `NoReplyError` when no Service answers, and
 * `QueryFailedError` when it answers with an error.
 */
export async function query<RequestSchema extends SchemaName, ResponseSchema extends SchemaName>(
  transport: Transport,
  endpoint: QueryEndpoint<RequestSchema, ResponseSchema>,
  request: MessageForSchema<RequestSchema>,
): Promise<MessageForSchema<ResponseSchema>> {
  return ask(transport, endpoint.key, {
    payload: encodeCdr(endpoint.requestSchema, request),
    encoding: cdrEncoding(endpoint.requestSchema),
  }, endpoint.responseSchema)
}

/** Calls the history Query of the Job type `jobType`: its last finished Jobs, in the order they ended (D-12). */
export async function jobHistory(transport: Transport, service: string, jobType: string): Promise<JobList> {
  return ask(transport, jobHistoryKey(service, jobType), undefined, 'blueos_msgs/msg/JobList')
}

async function control(transport: Transport, service: string, name: string, jobId: string): Promise<CommandAck> {
  return sendCommand(transport, { key: commandKey(service, name), goalSchema: 'std_msgs/msg/Empty' }, {}, jobId)
}

async function ask<ResponseSchema extends SchemaName>(
  transport: Transport,
  key: string,
  body: QueryBody | undefined,
  responseSchema: ResponseSchema,
): Promise<MessageForSchema<ResponseSchema>> {
  const [reply] = await transport.get(key, body)
  if (reply === undefined) {
    throw new NoReplyError(key)
  }
  if (reply.kind === 'error') {
    throw new QueryFailedError(key, new TextDecoder().decode(reply.payload))
  }
  return decodeSample(reply.sample, responseSchema)
}
