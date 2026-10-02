import { describe, expect, it } from 'vitest'

import type { LibraryRecording } from '@/libs/recorder/types'
import {
  canPlayRecording,
  operationFailureMessage,
  readySnapshotDownloadPath,
  snapshotDownloadPath,
  sortRecordingsNewestFirst,
} from '@/libs/recorder/view-logic'

function file(overrides: Partial<LibraryRecording> = {}): LibraryRecording {
  return {
    path: 'live.mcap',
    name: 'live.mcap',
    size_bytes: 1000,
    created: 1_700_000_000,
    state: 'recording',
    repair_bytes_processed: 0,
    repair_total_bytes: 0,
    repair_bytes_per_second: 0,
    repair_error: '',
    allowed_operations: ['SnapshotRecording'],
    ...overrides,
  }
}

describe('recorder view-logic', () => {
  it('sorts recordings newest first by created time', () => {
    const sorted = sortRecordingsNewestFirst([
      file({ path: 'old.mcap', created: 1 }),
      file({ path: 'new.mcap', created: 2 }),
    ])
    expect(sorted.map((entry) => entry.path)).toEqual(['new.mcap', 'old.mcap'])
  })

  it('allows playback for ready and in-progress recordings', () => {
    expect(canPlayRecording(file({ state: 'ready' }))).toBe(true)
    expect(canPlayRecording(file({ state: 'recording' }))).toBe(true)
    expect(canPlayRecording(file({ state: 'needs_repair' }))).toBe(false)
  })

  it('reads snapshot output path from a succeeded operation event', () => {
    expect(snapshotDownloadPath({
      operation: 'snapshot',
      path: 'live.mcap',
      output_path: 'live.snapshot-2024-01-02T03-04-05Z.mcap',
      succeeded: true,
      cancelled: false,
      error: '',
    })).toBe('live.snapshot-2024-01-02T03-04-05Z.mcap')
  })

  it('finds a ready snapshot file in the library after a missed event', () => {
    const output = readySnapshotDownloadPath('live.mcap', [
      file(),
      file({
        path: 'live.snapshot-2024-01-02T03-04-05Z.mcap',
        name: 'live.snapshot-2024-01-02T03-04-05Z.mcap',
        state: 'ready',
        allowed_operations: ['DeleteRecording'],
      }),
    ])
    expect(output).toBe('live.snapshot-2024-01-02T03-04-05Z.mcap')
  })

  it('builds an operation failure message', () => {
    expect(operationFailureMessage({
      operation: 'delete',
      path: 'gone.mcap',
      output_path: '',
      succeeded: false,
      cancelled: false,
      error: 'disk full',
    }, 'gone.mcap')).toBe('Delete failed for gone.mcap: disk full')
  })
})
