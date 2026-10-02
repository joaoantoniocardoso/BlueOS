import { describe, expect, it } from 'vitest'

import {
  commandKey, eventKey, queryKey, stateKey,
} from '@/libs/blueos-api/keys'
import type {
  DrainRequest,
  Emptied as EmptiedMessage,
  LevelAfterFillRequest,
  LevelAfterFillResponse,
  LevelChanged as LevelChangedMessage,
  LevelRequest,
  LevelResponse,
  ProbeRequest,
  ProbeResponse,
  SetLevelRequest,
  Tank,
} from '@/libs/blueos-api/services/tank'
import {
  Drain,
  Emptied,
  Level,
  LevelAfterFill,
  LevelChanged,
  NAME,
  Probe,
  SetLevel,
  tank,
} from '@/libs/blueos-api/services/tank'

describe('tank generated client', () => {
  it('exports keys built from the shared helpers', () => {
    expect(NAME).toBe('tank')
    expect(Drain.key).toBe(commandKey(NAME, 'Drain'))
    expect(SetLevel.key).toBe(commandKey(NAME, 'SetLevel'))
    expect(Level.key).toBe(queryKey(NAME, 'Level'))
    expect(LevelAfterFill.key).toBe(queryKey(NAME, 'LevelAfterFill'))
    expect(Probe.key).toBe(queryKey(NAME, 'Probe'))
    expect(tank.key).toBe(stateKey(NAME, 'tank'))
    expect(Emptied.key).toBe(eventKey(NAME, 'Emptied'))
    expect(LevelChanged.key).toBe(eventKey(NAME, 'LevelChanged'))
  })

  it('request and response types match the IDL messages', () => {
    const drain: DrainRequest = { padding: 0 }
    const setLevel: SetLevelRequest = { level: 3 }
    const levelRequest: LevelRequest = { padding: 0 }
    const levelResponse: LevelResponse = { level: 1, max_level: 10 }
    const afterFillRequest: LevelAfterFillRequest = { level: 5 }
    const afterFillResponse: LevelAfterFillResponse = { level: 5, max_level: 10 }
    const probeRequest: ProbeRequest = { padding: 0 }
    const probeResponse: ProbeResponse = { level: 2, max_level: 10 }
    const tankState: Tank = { level: 4, max_level: 10 }
    const emptied: EmptiedMessage = { padding: 0 }
    const changed: LevelChangedMessage = { level: 6, max_level: 10 }

    expect(drain.padding).toBe(0)
    expect(setLevel.level).toBe(3)
    expect(levelRequest.padding).toBe(0)
    expect(levelResponse.max_level).toBe(10)
    expect(afterFillRequest.level).toBe(5)
    expect(afterFillResponse.level).toBe(5)
    expect(probeRequest.padding).toBe(0)
    expect(probeResponse.level).toBe(2)
    expect(tankState.level).toBe(4)
    expect(emptied.padding).toBe(0)
    expect(changed.level).toBe(6)
  })
})
