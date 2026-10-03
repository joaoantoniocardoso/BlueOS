import { describe, expect, it } from 'vitest'

import {
  allVisibleSelected,
  pruneSelection,
  selectedVisibleRecordings,
  setVisibleSelection,
  someVisibleSelected,
  togglePathSelection,
} from '@/libs/recorder/selection'
import type { LibraryRecording } from '@/libs/recorder/types'

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
