/* eslint-disable import/no-extraneous-dependencies */
import type { CommandAck } from '@blueos-idl/messages'

import { decodeSample, encodeCdr } from './cdr'
import type { CommandEndpoint, QueryEndpoint } from './endpoints'
import { NoReplyError, QueryFailedError } from './errors'
import { cdrEncoding } from './keys'
import type { Transport } from './transport'
import { COMMAND_ACK_SCHEMA, type MessageForSchema, type SchemaName } from './types'

/** The `CommandAck.job_id` of a Command that started no Job; job ids start at 1. */
export const JOB_ID_NONE = 0

/**
 * Sends a Command and returns the Service's verdict: accepted, or rejected with a `reason`, and the root `job_id` it
 * started. Progress and results come later as Events or States (D-10).
 */
export async function sendCommand<Schema extends SchemaName>(
  transport: Transport,
  command: CommandEndpoint<Schema>,
  request: MessageForSchema<Schema>,
): Promise<CommandAck> {
  return query(transport, { ...command, responseSchema: COMMAND_ACK_SCHEMA }, request)
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
  const [reply] = await transport.get(endpoint.key, {
    payload: encodeCdr(endpoint.requestSchema, request),
    encoding: cdrEncoding(endpoint.requestSchema),
  })
  if (reply === undefined) {
    throw new NoReplyError(endpoint.key)
  }
  if (reply.kind === 'error') {
    throw new QueryFailedError(endpoint.key, new TextDecoder().decode(reply.payload))
  }
  return decodeSample(reply.sample, endpoint.responseSchema)
}
