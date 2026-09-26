/* eslint-disable import/no-extraneous-dependencies */
import type { CommandAck, ServiceInfo } from '@blueos-idl/messages'
import {
  Encoding, ReplyError, Sample, SampleKind, ZBytes,
} from '@eclipse-zenoh/zenoh-ts'

import {
  decodeCdrWithSchema, encodeCdrWithSchema, getServiceInfo, schemaNameFromEncoding,
} from '@/libs/blueos-api'
import { COMMAND_ACK_SCHEMA } from '@/libs/blueos-api/types'
import { getZenohSession, samplePayloadBytes, zenohQueryFirstSample } from '@/libs/blueos-api/zenoh-helpers'
import { receiveQueryReply, RecvErr } from '@/libs/zenoh'

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
    key: sample.keyexpr().toString(),
    payload: samplePayloadBytes(sample),
    encoding: sample.encoding().toString(),
    receivedAt: clock(),
    kind: sample.kind() === SampleKind.PUT ? 'put' : 'delete',
  }
}

function decodeReplySample(
  sample: Sample,
  responseSchemaName: string,
  schemaProvider: SchemaProvider,
): InspectorRequestResult {
  const schemaName = schemaNameFromEncoding(sample.encoding().toString()) ?? responseSchemaName
  const schemaText = schemaProvider.schemaText(schemaName)
  if (!schemaText) {
    throw new Error(`Unknown response schema ${schemaName}`)
  }
  if (schemaName === COMMAND_ACK_SCHEMA) {
    return {
      kind: 'ack',
      value: decodeCdrWithSchema(schemaName, schemaText, samplePayloadBytes(sample)) as CommandAck,
    }
  }
  return {
    kind: 'cdr',
    schemaName,
    value: decodeCdrWithSchema(schemaName, schemaText, samplePayloadBytes(sample)),
  }
}

export function createInspectorApiClient(
  schemaProvider: SchemaProvider,
  clock: () => number = Date.now,
): InspectorApiClient {
  return {
    serviceInfo(service: string): Promise<ServiceInfo> {
      return getServiceInfo(service)
    },

    async request(
      key: string,
      kind: InspectorRequestKind,
      requestSchemaName: string,
      responseSchemaName: string,
      message?: Record<string, unknown>,
    ): Promise<InspectorRequestResult> {
      let payload: Uint8Array | undefined
      if (message !== undefined && requestSchemaName) {
        const schemaText = schemaProvider.schemaText(requestSchemaName)
        if (!schemaText) {
          throw new Error(`Unknown request schema ${requestSchemaName}`)
        }
        payload = encodeCdrWithSchema(requestSchemaName, schemaText, message)
      }

      const sample = await zenohQueryFirstSample(key, {
        payload,
        requestSchema: requestSchemaName || undefined,
      })

      if (kind === 'command') {
        const ackText = schemaProvider.schemaText(COMMAND_ACK_SCHEMA)
        if (!ackText) {
          throw new Error('CommandAck schema is missing')
        }
        return {
          kind: 'ack',
          value: decodeCdrWithSchema(COMMAND_ACK_SCHEMA, ackText, samplePayloadBytes(sample)) as CommandAck,
        }
      }

      return decodeReplySample(sample, responseSchemaName, schemaProvider)
    },

    async rawQuery(key: string, text: string): Promise<SampleRecord[]> {
      const session = await getZenohSession()
      const payload = new TextEncoder().encode(text)
      const receiver = await session.get(key, {
        payload: new ZBytes(payload),
        encoding: Encoding.APPLICATION_JSON,
      })
      if (receiver === undefined) {
        return []
      }

      const records: SampleRecord[] = []
      // eslint-disable-next-line no-constant-condition
      while (true) {
        let reply
        try {
          // eslint-disable-next-line no-await-in-loop
          reply = await receiveQueryReply(receiver)
        } catch {
          break
        }
        if (reply === RecvErr.Disconnected) {
          break
        }
        const result = reply.result()
        if (result instanceof ReplyError) {
          continue
        }
        records.push(sampleToRecord(result, clock))
      }
      return records
    },
  }
}
