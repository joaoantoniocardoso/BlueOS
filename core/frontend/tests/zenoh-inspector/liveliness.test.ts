import { describe, expect, it } from 'vitest'

import {
  applyRos2Liveliness,
  applySample,
  createInspectorState,
} from '@/libs/zenoh-inspector/logic/registry'
import type { InspectorState } from '@/libs/zenoh-inspector/logic/types'

const RMW_CHATTER_HASH = 'RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18'

const RMW_CHATTER_KEY = [
  '0/chatter/std_msgs::msg::dds_::String_',
  RMW_CHATTER_HASH,
].join('/')

const RMW_PUBLISHER_TOKEN = [
  '@ros2_lv/0/aac3178e146ba6f1fc6e6a4085e77f21/0/11/MP/%/%/talker/%chatter',
  '/std_msgs::msg::dds_::String_',
  `/${RMW_CHATTER_HASH}/::,7:,:,:,,`,
].join('')

const RMW_PUBLISHER_TOKEN_B = [
  '@ros2_lv/0/aac3178e146ba6f1fc6e6a4085e77f21/0/14/MP/%/%/talker2/%chatter',
  '/std_msgs::msg::dds_::String_',
  `/${RMW_CHATTER_HASH}/::,7:,:,:,,`,
].join('')

const RMW_SUBSCRIBER_TOKEN = [
  '@ros2_lv/0/aac3178e146ba6f1fc6e6a4085e77f21/0/12/MS/%/%/listener/%chatter',
  '/std_msgs::msg::dds_::String_',
  `/${RMW_CHATTER_HASH}/::,7:,:,:,,`,
].join('')

const ROS2DDS_PUBLISHER_TOKEN = [
  '@/aac3178e146ba6f1fc6e6a4085e77f21/@ros2_lv/MP/chatter/',
  'std_msgs\u00a7msg\u00a7String',
].join('')

const ROS2DDS_SUBSCRIBER_TOKEN = [
  '@/aac3178e146ba6f1fc6e6a4085e77f21/@ros2_lv/MS/chatter/',
  'std_msgs\u00a7msg\u00a7String',
].join('')

const ROS2DDS_SERVICE_TOKEN = [
  '@/aac3178e146ba6f1fc6e6a4085e77f21/@ros2_lv/SS/add_two_ints/',
  'example_interfaces\u00a7srv\u00a7AddTwoInts',
].join('')

function applyDataSample(state: InspectorState): InspectorState {
  return applySample(state, {
    key: RMW_CHATTER_KEY,
    payload: new Uint8Array([0]),
    encoding: 'zenoh/bytes',
    receivedAt: 0,
    kind: 'put',
  })
}

describe('ROS 2 liveliness on data topics', () => {
  it('keeps publisher entity kind when a subscriber token arrives (publisher first)', () => {
    let state = applyDataSample(createInspectorState())
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, true)
    state = applyRos2Liveliness(state, RMW_SUBSCRIBER_TOKEN, true)
    expect(state.topics[RMW_CHATTER_KEY].ros2?.entityKind).toBe('publisher')
    expect(state.topics[RMW_CHATTER_KEY].alive).toBe(true)
  })

  it('keeps publisher entity kind when a subscriber token arrives (subscriber first)', () => {
    let state = applyDataSample(createInspectorState())
    state = applyRos2Liveliness(state, RMW_SUBSCRIBER_TOKEN, true)
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, true)
    expect(state.topics[RMW_CHATTER_KEY].ros2?.entityKind).toBe('publisher')
    expect(state.topics[RMW_CHATTER_KEY].alive).toBe(true)
  })

  it('stays alive when only the subscriber token drops', () => {
    let state = applyDataSample(createInspectorState())
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, true)
    state = applyRos2Liveliness(state, RMW_SUBSCRIBER_TOKEN, true)
    state = applyRos2Liveliness(state, RMW_SUBSCRIBER_TOKEN, false)
    expect(state.topics[RMW_CHATTER_KEY].alive).toBe(true)
    expect(state.topics[RMW_CHATTER_KEY].ros2?.entityKind).toBe('publisher')
  })

  it('dies when the last publisher token drops', () => {
    let state = applyDataSample(createInspectorState())
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, true)
    state = applyRos2Liveliness(state, RMW_SUBSCRIBER_TOKEN, true)
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, false)
    expect(state.topics[RMW_CHATTER_KEY].alive).toBe(false)
  })

  it('does not let a ros2dds subscriber token modify an existing data topic', () => {
    let state = createInspectorState()
    state = applySample(state, {
      key: 'chatter',
      payload: new Uint8Array([0]),
      encoding: 'zenoh/bytes',
      receivedAt: 0,
      kind: 'put',
    })
    state = applyRos2Liveliness(state, ROS2DDS_PUBLISHER_TOKEN, true)
    const beforeSubscriber = state.topics.chatter
    state = applyRos2Liveliness(state, ROS2DDS_SUBSCRIBER_TOKEN, true)
    expect(state.topics.chatter).toEqual(beforeSubscriber)
    expect(state.topics.chatter.ros2?.entityKind).toBe('publisher')
    state = applyRos2Liveliness(state, ROS2DDS_SUBSCRIBER_TOKEN, false)
    expect(state.topics.chatter.alive).toBe(true)
  })

  it('does not create a ros2dds data topic for service tokens', () => {
    let state = createInspectorState()
    state = applyRos2Liveliness(state, ROS2DDS_SERVICE_TOKEN, true)
    expect(state.topics.add_two_ints).toBeUndefined()
    expect(Object.keys(state.topics).some((key) => key.startsWith('@entity/'))).toBe(true)
  })

  it('treats duplicate publisher token delivery as idempotent', () => {
    let state = applyDataSample(createInspectorState())
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, true)
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, true)
    expect(state.topics[RMW_CHATTER_KEY].publisherTokens).toEqual([RMW_PUBLISHER_TOKEN])
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, false)
    expect(state.topics[RMW_CHATTER_KEY].alive).toBe(false)
    expect(state.topics[RMW_CHATTER_KEY].publisherTokens).toEqual([])
  })

  it('stays alive while another publisher token remains', () => {
    let state = applyDataSample(createInspectorState())
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, true)
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN_B, true)
    state = applyRos2Liveliness(state, RMW_PUBLISHER_TOKEN, false)
    expect(state.topics[RMW_CHATTER_KEY].alive).toBe(true)
    expect(state.topics[RMW_CHATTER_KEY].publisherTokens).toEqual([RMW_PUBLISHER_TOKEN_B])
  })
})
