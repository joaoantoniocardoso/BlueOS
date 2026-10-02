import { describe, expect, it } from 'vitest'

import {
  FOXGLOVE_LOG_SCHEMA,
  resolveSchemaName,
  schemaNameFromEncoding,
} from '@/libs/zenoh-inspector/logic/schema-resolution'
import type { SchemaProvider, TopicInfo } from '@/libs/zenoh-inspector/logic/types'

const provider: SchemaProvider = {
  schemaText(schemaName: string): string | undefined {
    if (schemaName === 'std_msgs/msg/String') {
      return 'string data'
    }
    if (schemaName === FOXGLOVE_LOG_SCHEMA) {
      return 'string message'
    }
    return undefined
  },
}

describe('schema resolution', () => {
  it('prefers encoding suffix when the provider has the schema', () => {
    const topic: TopicInfo = {
      key: 'blueos/v1/example/state/x',
      source: 'blueos',
      sampleCount: 0,
      lastSample: {
        key: 'blueos/v1/example/state/x',
        payload: new Uint8Array(),
        encoding: 'application/cdr;std_msgs/msg/String',
        receivedAt: 0,
        kind: 'put',
      },
    }
    expect(schemaNameFromEncoding('application/cdr;std_msgs/msg/String')).toBe('std_msgs/msg/String')
    expect(resolveSchemaName(topic, provider)).toBe('std_msgs/msg/String')
  })

  it('falls back to transport type names', () => {
    const rmwKey = [
      '0/chatter/std_msgs::msg::dds_::String_',
      'RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18',
    ].join('/')
    const topic: TopicInfo = {
      key: rmwKey,
      source: 'rmw_zenoh',
      sampleCount: 0,
      ros2: {
        transport: 'rmw_zenoh',
        entityKind: 'publisher',
        topic: '/chatter',
        typeName: 'std_msgs/msg/String',
      },
    }
    expect(resolveSchemaName(topic, provider)).toBe('std_msgs/msg/String')
  })

  it('resolves BlueOS service log keys to Foxglove Log', () => {
    const topic: TopicInfo = {
      key: 'blueos/v1/tank/log',
      source: 'blueos',
      sampleCount: 1,
      blueos: { service: 'tank', kind: 'log', name: '' },
      lastSample: {
        key: 'blueos/v1/tank/log',
        payload: new Uint8Array([0, 1, 0, 0, 0, 0, 0, 0]),
        encoding: 'application/cdr',
        receivedAt: 0,
        kind: 'put',
      },
    }
    expect(resolveSchemaName(topic, provider)).toBe(FOXGLOVE_LOG_SCHEMA)
  })

  it('resolves extension log keys under log/extension', () => {
    const topic: TopicInfo = {
      key: 'blueos/v1/kraken/log/extension/my_ext',
      source: 'blueos',
      sampleCount: 1,
      blueos: { service: 'kraken', kind: 'log', name: 'extension/my_ext' },
    }
    expect(resolveSchemaName(topic, provider)).toBe(FOXGLOVE_LOG_SCHEMA)
  })
})
