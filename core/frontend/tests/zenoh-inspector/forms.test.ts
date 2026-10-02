import type { EndpointInfo } from '@blueos-idl/messages'
import { describe, expect, it } from 'vitest'

import { endpointsByKind, parseRequestText } from '@/libs/zenoh-inspector/logic/forms'

describe('forms', () => {
  it('parses request JSON objects', () => {
    expect(parseRequestText('{"x":1}')).toEqual({ ok: true, value: { x: 1 } })
    expect(parseRequestText('[]').ok).toBe(false)
    expect(parseRequestText('  ')).toEqual({ ok: true, value: {} })
  })

  it('groups endpoints by kind', () => {
    const endpoints: EndpointInfo[] = [
      {
        kind: 'command',
        name: 'start',
        key: 'blueos/v1/recorder/command/start',
        request_schema: 'blueos_msgs/srv/Start',
        response_schema: 'blueos_msgs/srv/Start',
      },
      {
        kind: 'state',
        name: 'status',
        key: 'blueos/v1/recorder/state/status',
        request_schema: '',
        response_schema: 'blueos_msgs/msg/ServiceStatus',
      },
    ]
    const grouped = endpointsByKind(endpoints)
    expect(grouped.command).toHaveLength(1)
    expect(grouped.state).toHaveLength(1)
  })
})
