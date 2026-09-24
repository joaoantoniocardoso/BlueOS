import type { CommandAck } from '@blueos-idl/messages'

import { encodeCdr } from './cdr'
import { commandKey } from './keys'
import { COMMAND_ACK_SCHEMA, type MessageForSchema, type SchemaName } from './types'
import { zenohQueryCdr } from './zenoh-helpers'

/* eslint-disable import/prefer-default-export */
export async function sendCommand<RequestSchema extends SchemaName>(
  service: string,
  name: string,
  requestSchemaName: RequestSchema,
  request: MessageForSchema<RequestSchema>,
  timeoutMs = 30_000,
): Promise<CommandAck> {
  const key = commandKey(service, name)
  const payload = encodeCdr(requestSchemaName, request)
  return zenohQueryCdr(key, COMMAND_ACK_SCHEMA, {
    payload,
    requestSchema: requestSchemaName,
    timeoutMs,
  })
}
