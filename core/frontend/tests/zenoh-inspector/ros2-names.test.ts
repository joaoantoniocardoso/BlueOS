import { readFileSync } from 'fs'
import path from 'path'
import { describe, expect, it } from 'vitest'

import {
  ddsToRosTypeName,
  parseRmwZenohDataKey,
  parseRmwZenohToken,
  parseRos2ddsToken,
  ros2ddsTopicFromDataKey,
} from '@/libs/zenoh-inspector/logic/ros2-names'
import type { Ros2Info } from '@/libs/zenoh-inspector/logic/types'

const vectorsPath = path.resolve(
  __dirname,
  '../../../libs/logic/ros2-names/tests/vectors/names.json',
)

type VectorFile = {
  rmw_zenoh_data_keys: { key: string, expected: Record<string, unknown> | null }[]
  rmw_zenoh_tokens: { key: string, expected: Record<string, unknown> | null }[]
  ros2dds_tokens: { key: string, expected: Record<string, unknown> | null }[]
  dds_type_names: { dds: string, ros: string | null }[]
  ros2dds_data_key_topics: { key: string, topic: string }[]
}

const vectors = JSON.parse(readFileSync(vectorsPath, 'utf8')) as VectorFile

// Only keys listed in `expected` are compared; extra fields on the parse result are ignored.
function expectPartial(actual: Ros2Info | undefined, expected: Record<string, unknown> | null): void {
  if (expected === null) {
    expect(actual).toBeUndefined()
    return
  }
  expect(actual).toBeDefined()
  for (const [field, value] of Object.entries(expected)) {
    expect(actual?.[field as keyof Ros2Info]).toEqual(value)
  }
}

describe('ros2-names vectors', () => {
  it('parses rmw_zenoh data keys', () => {
    for (const entry of vectors.rmw_zenoh_data_keys) {
      expectPartial(parseRmwZenohDataKey(entry.key), entry.expected)
    }
  })

  it('parses rmw_zenoh liveliness tokens', () => {
    for (const entry of vectors.rmw_zenoh_tokens) {
      expectPartial(parseRmwZenohToken(entry.key), entry.expected)
    }
  })

  it('parses ros2dds liveliness tokens', () => {
    for (const entry of vectors.ros2dds_tokens) {
      expectPartial(parseRos2ddsToken(entry.key), entry.expected)
    }
  })

  it('maps DDS type names to ROS types', () => {
    for (const entry of vectors.dds_type_names) {
      const ros = ddsToRosTypeName(entry.dds)
      if (entry.ros === null) {
        expect(ros).toBeUndefined()
      } else {
        expect(ros).toBe(entry.ros)
      }
    }
  })

  it('maps ros2dds data keys to ROS topic names', () => {
    for (const entry of vectors.ros2dds_data_key_topics) {
      expect(ros2ddsTopicFromDataKey(entry.key)).toBe(entry.topic)
    }
  })
})
