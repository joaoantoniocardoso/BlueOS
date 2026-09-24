/* eslint-disable import/no-extraneous-dependencies */
import {
  Encoding,
  QueryTarget,
  ReplyError,
  Sample,
  Session,
  ZBytes,
} from '@eclipse-zenoh/zenoh-ts'
import { Duration } from 'typed-duration'

import zenoh, { receiveQueryReply, RecvErr } from '@/libs/zenoh'

import { decodeCdr, schemaNameFromEncoding } from './cdr'
import { BlueosApiError } from './errors'
import { cdrEncoding } from './keys'
import type { MessageForSchema, SchemaName } from './types'

export async function getZenohSession(): Promise<Session> {
  return zenoh.getSession()
}

export function samplePayloadBytes(sample: Sample): Uint8Array {
  return sample.payload().toBytes()
}

export function decodeSample<Schema extends SchemaName>(
  sample: Sample,
  expectedSchema: Schema,
): MessageForSchema<Schema> {
  const encoding = sample.encoding().toString()
  const schemaFromEncoding = schemaNameFromEncoding(encoding)
  if (schemaFromEncoding !== undefined && schemaFromEncoding !== expectedSchema) {
    throw new BlueosApiError(
      `Encoding schema mismatch: expected ${expectedSchema}, got ${schemaFromEncoding}`,
    )
  }
  return decodeCdr(expectedSchema, samplePayloadBytes(sample))
}

export interface ZenohQueryOptions {
  payload?: Uint8Array
  requestSchema?: SchemaName
  timeoutMs?: number
}

export async function zenohQueryFirstSample(
  key: string,
  options: ZenohQueryOptions = {},
): Promise<Sample> {
  const session = await getZenohSession()
  const timeoutMs = options.timeoutMs ?? 30_000
  const encoding = options.requestSchema !== undefined
    ? Encoding.APPLICATION_CDR.withSchema(options.requestSchema)
    : undefined

  const receiver = await session.get(key, {
    target: QueryTarget.BEST_MATCHING,
    timeout: Duration.milliseconds.of(timeoutMs),
    encoding,
    payload: options.payload !== undefined ? new ZBytes(options.payload) : undefined,
  })

  if (receiver === undefined) {
    throw new BlueosApiError(`Zenoh query failed for key ${key}`)
  }

  const reply = await receiveQueryReply(receiver)
  if (reply === RecvErr.Disconnected) {
    throw new BlueosApiError(`Zenoh query timed out or disconnected for key ${key}`)
  }

  const result = reply.result()
  if (result instanceof ReplyError) {
    throw new BlueosApiError(
      `Zenoh query error for key ${key}: ${result.payload().toString()}`,
    )
  }
  return result
}

export async function zenohQueryCdr<Schema extends SchemaName>(
  key: string,
  responseSchema: Schema,
  options: ZenohQueryOptions = {},
): Promise<MessageForSchema<Schema>> {
  const sample = await zenohQueryFirstSample(key, options)
  return decodeSample(sample, responseSchema)
}

export function cdrZenohEncoding(schemaName: SchemaName): Encoding {
  return Encoding.fromString(cdrEncoding(schemaName))
}

export type Unsubscribe = () => void
