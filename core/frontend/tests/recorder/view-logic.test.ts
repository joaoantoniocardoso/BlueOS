/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus } from '@blueos-idl/constants'
import { describe, expect, it } from 'vitest'

import { DELETE_RECORDING, REPAIR_RECORDING, SNAPSHOT_RECORDING } from '@/libs/recorder/constants'
import type { LibraryRecording, RecordingJobResult } from '@/libs/recorder/types'
import {
  canPlayRecording,
  jobCanceledMessage,
  jobFailureMessage,
  readySnapshotDownloadPath,
  recordingByPath,
  repairProgress,
  type RepairProgress,
  snapshotDownloadPath,
  sortRecordingsNewestFirst,
  withRepairJobs,
} from '@/libs/recorder/view-logic'

function jobResult(
  jobType: string,
  status: JobStatusStatus,
  result: RecordingJobResult['result'],
  reason = '',
): RecordingJobResult {
  return {
    job: {
      job_id: '0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10', job_type: jobType, status, reason,
    },
    result,
  }
}

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
    repair_job_id: '',
    allowed_operations: ['SnapshotRecording'],
    ...overrides,
  }
}

describe('recorder view-logic', () => {
  it('returns the current library row for an open recording path', () => {
    const growing = file({ path: 'live.mcap', size_bytes: 1000 })
    expect(recordingByPath([growing], 'live.mcap')?.size_bytes).toBe(1000)
    const updated = file({ path: 'live.mcap', size_bytes: 5000 })
    expect(recordingByPath([updated], 'live.mcap')?.size_bytes).toBe(5000)
    expect(recordingByPath([updated], 'other.mcap')).toBeNull()
    expect(recordingByPath([updated], null)).toBeNull()
  })

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

  it('reads the snapshot output path from the result of a succeeded snapshot Job', () => {
    const output = 'live.snapshot-2024-01-02T03-04-05Z.mcap'
    const result = { path: 'live.mcap', output_path: output }
    expect(snapshotDownloadPath(jobResult(SNAPSHOT_RECORDING, JobStatusStatus.Succeeded, result))).toBe(output)
    expect(snapshotDownloadPath(jobResult(SNAPSHOT_RECORDING, JobStatusStatus.Aborted, result))).toBeNull()
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

  it('says why a Job aborted, and nothing for a Job that succeeded or was canceled', () => {
    const result = { path: 'gone.mcap' }
    expect(jobFailureMessage(jobResult(DELETE_RECORDING, JobStatusStatus.Aborted, result, 'disk full')))
      .toBe('Delete failed for gone.mcap: disk full')
    expect(jobFailureMessage(jobResult(REPAIR_RECORDING, JobStatusStatus.Aborted, result, '')))
      .toBe('Repair failed for gone.mcap: unknown error')
    expect(jobFailureMessage(jobResult(DELETE_RECORDING, JobStatusStatus.Succeeded, result))).toBeNull()
    expect(jobFailureMessage(jobResult(REPAIR_RECORDING, JobStatusStatus.Canceled, result))).toBeNull()
  })

  it('reports how far a repairing recording is, and nothing for any other', () => {
    const repairing = file({
      state: 'repairing', repair_bytes_processed: 512 * 1024, repair_total_bytes: 2 * 1024 * 1024,
    })
    expect(repairProgress(repairing)).toEqual({ percent: 25, label: '512.0 kB of 2.0 MB' })
    expect(repairProgress({ ...repairing, repair_bytes_processed: 3 * 1024 * 1024 }))
      .toEqual({ percent: 100, label: '3.0 MB of 2.0 MB' })
    expect(repairProgress({ ...repairing, repair_total_bytes: 0 })).toBeNull()
    expect(repairProgress({ ...repairing, state: 'needs_repair' })).toBeNull()
  })

  it('reports the live Feedback of the repair Job for a repairing row', () => {
    const jobId = '0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10'
    const repairing = file({ state: 'repairing', repair_job_id: jobId })
    const [live] = withRepairJobs(
      [repairing],
      { [jobId]: { bytes_processed: 1024 * 1024, total_bytes: 4 * 1024 * 1024 } } as RepairProgress,
      [],
    )
    expect(repairProgress(live)).toEqual({ percent: 25, label: '1.0 MB of 4.0 MB' })
  })

  it('says which Job was canceled, and nothing for a Job that ended any other way', () => {
    const result = { path: 'half.mcap' }
    expect(jobCanceledMessage(jobResult(REPAIR_RECORDING, JobStatusStatus.Canceled, result)))
      .toBe('Repair canceled for half.mcap')
    expect(jobCanceledMessage(jobResult(REPAIR_RECORDING, JobStatusStatus.Succeeded, result))).toBeNull()
    expect(jobCanceledMessage(jobResult(REPAIR_RECORDING, JobStatusStatus.Aborted, result, 'disk full'))).toBeNull()
  })
})
