/* eslint-disable import/no-extraneous-dependencies */
import { CATALOG_SCHEMAS } from '@blueos-idl/catalog'
import { describe, expect, it } from 'vitest'

import {
  decodeCdrWithSchema,
  defaultMessageForSchema,
  encodeCdrWithSchema,
} from '@/libs/blueos-api/cdr'
import { decodePayload } from '@/libs/zenoh-inspector/logic/decode'
import {
  applyRos2Liveliness,
  applySample,
  createInspectorState,
  sortedTopicKeys,
} from '@/libs/zenoh-inspector/logic/registry'
import type { CdrCodec, SchemaProvider } from '@/libs/zenoh-inspector/logic/types'

const STD_STRING_SCHEMA = 'std_msgs/msg/String'
const STD_STRING_TEXT = CATALOG_SCHEMAS[STD_STRING_SCHEMA]

const RMW_CHATTER_HASH = 'RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18'

const RMW_CHATTER_KEY = [
  '0/chatter/std_msgs::msg::dds_::String_',
  RMW_CHATTER_HASH,
].join('/')
const RMW_CHATTER_TOKEN = [
  '@ros2_lv/0/aac3178e146ba6f1fc6e6a4085e77f21/0/11/MP/%/%/talker/%chatter',
  '/std_msgs::msg::dds_::String_',
  `/${RMW_CHATTER_HASH}/::,7:,:,:,,`,
].join('')

const ROS2DDS_CHATTER_TOKEN = [
  '@/aac3178e146ba6f1fc6e6a4085e77f21/@ros2_lv/MP/chatter/',
  'std_msgs\u00a7msg\u00a7String',
].join('')

const catalogProvider: SchemaProvider = {
  schemaText(schemaName: string): string | undefined {
    return CATALOG_SCHEMAS[schemaName as keyof typeof CATALOG_SCHEMAS]
  },
}

const catalogCodec: CdrCodec = {
  decode(schemaName, schemaText, payload) {
    return decodeCdrWithSchema(schemaName, schemaText, payload) as Record<string, unknown>
  },
  encode(schemaName, schemaText, message) {
    return encodeCdrWithSchema(schemaName, schemaText, message)
  },
  defaults(schemaName, schemaText) {
    return defaultMessageForSchema(schemaName, schemaText) as Record<string, unknown>
  },
}

function stringCdrPayload(data: string): Uint8Array {
  return encodeCdrWithSchema(STD_STRING_SCHEMA, STD_STRING_TEXT, { data })
}

describe('ROS 2 payload decode and registry join', () => {
  it('decodes rmw_zenoh zenoh/bytes from the data key type', () => {
    const payload = stringCdrPayload('rmw sample')
    let state = createInspectorState()
    state = applySample(state, {
      key: RMW_CHATTER_KEY,
      payload,
      encoding: 'zenoh/bytes',
      receivedAt: 0,
      kind: 'put',
    })
    const topic = state.topics[RMW_CHATTER_KEY]
    const decoded = decodePayload(topic, undefined, catalogProvider, catalogCodec)
    expect(decoded).toEqual({
      kind: 'cdr',
      schemaName: STD_STRING_SCHEMA,
      value: { data: 'rmw sample' },
    })
  })

  it('decodes ros2dds zenoh/bytes when the token arrives before the sample', () => {
    const payload = stringCdrPayload('token first')
    let state = createInspectorState()
    state = applyRos2Liveliness(state, ROS2DDS_CHATTER_TOKEN, true)
    state = applySample(state, {
      key: 'chatter',
      payload,
      encoding: 'zenoh/bytes',
      receivedAt: 0,
      kind: 'put',
    })
    const topic = state.topics.chatter
    expect(topic.ros2?.typeName).toBe(STD_STRING_SCHEMA)
    const decoded = decodePayload(topic, undefined, catalogProvider, catalogCodec)
    expect(decoded).toEqual({
      kind: 'cdr',
      schemaName: STD_STRING_SCHEMA,
      value: { data: 'token first' },
    })
  })

  it('decodes ros2dds zenoh/bytes when the token arrives after the sample', () => {
    const payload = stringCdrPayload('sample first')
    let state = createInspectorState()
    state = applySample(state, {
      key: 'chatter',
      payload,
      encoding: 'zenoh/bytes',
      receivedAt: 0,
      kind: 'put',
    })
    state = applyRos2Liveliness(state, ROS2DDS_CHATTER_TOKEN, true)
    const topic = state.topics.chatter
    const decoded = decodePayload(topic, undefined, catalogProvider, catalogCodec)
    expect(decoded).toEqual({
      kind: 'cdr',
      schemaName: STD_STRING_SCHEMA,
      value: { data: 'sample first' },
    })
  })

  it('still decodes zenoh/bytes JSON when the payload is not CDR', () => {
    const topic = {
      key: 'custom',
      source: 'raw' as const,
      sampleCount: 1,
      lastSample: {
        key: 'custom',
        payload: new TextEncoder().encode('{"hello":"world"}'),
        encoding: 'zenoh/bytes',
        receivedAt: 0,
        kind: 'put' as const,
      },
    }
    const decoded = decodePayload(topic, undefined, catalogProvider, catalogCodec)
    expect(decoded).toEqual({ kind: 'json', value: { hello: 'world' } })
  })

  it('joins rmw_zenoh liveliness to data keys when the token arrives first', () => {
    let state = createInspectorState()
    state = applyRos2Liveliness(state, RMW_CHATTER_TOKEN, true)
    expect(state.topics['/chatter']?.entityOnly).toBe(true)
    state = applySample(state, {
      key: RMW_CHATTER_KEY,
      payload: stringCdrPayload('joined'),
      encoding: 'zenoh/bytes',
      receivedAt: 0,
      kind: 'put',
    })
    expect(state.topics['/chatter']).toBeUndefined()
    expect(state.topics[RMW_CHATTER_KEY].alive).toBe(true)
    expect(state.topics[RMW_CHATTER_KEY].ros2?.entityKind).toBe('publisher')
    expect(state.topics[RMW_CHATTER_KEY].ros2?.node).toBe('talker')
    expect(sortedTopicKeys(state).filter((key) => key.includes('chatter'))).toHaveLength(1)
  })

  it('joins rmw_zenoh liveliness to data keys when the sample arrives first', () => {
    let state = createInspectorState()
    state = applySample(state, {
      key: RMW_CHATTER_KEY,
      payload: stringCdrPayload('joined'),
      encoding: 'zenoh/bytes',
      receivedAt: 0,
      kind: 'put',
    })
    state = applyRos2Liveliness(state, RMW_CHATTER_TOKEN, true)
    expect(state.topics['/chatter']).toBeUndefined()
    expect(state.topics[RMW_CHATTER_KEY].alive).toBe(true)
    expect(state.topics[RMW_CHATTER_KEY].ros2?.entityKind).toBe('publisher')
    expect(sortedTopicKeys(state).filter((key) => key.includes('chatter'))).toHaveLength(1)
  })
})
