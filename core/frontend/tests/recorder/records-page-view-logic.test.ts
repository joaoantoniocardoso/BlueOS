import { describe, expect, it } from 'vitest'

import type { LibraryRecording } from '@/libs/recorder'
import {
  browsingAllowedWhileArmed,
  deleteConfirmationMessage,
  filterRecordingsByDate,
  readySnapshotDownloadPath,
  recordingDurationSeconds,
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

describe('records page view logic', () => {
  it('pauses browsing while armed', () => {
    expect(browsingAllowedWhileArmed(false)).toBe(true)
    expect(browsingAllowedWhileArmed(true)).toBe(false)
  })

  it('filters by UTC date', () => {
    const recordings = [file({ created: 1_704_067_200 })]
    const day = '2024-01-01'
    expect(filterRecordingsByDate(recordings, day).length).toBe(
      filterRecordingsByDate(recordings, '2024-01-01').length,
    )
  })

  it('builds delete confirmation copy', () => {
    expect(deleteConfirmationMessage([file()])).toContain('a.mcap')
  })

  it('estimates live recording duration', () => {
    const duration = recordingDurationSeconds(
      file({ state: 'recording', created: 100 }),
      {},
      150,
    )
    expect(duration).toBe(50)
  })

  it('finds a ready snapshot next to the source recording', () => {
    const source = file({ path: 'folder/live.mcap', name: 'live.mcap', state: 'recording' })
    const snapshot = file({
      path: 'folder/live.snapshot-2024-01-02T03-04-05Z.mcap',
      name: 'live.snapshot-2024-01-02T03-04-05Z.mcap',
      state: 'ready',
    })
    expect(readySnapshotDownloadPath(source.path, [source, snapshot])).toBe(snapshot.path)
  })
})
