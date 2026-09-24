import { encodeCdr } from './cdr'
import { queryKey } from './keys'
import type { MessageForSchema, SchemaName } from './types'
import { zenohQueryCdr } from './zenoh-helpers'

/* eslint-disable import/prefer-default-export */
export async function query<
  RequestSchema extends SchemaName | undefined,
  ResponseSchema extends SchemaName,
>(
  service: string,
  name: string,
  responseSchemaName: ResponseSchema,
  requestSchemaName?: RequestSchema,
  request?: RequestSchema extends SchemaName ? MessageForSchema<RequestSchema> : never,
  timeoutMs = 30_000,
): Promise<MessageForSchema<ResponseSchema>> {
  const key = queryKey(service, name)
  const payload = requestSchemaName !== undefined && request !== undefined
    ? encodeCdr(requestSchemaName, request)
    : undefined
  return zenohQueryCdr(key, responseSchemaName, {
    payload,
    requestSchema: requestSchemaName,
    timeoutMs,
  })
}
