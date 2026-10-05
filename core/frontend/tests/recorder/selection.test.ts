import { describe, expect, it } from 'vitest'

import {
  allVisibleSelected,
  pruneSelection,
  selectedVisibleRecordings,
  selectionSummary,
  selectionSummaryLabel,
  setVisibleSelection,
  someVisibleSelected,
  togglePathSelection,
} from '@/libs/recorder/selection'
import type { LibraryRecording } from '@/libs/recorder/types'
import { withLiveDuration } from '@/libs/recorder/view-logic'

function file(path: string): LibraryRecording {
  return {
    path,
    name: path,
    size_bytes: 1,
    created: 1,
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
  }
}

const visible = [file('a.mcap'), file('b.mcap'), file('c.mcap')]

describe('recorder selection', () => {
  it('selects and deselects one path', () => {
    expect(togglePathSelection([], 'a.mcap')).toEqual(['a.mcap'])
    expect(togglePathSelection(['a.mcap'], 'a.mcap')).toEqual([])
  })

  it('selects the chosen visible paths without dropping hidden selections', () => {
    expect(setVisibleSelection(['hidden.mcap'], visible, visible)).toEqual([
      'hidden.mcap',
      'a.mcap',
      'b.mcap',
      'c.mcap',
    ])
    expect(setVisibleSelection(['hidden.mcap', 'a.mcap', 'b.mcap'], visible, [visible[2]])).toEqual([
      'hidden.mcap',
      'c.mcap',
    ])
    expect(setVisibleSelection(['hidden.mcap', 'a.mcap', 'b.mcap'], visible, [])).toEqual(['hidden.mcap'])
  })

  it('knows when all or some visible rows are selected', () => {
    expect(allVisibleSelected([], visible)).toBe(false)
    expect(allVisibleSelected(['a.mcap', 'b.mcap', 'c.mcap'], visible)).toBe(true)
    expect(someVisibleSelected(['b.mcap'], visible)).toBe(true)
    expect(someVisibleSelected([], visible)).toBe(false)
  })

  it('lists the selected recordings that are visible, in visible order', () => {
    expect(selectedVisibleRecordings(['c.mcap', 'a.mcap', 'missing.mcap'], visible)).toEqual([
      file('a.mcap'),
      file('c.mcap'),
    ])
  })

  it('drops paths that left the library', () => {
    expect(pruneSelection(['a.mcap', 'gone.mcap'], ['a.mcap', 'b.mcap'])).toEqual(['a.mcap'])
  })
})

describe('selectionSummary', () => {
  const now = 10_000
  const files = withLiveDuration([
    { ...file('a.mcap'), size_bytes: 1024 ** 3, duration_seconds: 3600 },
    { ...file('b.mcap'), size_bytes: 512 * 1024 ** 2, state: 'needs_repair' },
    {
      ...file('live.mcap'), size_bytes: 1024, state: 'recording', created: now - 120,
    },
  ], now)

  it('sums size and duration, counts unknown durations and Needs repair, and times the live file until now', () => {
    expect(selectionSummary(files)).toEqual({
      count: 3,
      sizeBytes: 1024 ** 3 + 512 * 1024 ** 2 + 1024,
      durationSeconds: 3720,
      unknownDurationCount: 1,
      needsRepairCount: 1,
    })
  })

  it('labels the summary and drops the parts that are zero', () => {
    expect(selectionSummaryLabel(selectionSummary(files)))
      .toBe('3 selected \u00B7 1.5 GB \u00B7 1h 02m 00s (1 without a known duration) \u00B7 1 need repair')
    expect(selectionSummaryLabel(selectionSummary([files[0]]))).toBe('1 selected \u00B7 1.0 GB \u00B7 1h 00m 00s')
  })
})
