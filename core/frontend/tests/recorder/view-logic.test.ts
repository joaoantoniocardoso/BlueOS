import { describe, expect, it } from 'vitest'

import type { LibraryRecording } from '@/libs/recorder'
import {
  canDownloadRecording,
  formatDuration,
  isSnapshotOperationForPath,
  needsSnapshotBeforeDownload,
  repairEstimateMessage,
  snapshotDownloadPath,
  sortRecordingsNewestFirst,
  utcCalendarDay,
} from '@/libs/recorder'

function file(overrides: Partial<LibraryRecording> = {}): LibraryRecording {
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
    ...overrides,
  }
}

describe('recorder view logic', () => {
  it('formats durations', () => {
    expect(formatDuration(125)).toBe('2m 05s')
  })

  it('groups UTC calendar days', () => {
    expect(utcCalendarDay(1_704_067_200)).toMatch(/^\d{4}-\d{2}-\d{2}$/)
  })

  it('sorts newest first', () => {
    const sorted = sortRecordingsNewestFirst([
      file({ path: 'old.mcap', created: 1 }),
      file({ path: 'new.mcap', created: 2 }),
    ])
    expect(sorted[0].path).toBe('new.mcap')
  })

  it('decides download eligibility', () => {
    expect(canDownloadRecording(file({ state: 'ready' }), true)).toBe(true)
    expect(canDownloadRecording(file({ state: 'needs_repair' }), true)).toBe(false)
    expect(needsSnapshotBeforeDownload(file({ state: 'recording' }))).toBe(true)
  })

  it('matches snapshot operations', () => {
    const event = {
      operation: 'snapshot' as const,
      path: 'live.mcap',
      output_path: 'live.snapshot.mcap',
      succeeded: true,
      cancelled: false,
      error: '',
    }
    expect(isSnapshotOperationForPath(event, 'live.mcap')).toBe(true)
    expect(snapshotDownloadPath(event)).toBe('live.snapshot.mcap')
  })

  it('builds repair estimate copy', () => {
    const message = repairEstimateMessage([file({ name: 'x.mcap', size_bytes: 1024 })], (bytes) => `${bytes}B`)
    expect(message).toContain('x.mcap')
  })
})
