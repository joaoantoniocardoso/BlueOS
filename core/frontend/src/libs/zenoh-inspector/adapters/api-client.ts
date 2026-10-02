/* eslint-disable import/no-extraneous-dependencies */
import type { CommandAck, ServiceInfo } from '@blueos-idl/messages'

import {
  decodeCdrWithSchema,
  encodeCdrWithSchema,
  schemaNameFromEncoding,
} from '@/libs/blueos-api/cdr'
import { NoReplyError, QueryFailedError } from '@/libs/blueos-api/errors'
import { cdrEncoding, infoQueryKey } from '@/libs/blueos-api/keys'
import type { Sample, Transport } from '@/libs/blueos-api/transport'
import { COMMAND_ACK_SCHEMA, SERVICE_INFO_SCHEMA } from '@/libs/blueos-api/types'

import type { SampleRecord, SchemaProvider } from '../logic/types'

export type InspectorRequestKind = 'command' | 'query' | 'io_query'

export type InspectorRequestResult =
  | { kind: 'ack', value: CommandAck }
  | { kind: 'cdr', schemaName: string, value: unknown }

export interface InspectorApiClient {
  serviceInfo(service: string): Promise<ServiceInfo>
  request(
    key: string,
    kind: InspectorRequestKind,
    requestSchemaName: string,
    responseSchemaName: string,
    message?: Record<string, unknown>,
  ): Promise<InspectorRequestResult>
  rawQuery(key: string, text: string): Promise<SampleRecord[]>
}

function sampleToRecord(sample: Sample, clock: () => number): SampleRecord {
  return {
    key: sample.key,
    payload: sample.payload,
    encoding: sample.encoding,
    receivedAt: clock(),
    kind: 'put',
  }
}

function decodeReplySample(
  sample: Sample,
  responseSchemaName: string,
  schemaProvider: SchemaProvider,
): InspectorRequestResult {
  const schemaName = schemaNameFromEncoding(sample.encoding) ?? responseSchemaName
  const schemaText = schemaProvider.schemaText(schemaName)
  if (!schemaText) {
    throw new Error(`Unknown response schema ${schemaName}`)
  }
  if (schemaName === COMMAND_ACK_SCHEMA) {
    return {
      kind: 'ack',
      value: decodeCdrWithSchema(schemaName, schemaText, sample.payload) as CommandAck,
    }
  }
  return {
    kind: 'cdr',
    schemaName,
    value: decodeCdrWithSchema(schemaName, schemaText, sample.payload),
  }
}

function firstSampleReply(transport: Transport, key: string, body?: {
  payload: Uint8Array
  encoding: string
}): Promise<Sample> {
  return transport.get(key, body).then((replies) => {
    const reply = replies[0]
    if (reply === undefined) {
      throw new NoReplyError(key)
    }
    if (reply.kind === 'error') {
      throw new QueryFailedError(key, new TextDecoder().decode(reply.payload))
    }
    return reply.sample
  })
}

export function createInspectorApiClient(
  transportProvider: () => Promise<Transport>,
  schemaProvider: SchemaProvider,
  clock: () => number = Date.now,
): InspectorApiClient {
  return {
    async serviceInfo(service: string): Promise<ServiceInfo> {
      const transport = await transportProvider()
      const key = infoQueryKey(service)
      const sample = await firstSampleReply(transport, key)
      const schemaText = schemaProvider.schemaText(SERVICE_INFO_SCHEMA)
      if (!schemaText) {
        throw new Error('ServiceInfo schema is missing')
      }
      return decodeCdrWithSchema(SERVICE_INFO_SCHEMA, schemaText, sample.payload) as ServiceInfo
    },

    async request(
      key: string,
      kind: InspectorRequestKind,
      requestSchemaName: string,
      responseSchemaName: string,
      message?: Record<string, unknown>,
    ): Promise<InspectorRequestResult> {
      const transport = await transportProvider()
      let body: { payload: Uint8Array, encoding: string } | undefined
      if (message !== undefined && requestSchemaName) {
        const schemaText = schemaProvider.schemaText(requestSchemaName)
        if (!schemaText) {
          throw new Error(`Unknown request schema ${requestSchemaName}`)
        }
        body = {
          payload: encodeCdrWithSchema(requestSchemaName, schemaText, message),
          encoding: cdrEncoding(requestSchemaName),
        }
      }

      const sample = await firstSampleReply(transport, key, body)

      if (kind === 'command') {
        const ackText = schemaProvider.schemaText(COMMAND_ACK_SCHEMA)
        if (!ackText) {
          throw new Error('CommandAck schema is missing')
        }
        return {
          kind: 'ack',
          value: decodeCdrWithSchema(COMMAND_ACK_SCHEMA, ackText, sample.payload) as CommandAck,
        }
      }

      return decodeReplySample(sample, responseSchemaName, schemaProvider)
    },

    async rawQuery(key: string, text: string): Promise<SampleRecord[]> {
      const transport = await transportProvider()
      const replies = await transport.get(key, {
        payload: new TextEncoder().encode(text),
        encoding: 'application/json',
      })
      return replies
        .filter((reply): reply is { kind: 'sample', sample: Sample } => reply.kind === 'sample')
        .map((reply) => sampleToRecord(reply.sample, clock))
    },
  }
}
