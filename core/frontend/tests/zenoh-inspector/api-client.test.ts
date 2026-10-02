/* eslint-disable import/no-extraneous-dependencies */
import type { CommandAck, ServiceInfo } from '@blueos-idl/messages'
import { SCHEMAS } from '@blueos-idl/schemas'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding, infoQueryKey } from '@/libs/blueos-api/keys'
import { Drain } from '@/libs/blueos-api/services/tank'
import { COMMAND_ACK_SCHEMA, SERVICE_INFO_SCHEMA } from '@/libs/blueos-api/types'
import { createInspectorApiClient } from '@/libs/zenoh-inspector/adapters/api-client'
import type { SchemaProvider } from '@/libs/zenoh-inspector/logic/types'

import FakeTransport from '../blueos-api/fake-transport'

function idlSchemaProvider(): SchemaProvider {
  return {
    schemaText(schemaName: string): string | undefined {
      return SCHEMAS[schemaName as keyof typeof SCHEMAS]
    },
    ready(): Promise<void> {
      return Promise.resolve()
    },
  }
}

function tankServiceInfo(): ServiceInfo {
  return {
    name: 'tank',
    version: '0.1.0',
    build: 'test',
    capabilities: [],
    endpoints: [
      {
        kind: 'command',
        name: Drain.name,
        key: Drain.key,
        request_schema: Drain.requestSchema,
        response_schema: COMMAND_ACK_SCHEMA,
      },
      {
        kind: 'query',
        name: 'Level',
        key: 'blueos/v1/tank/query/Level',
        request_schema: 'blueos_example_msgs/msg/EmptyRequest',
        response_schema: 'blueos_example_msgs/msg/LevelQueryResponse',
      },
      {
        kind: 'state',
        name: 'tank',
        key: 'blueos/v1/tank/state/tank',
        request_schema: '',
        response_schema: 'blueos_example_msgs/msg/LevelQueryResponse',
      },
      {
        kind: 'event',
        name: 'Emptied',
        key: 'blueos/v1/tank/event/Emptied',
        request_schema: '',
        response_schema: 'blueos_example_msgs/msg/EmptyRequest',
      },
    ],
  }
}

describe('createInspectorApiClient', () => {
  it('lists every manifest endpoint from a Rust Service info query', async () => {
    const transport = new FakeTransport()
    const client = createInspectorApiClient(async () => transport, idlSchemaProvider())
    const pending = transport.nextQuery()
    const infoPromise = client.serviceInfo('tank')
    const query = await pending
    expect(query.key).toBe(infoQueryKey('tank'))
    expect(query.body).toBeUndefined()
    query.reply({
      kind: 'sample',
      sample: {
        key: infoQueryKey('tank'),
        payload: encodeCdr(SERVICE_INFO_SCHEMA, tankServiceInfo()),
        encoding: cdrEncoding(SERVICE_INFO_SCHEMA),
      },
    })
    const info = await infoPromise
    expect(info.endpoints).toHaveLength(4)
  })

  it('returns an empty endpoint list for a Python commonwealth Service', async () => {
    const transport = new FakeTransport()
    const client = createInspectorApiClient(async () => transport, idlSchemaProvider())
    const pending = transport.nextQuery()
    const infoPromise = client.serviceInfo('cable_guy')
    const query = await pending
    query.reply({
      kind: 'sample',
      sample: {
        key: infoQueryKey('cable_guy'),
        payload: encodeCdr(SERVICE_INFO_SCHEMA, {
          name: 'cable_guy',
          version: '1',
          build: '',
          capabilities: [],
          endpoints: [],
        }),
        encoding: cdrEncoding(SERVICE_INFO_SCHEMA),
      },
    })
    const info = await infoPromise
    expect(info.endpoints).toEqual([])
  })

  it('sends a Command from the generated form schemas', async () => {
    const transport = new FakeTransport()
    const client = createInspectorApiClient(async () => transport, idlSchemaProvider())
    const pending = transport.nextQuery()
    const ack: CommandAck = { accepted: true, job_id: 1, reason: '' }
    const resultPromise = client.request(
      Drain.key,
      'command',
      Drain.requestSchema,
      COMMAND_ACK_SCHEMA,
      {},
    )
    const query = await pending
    expect(query.body?.encoding).toBe(cdrEncoding(Drain.requestSchema))
    query.reply({
      kind: 'sample',
      sample: {
        key: Drain.key,
        payload: encodeCdr(COMMAND_ACK_SCHEMA, ack),
        encoding: cdrEncoding(COMMAND_ACK_SCHEMA),
      },
    })
    const result = await resultPromise
    expect(result).toEqual({ kind: 'ack', value: ack })
  })
})
