import { describe, expect, it } from 'vitest'

import {
  applyRos2Liveliness,
  applySample,
  createInspectorState,
} from '@/libs/zenoh-inspector/logic/registry'
import {
  hasCdrEncapsulationHeader,
  isAmbiguousCdrEncoding,
  payloadIsCdrCandidate,
} from '@/libs/zenoh-inspector/logic/topic-classification'

const CDR_HEADER = new Uint8Array([0x00, 0x01, 0x00, 0x00, 0x0a])

describe('topic classification', () => {
  it('detects CDR encapsulation headers', () => {
    expect(hasCdrEncapsulationHeader(CDR_HEADER)).toBe(true)
    expect(hasCdrEncapsulationHeader(new Uint8Array([0x7b]))).toBe(false)
  })

  it('marks zenoh/bytes, empty and application/cdr as ambiguous encodings', () => {
    expect(isAmbiguousCdrEncoding('zenoh/bytes')).toBe(true)
    expect(isAmbiguousCdrEncoding('')).toBe(true)
    expect(isAmbiguousCdrEncoding('application/cdr')).toBe(true)
    expect(isAmbiguousCdrEncoding('application/json')).toBe(false)
  })

  it('requires a CDR header for ambiguous encodings', () => {
    expect(payloadIsCdrCandidate('zenoh/bytes', CDR_HEADER)).toBe(true)
    expect(payloadIsCdrCandidate('zenoh/bytes', new Uint8Array([0x7b, 0x22]))).toBe(false)
    expect(payloadIsCdrCandidate('application/cdr', CDR_HEADER)).toBe(true)
    expect(payloadIsCdrCandidate('application/cdr', new Uint8Array([0x01]))).toBe(false)
    expect(payloadIsCdrCandidate('', CDR_HEADER)).toBe(true)
  })

  it('keeps samples raw until a ros2dds publisher token arrives', () => {
    let state = createInspectorState()
    state = applySample(state, {
      key: 'chatter',
      payload: CDR_HEADER,
      encoding: 'application/cdr',
      receivedAt: 0,
      kind: 'put',
    })
    expect(state.topics.chatter.source).toBe('raw')
    state = applyRos2Liveliness(
      state,
      '@/aac3178e146ba6f1fc6e6a4085e77f21/@ros2_lv/MP/chatter/std_msgs\u00a7msg\u00a7String',
      true,
    )
    expect(state.topics.chatter.source).toBe('ros2dds')
  })
})
