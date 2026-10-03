import { describe, expect, it } from 'vitest'

import {
  dateFilterOptions,
  filterRecordings,
  NO_RECORDING_FILTERS,
  utcCalendarDay,
} from '@/libs/recorder/filter'
import type { LibraryRecording } from '@/libs/recorder/types'

const DAY_ONE = Date.UTC(2024, 0, 1, 23, 59, 59) / 1000
const DAY_TWO = Date.UTC(2024, 0, 2, 0, 0, 1) / 1000

function file(overrides: Partial<LibraryRecording> = {}): LibraryRecording {
  return {
    path: 'recording.mcap',
    name: 'recording.mcap',
    size_bytes: 1000,
    created: DAY_ONE,
    state: 'ready',
    repair_bytes_processed: 0,
    repair_total_bytes: 0,
    repair_bytes_per_second: 0,
    repair_error: '',
    allowed_operations: [],
    ...overrides,
  }
}

const recordings = [
  file({ path: 'dock-dive.mcap', name: 'dock-dive.mcap', created: DAY_ONE, state: 'ready' }),
  file({ path: 'reef-survey.mcap', name: 'reef-survey.mcap', created: DAY_TWO, state: 'ready' }),
  file({ path: 'reef-night.mcap', name: 'reef-night.mcap', created: DAY_TWO, state: 'needs_repair' }),
  file({ path: 'Pool-Test.mcap', name: 'Pool-Test.mcap', created: DAY_ONE, state: 'recording' }),
]

function names(filtered: LibraryRecording[]): string[] {
  return filtered.map((entry) => entry.name)
}

describe('recorder filter', () => {
  it('keeps every recording when no filter is set', () => {
    expect(filterRecordings(recordings, NO_RECORDING_FILTERS)).toEqual(recordings)
  })

  it('searches by name ignoring case and surrounding spaces', () => {
    expect(names(filterRecordings(recordings, { ...NO_RECORDING_FILTERS, search: '  POOL ' })))
      .toEqual(['Pool-Test.mcap'])
    expect(names(filterRecordings(recordings, { ...NO_RECORDING_FILTERS, search: 'reef' })))
      .toEqual(['reef-survey.mcap', 'reef-night.mcap'])
    expect(filterRecordings(recordings, { ...NO_RECORDING_FILTERS, search: 'missing' })).toEqual([])
  })

  it('filters by recording state', () => {
    expect(names(filterRecordings(recordings, { ...NO_RECORDING_FILTERS, state: 'needs_repair' })))
      .toEqual(['reef-night.mcap'])
    expect(filterRecordings(recordings, { ...NO_RECORDING_FILTERS, state: 'repairing' })).toEqual([])
  })

  it('filters by UTC calendar day', () => {
    expect(names(filterRecordings(recordings, { ...NO_RECORDING_FILTERS, date: '2024-01-02' })))
      .toEqual(['reef-survey.mcap', 'reef-night.mcap'])
  })

  it('combines search, state and date', () => {
    const filters = { search: 'reef', state: 'ready' as const, date: '2024-01-02' }
    expect(names(filterRecordings(recordings, filters))).toEqual(['reef-survey.mcap'])
    expect(filterRecordings(recordings, { ...filters, date: '2024-01-01' })).toEqual([])
  })

  it('does not change the input list', () => {
    const before = [...recordings]
    filterRecordings(recordings, { search: 'reef', state: 'ready', date: null })
    expect(recordings).toEqual(before)
  })

  it('lists the days present, newest first', () => {
    expect(utcCalendarDay(DAY_ONE)).toBe('2024-01-01')
    expect(dateFilterOptions(recordings)).toEqual([
      { text: '2024-01-02 UTC', value: '2024-01-02' },
      { text: '2024-01-01 UTC', value: '2024-01-01' },
    ])
  })
})
