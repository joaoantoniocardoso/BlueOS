import { describe, expect, it } from 'vitest'

import {
  applyBlueosServiceLiveliness,
  applyRos2Liveliness,
  applySample,
  createInspectorState,
  sortedTopicKeys,
  topicsBySource,
} from '@/libs/zenoh-inspector/logic/registry'

describe('inspector registry', () => {
  it('tracks samples and liveliness on null-prototype maps', () => {
    let state = createInspectorState()
    expect(Object.getPrototypeOf(state.topics)).toBeNull()

    state = applySample(state, {
      key: 'blueos/v1/recorder/state/library',
      payload: new Uint8Array([1]),
      encoding: 'application/json',
      receivedAt: 1_700_000_000_000,
      kind: 'put',
    })
    expect(state.topics['blueos/v1/recorder/state/library'].sampleCount).toBe(1)
    expect(state.topics['blueos/v1/recorder/state/library'].source).toBe('blueos')

    state = applyBlueosServiceLiveliness(state, 'recorder', true)
    expect(state.topics['blueos/v1/services/recorder'].alive).toBe(true)
  })

  it('lists the log of an alive service before its first log line', () => {
    let state = createInspectorState()
    state = applyBlueosServiceLiveliness(state, 'recorder', true)
    expect(sortedTopicKeys(state)).toEqual(['blueos/v1/recorder/log', 'blueos/v1/services/recorder'])
    expect(state.topics['blueos/v1/recorder/log'].blueos?.kind).toBe('log')
    expect(state.topics['blueos/v1/recorder/log'].sampleCount).toBe(0)

    state = applySample(state, {
      key: 'blueos/v1/recorder/log',
      payload: new Uint8Array([0, 1, 0, 0]),
      encoding: 'application/cdr;foxglove_msgs/msg/Log',
      receivedAt: 0,
      kind: 'put',
    })
    state = applyBlueosServiceLiveliness(state, 'recorder', true)
    expect(state.topics['blueos/v1/recorder/log'].sampleCount).toBe(1)
  })

  it('lists ros2dds subscriber tokens as entity-only entries', () => {
    const token = '@/zenoh/@ros2_lv/MS/robot1\u00a7camera\u00a7image_raw/sensor_msgs\u00a7msg\u00a7Image'
    let state = createInspectorState()
    state = applyRos2Liveliness(state, token, true)
    expect(state.topics['robot1/camera/image_raw']).toBeUndefined()
    const entityKey = Object.keys(state.topics).find((key) => key.startsWith('@entity/'))
    expect(entityKey).toBeDefined()
    expect(state.topics[entityKey!].entityOnly).toBe(true)
    expect(state.topics[entityKey!].ros2?.entityKind).toBe('subscriber')
  })

  it('keeps rmw_zenoh entity-only topics until data arrives', () => {
    const token = [
      '@ros2_lv/0/aac3178e146ba6f1fc6e6a4085e77f21/0/11/MP/%/%/talker/%chatter',
      '/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18',
      '/::,7:,:,:,,',
    ].join('')
    let state = createInspectorState()
    state = applyRos2Liveliness(state, token, true)
    expect(sortedTopicKeys(state)).toEqual(['/chatter'])
    expect(state.topics['/chatter'].entityOnly).toBe(true)
  })

  it('sorts topics and groups by source', () => {
    let state = createInspectorState()
    state = applyRos2Liveliness(state, '@/zenoh/@ros2_lv/MP/chatter/std_msgs\u00a7msg\u00a7String', true)
    state = applySample(state, {
      key: 'mavlink/1/1/AHRS',
      payload: new Uint8Array(),
      encoding: 'application/json',
      receivedAt: 0,
      kind: 'put',
    })
    state = applySample(state, {
      key: 'chatter',
      payload: new Uint8Array(),
      encoding: 'application/cdr',
      receivedAt: 0,
      kind: 'put',
    })
    state = applySample(state, {
      key: 'blueos/v1/recorder/state/status',
      payload: new Uint8Array(),
      encoding: 'application/json',
      receivedAt: 0,
      kind: 'put',
    })
    expect(sortedTopicKeys(state)).toEqual([
      'blueos/v1/recorder/state/status',
      'chatter',
      'mavlink/1/1/AHRS',
    ])
    const groups = topicsBySource(state)
    expect(groups.some((group) => group.source === 'blueos')).toBe(true)
    expect(state.topics.chatter.source).toBe('ros2dds')
    expect(state.topics['mavlink/1/1/AHRS'].source).toBe('raw')
  })
})
