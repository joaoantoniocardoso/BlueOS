import { describe, expect, it } from 'vitest'

import { type RecordingSortKey, sortRecordings } from '@/libs/recorder/sort'
import type { LibraryRecording } from '@/libs/recorder/types'

function file(overrides: Partial<LibraryRecording>): LibraryRecording {
  return {
    path: 'a.mcap',
    name: 'a.mcap',
    size_bytes: 1000,
    created: 1_700_000_000,
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

function sortedPaths(files: LibraryRecording[], key: RecordingSortKey, descending: boolean): string[] {
  return sortRecordings(files, key, descending).map((recording) => recording.path)
}

describe('sortRecordings', () => {
  const files = [
    file({
      path: 'b.mcap', name: 'beta.mcap', size_bytes: 30, created: 2, state: 'ready',
    }),
    file({
      path: 'c.mcap', name: 'gamma.mcap', size_bytes: 10, created: 3, state: 'needs_repair',
    }),
    file({
      path: 'a.mcap', name: 'alpha.mcap', size_bytes: 20, created: 1, state: 'recording',
    }),
  ]

  it.each([
    ['created', ['a.mcap', 'b.mcap', 'c.mcap']],
    ['name', ['a.mcap', 'b.mcap', 'c.mcap']],
    ['size_bytes', ['c.mcap', 'a.mcap', 'b.mcap']],
    ['state', ['c.mcap', 'b.mcap', 'a.mcap']],
  ] as [RecordingSortKey, string[]][])('sorts by %s in both directions', (key, ascending) => {
    expect(sortedPaths(files, key, false)).toEqual(ascending)
    expect(sortedPaths(files, key, true)).toEqual([...ascending].reverse())
  })

  it('breaks ties by path', () => {
    const tied = [file({ path: 'z.mcap' }), file({ path: 'm.mcap' }), file({ path: 'a.mcap' })]
    expect(sortedPaths(tied, 'size_bytes', false)).toEqual(['a.mcap', 'm.mcap', 'z.mcap'])
    expect(sortedPaths(tied, 'size_bytes', true)).toEqual(['z.mcap', 'm.mcap', 'a.mcap'])
  })

  it('sorts by duration, an unknown duration last both ways', () => {
    const timed = [
      file({ path: 'long.mcap', duration_seconds: 90 }),
      file({ path: 'broken.mcap', state: 'needs_repair' }),
      file({ path: 'short.mcap', duration_seconds: 30 }),
      file({ path: 'empty.mcap', duration_seconds: 0 }),
    ]
    expect(sortedPaths(timed, 'duration', false)).toEqual(['empty.mcap', 'short.mcap', 'long.mcap', 'broken.mcap'])
    expect(sortedPaths(timed, 'duration', true)).toEqual(['long.mcap', 'short.mcap', 'empty.mcap', 'broken.mcap'])
  })

  it('leaves the given list in place', () => {
    const given = [...files]
    sortRecordings(given, 'name', false)
    expect(given).toEqual(files)
  })
})
