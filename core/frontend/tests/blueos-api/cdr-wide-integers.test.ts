import { describe, expect, it } from 'vitest'

import {
  decodeCdr, decodeCdrWithSchema, encodeCdr, encodeCdrWithSchema,
} from '@/libs/blueos-api/cdr'

const SCHEMA_NAME = 'test_msgs/msg/Wide'
const SCHEMA_TEXT = `string label
int64 signed
uint64[] counts
float64[] weights
`

describe('blueos-api CDR 64-bit integers', () => {
  it('round-trips an int64 and a uint64[] given as numbers', () => {
    const message = {
      label: 'wide',
      signed: -42,
      counts: [0, 1, 4096],
      weights: [0.5, 1.5],
    }

    const decoded = decodeCdrWithSchema(
      SCHEMA_NAME,
      SCHEMA_TEXT,
      encodeCdrWithSchema(SCHEMA_NAME, SCHEMA_TEXT, message),
    )

    expect(decoded).toEqual(message)
  })

  it('round-trips the uint64[] of a nested message', () => {
    const metrics = {
      counters: [{ name: 'commands_handled', labels: [], value: 7 }],
      gauges: [],
      histograms: [{
        name: 'inbox_step_seconds',
        labels: [],
        count: 6,
        sum: 0.25,
        bucket_bounds: [0.001, 0.01],
        bucket_counts: [1, 2, 3],
      }],
    }

    const decoded = decodeCdr('blueos_msgs/msg/ServiceMetrics', encodeCdr('blueos_msgs/msg/ServiceMetrics', metrics))

    expect(decoded).toEqual(metrics)
  })
})
