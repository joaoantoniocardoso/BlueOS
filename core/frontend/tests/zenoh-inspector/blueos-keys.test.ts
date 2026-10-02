import { describe, expect, it } from 'vitest'

import { parseBlueosKey } from '@/libs/zenoh-inspector/logic/blueos-keys'

describe('parseBlueosKey', () => {
  it('parses command and state keys', () => {
    expect(parseBlueosKey('blueos/v1/recorder/command/start')).toEqual({
      service: 'recorder',
      kind: 'command',
      name: 'start',
    })
    expect(parseBlueosKey('blueos/v1/recorder/state/library')).toEqual({
      service: 'recorder',
      kind: 'state',
      name: 'library',
    })
  })

  it('parses service liveliness and http gateway keys', () => {
    expect(parseBlueosKey('blueos/v1/services/recorder')).toEqual({
      service: 'recorder',
      kind: 'service_liveliness',
      name: '',
    })
    expect(parseBlueosKey('blueos/v1/recorder/http/api/status')).toEqual({
      service: 'recorder',
      kind: 'http',
      name: 'api/status',
    })
  })

  it('returns undefined for non-BlueOS keys', () => {
    expect(parseBlueosKey('chatter')).toBeUndefined()
    expect(parseBlueosKey('blueos/v2/recorder/state/x')).toBeUndefined()
  })
})
