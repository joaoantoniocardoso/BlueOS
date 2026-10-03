import type { EndpointInfo } from '@blueos-idl/messages'
import { describe, expect, it } from 'vitest'

import { endpointsByKind, endpointSchemas, parseRequestText } from '@/libs/zenoh-inspector/logic/forms'

const START: EndpointInfo = {
  kind: 'job',
  name: 'StartRecording',
  key: 'blueos/v1/recorder/command/StartRecording',
  interface_type: 'blueos_recorder_msgs/action/StartRecording',
  schema: '',
}

const STATUS: EndpointInfo = {
  kind: 'state',
  name: 'status',
  key: 'blueos/v1/recorder/state/status',
  interface_type: 'blueos_msgs/msg/ServiceStatus',
  schema: '',
}

describe('forms', () => {
  it('parses request JSON objects', () => {
    expect(parseRequestText('{"x":1}')).toEqual({ ok: true, value: { x: 1 } })
    expect(parseRequestText('[]').ok).toBe(false)
    expect(parseRequestText('  ')).toEqual({ ok: true, value: {} })
  })

  it('groups endpoints by kind', () => {
    const grouped = endpointsByKind([START, STATUS])
    expect(grouped.job).toHaveLength(1)
    expect(grouped.state).toHaveLength(1)
  })

  it('names the parts of the interface type each kind sends and reads', () => {
    const index: EndpointInfo = {
      kind: 'query',
      name: 'index',
      key: 'blueos/v1/recorder/query/index',
      interface_type: 'blueos_recorder_msgs/srv/RecordingIndex',
      schema: '',
    }

    expect(endpointSchemas(START)).toEqual({
      requestSchema: 'blueos_recorder_msgs/action/StartRecording_Goal',
      responseSchema: 'blueos_msgs/msg/CommandAck',
    })
    expect(endpointSchemas(index)).toEqual({
      requestSchema: 'blueos_recorder_msgs/srv/RecordingIndex_Request',
      responseSchema: 'blueos_recorder_msgs/srv/RecordingIndex_Response',
    })
    expect(endpointSchemas(STATUS)).toEqual({ requestSchema: '', responseSchema: 'blueos_msgs/msg/ServiceStatus' })
  })
})
