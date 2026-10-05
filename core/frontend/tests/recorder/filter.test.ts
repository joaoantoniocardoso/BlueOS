import { describe, expect, it } from 'vitest'

import {
  dateFilterOptions,
  filterRecordings,
  NO_RECORDING_FILTERS,
  localCalendarDay,
} from '@/libs/recorder/filter'
import type { LibraryRecording } from '@/libs/recorder/types'

// Three hours behind UTC, so a UTC-based day would disagree with the local clock the cards display. Set before the
// constants below, which build dates in the local zone.
process.env.TZ = 'America/Sao_Paulo'

const DAY_ONE = new Date(2024, 0, 1, 23, 59, 59).getTime() / 1000
const DAY_TWO = new Date(2024, 0, 2, 0, 0, 1).getTime() / 1000

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
    repair_job_id: '',
    allowed_operations: [],
    duration_seconds: null,
    video_topics: [],
    other_topic_count: null,
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

  it('filters by local calendar day', () => {
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
    expect(localCalendarDay(DAY_ONE)).toBe('2024-01-01')
    expect(dateFilterOptions(recordings)).toEqual([
      { text: '2024-01-02', value: '2024-01-02' },
      { text: '2024-01-01', value: '2024-01-01' },
    ])
  })

  it('files a recording under the day its card shows, not the UTC day', () => {
    // recorder_20260930_021902 is 02:19:02 UTC, which is 23:19:02 on the 29th at UTC-3.
    const created = Date.UTC(2026, 8, 30, 2, 19, 2) / 1000
    expect(new Date(created * 1000).toLocaleDateString('en-CA')).toBe('2026-09-29')
    expect(localCalendarDay(created)).toBe('2026-09-29')
    const late = file({ path: 'late.mcap', name: 'late.mcap', created })
    expect(filterRecordings([late], { ...NO_RECORDING_FILTERS, date: '2026-09-29' })).toEqual([late])
    expect(filterRecordings([late], { ...NO_RECORDING_FILTERS, date: '2026-09-30' })).toEqual([])
  })
})
