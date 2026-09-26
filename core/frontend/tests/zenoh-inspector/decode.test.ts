import { describe, expect, it } from 'vitest'

import { decodePayload, formatTopicJson, toDisplayValue } from '@/libs/zenoh-inspector/logic/decode'
import type { CdrCodec, SchemaProvider, TopicInfo } from '@/libs/zenoh-inspector/logic/types'

const provider: SchemaProvider = {
  schemaText(): string | undefined {
    return 'string data'
  },
}

const codec: CdrCodec = {
  decode(): Record<string, unknown> {
    return { data: 'hello' }
  },
  encode(): Uint8Array {
    return new Uint8Array()
  },
  defaults(): Record<string, unknown> {
    return { data: '' }
  },
}

function topicWithSample(
  sample: TopicInfo['lastSample'],
  overrides: Partial<TopicInfo> = {},
): TopicInfo {
  return {
    key: sample?.key ?? 'k',
    source: 'raw',
    sampleCount: 1,
    lastSample: sample,
    ...overrides,
  }
}

describe('decode', () => {
  it('decodes text and json payloads', () => {
    const text = decodePayload(
      topicWithSample({
        key: 'k',
        payload: new TextEncoder().encode('plain'),
        encoding: 'text/plain',
        receivedAt: 0,
        kind: 'put',
      }),
      undefined,
      provider,
      codec,
    )
    expect(text).toEqual({ kind: 'text', value: 'plain' })

    const json = decodePayload(
      topicWithSample({
        key: 'k',
        payload: new TextEncoder().encode('{"a":1}'),
        encoding: 'application/json',
        receivedAt: 0,
        kind: 'put',
      }),
      undefined,
      provider,
      codec,
    )
    expect(json).toEqual({ kind: 'json', value: { a: 1 } })
  })

  it('summarizes large numeric arrays for display', () => {
    const numbers = Array.from({ length: 100 }, (_, index) => index)
    const displayed = toDisplayValue(numbers) as { bytes: number, preview: string }
    expect(displayed.bytes).toBe(100)
    expect(displayed.preview.length).toBe(64)
  })

  it('formats topic JSON like the legacy inspector', () => {
    const topic: TopicInfo = {
      key: 'chatter',
      source: 'ros2dds',
      sampleCount: 1,
      alive: true,
      schemaName: 'std_msgs/msg/String',
      lastSample: {
        key: 'chatter',
        payload: new Uint8Array(),
        encoding: 'text/plain',
        receivedAt: Date.parse('2024-01-01T00:00:00.000Z'),
        kind: 'put',
      },
    }
    const formatted = formatTopicJson(topic, { kind: 'text', value: 'hi' })
    const parsed = JSON.parse(formatted) as Record<string, unknown>
    expect(parsed.topic).toBe('chatter')
    expect(parsed.liveliness).toBe('Alive')
    expect(parsed.payload).toBe('hi')

    const cdr = formatTopicJson({ ...topic, schemaName: undefined }, {
      kind: 'cdr',
      schemaName: 'blueos_msgs/msg/ServiceInfo',
      value: {},
    })
    expect((JSON.parse(cdr) as Record<string, unknown>).schema).toBe('blueos_msgs/msg/ServiceInfo')
  })
})
