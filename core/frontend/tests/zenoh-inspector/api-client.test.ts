/* eslint-disable import/no-extraneous-dependencies */
import { CommandAckStatus } from '@blueos-idl/constants'
import type { CommandAck, ServiceInfo } from '@blueos-idl/messages'
import { SCHEMAS } from '@blueos-idl/schemas'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding, infoQueryKey } from '@/libs/blueos-api/keys'
import { SetLevel } from '@/libs/blueos-api/services/example'
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
        kind: 'job',
        name: 'SetLevel',
        key: 'blueos/v1/tank/command/SetLevel',
        interface_type: 'blueos_example_msgs/action/SetLevel',
        schema: 'uint8 level\n---\nuint8 level\n---\nuint8 level\n',
      },
      {
        kind: 'query',
        name: 'Level',
        key: 'blueos/v1/tank/query/Level',
        interface_type: 'blueos_example_msgs/srv/Level',
        schema: '---\nuint8 level\nuint8 max_level\n',
      },
      {
        kind: 'state',
        name: 'pump',
        key: 'blueos/v1/tank/state/pump',
        interface_type: 'blueos_example_msgs/msg/PumpState',
        schema: SCHEMAS['blueos_example_msgs/msg/PumpState'],
      },
      {
        kind: 'event',
        name: 'SetLevel/result',
        key: 'blueos/v1/tank/event/jobs/SetLevel/result',
        interface_type: 'blueos_msgs/msg/JobResult',
        schema: SCHEMAS['blueos_msgs/msg/JobResult'],
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

  it('submits a Goal to a Job type and decodes its CommandAck', async () => {
    const transport = new FakeTransport()
    const client = createInspectorApiClient(async () => transport, idlSchemaProvider())
    const pending = transport.nextQuery()
    const ack: CommandAck = {
      accepted: true, job_id: '0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10', status: CommandAckStatus.Succeeded, reason: '',
    }
    const resultPromise = client.request(
      SetLevel.key,
      'job',
      SetLevel.goalSchema,
      COMMAND_ACK_SCHEMA,
      {},
    )
    const query = await pending
    expect(query.body?.encoding).toBe(cdrEncoding(SetLevel.goalSchema))
    expect(new TextDecoder().decode(query.body?.attachment)).toMatch(/^[0-9a-f-]{36}$/)
    query.reply({
      kind: 'sample',
      sample: {
        key: SetLevel.key,
        payload: encodeCdr(COMMAND_ACK_SCHEMA, ack),
        encoding: cdrEncoding(COMMAND_ACK_SCHEMA),
      },
    })
    const result = await resultPromise
    expect(result).toEqual({ kind: 'ack', value: ack })
  })
})
