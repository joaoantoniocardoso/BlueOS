/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus } from '@blueos-idl/constants'
import { describe, expect, it } from 'vitest'

import {
  CANCEL_JOB,
  DELETE_RECORDING,
  REPAIR_BYTES_PER_SECOND_ESTIMATE,
  REPAIR_RECORDING,
  SNAPSHOT_RECORDING,
} from '@/libs/recorder/constants'
import type { LibraryRecording, RecordingJobResult } from '@/libs/recorder/types'
import {
  canDownloadRecording,
  canLoadThumbnail,
  canPlayRecording,
  deleteConfirmationMessage,
  downloadTooltip,
  durationLabel,
  formatDuration,
  jobCanceledMessage,
  jobFailureMessage,
  operationButtons,
  operationDisabledReason,
  readySnapshotDownloadPath,
  recorderMetrics,
  RECORDING_OPERATION_UI,
  recordingByPath,
  recordingCaption,
  recordingMetaLabel,
  recordingDownload,
  recordingEndSeconds,
  repairEstimateMessage,
  repairProgress,
  type RepairProgress,
  snapshotDownloadPath,
  tracksLabel,
  withLiveDuration,
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
    duration_seconds: null,
    video_topics: ['video/camera/stream'],
    other_topic_count: 0,
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

  it('allows playback for ready, in-progress and unindexed recordings, but not while one is rewritten', () => {
    expect(canPlayRecording(file({ state: 'ready' }))).toBe(true)
    expect(canPlayRecording(file({ state: 'recording' }))).toBe(true)
    expect(canPlayRecording(file({ state: 'needs_repair' }))).toBe(true)
    expect(canPlayRecording(file({ state: 'repairing' }))).toBe(false)
  })

  it('builds a thumbnail only for a recording with an index to seek in', () => {
    expect(canLoadThumbnail(file({ state: 'ready' }))).toBe(true)
    expect(canLoadThumbnail(file({ state: 'recording' }))).toBe(false)
    expect(canLoadThumbnail(file({ state: 'needs_repair' }))).toBe(false)
    expect(canLoadThumbnail(file({ state: 'repairing' }))).toBe(false)
    expect(canLoadThumbnail(file({ state: 'ready', video_topics: [] }))).toBe(false)
    expect(canLoadThumbnail(file({ state: 'ready', video_topics: null }))).toBe(true)
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

  it('adds the time left of a repair from its speed', () => {
    const repairing = file({
      state: 'repairing',
      repair_bytes_processed: 1024 * 1024,
      repair_total_bytes: 4 * 1024 * 1024,
      repair_bytes_per_second: 1024 * 1024 / 50,
    })
    expect(repairProgress(repairing)?.label).toBe('1.0 MB of 4.0 MB \u00B7 2m 30s left')
    expect(repairProgress({ ...repairing, repair_bytes_processed: 4 * 1024 * 1024 })?.label).toBe('4.0 MB of 4.0 MB')
  })

  it('formats a duration in days, hours, minutes and seconds', () => {
    expect(formatDuration(125)).toBe('2m 05s')
    expect(formatDuration(3 * 3600 + 7 * 60 + 9)).toBe('3h 07m 09s')
    expect(formatDuration(2 * 86400 + 5 * 3600 + 60)).toBe('2d 5h 01m')
    expect(formatDuration(-4)).toBe('0m 00s')
  })

  it('tells how much a repair rewrites and how long it should take', () => {
    const small = file({ name: 'small.mcap', size_bytes: REPAIR_BYTES_PER_SECOND_ESTIMATE * 10 })
    expect(repairEstimateMessage([small])).toMatch(
      /^Repairing small\.mcap rewrites 250\.0 MB on the vehicle and should take less than a minute,/,
    )
    const large = file({ name: 'large.mcap', size_bytes: REPAIR_BYTES_PER_SECOND_ESTIMATE * 3600 })
    expect(repairEstimateMessage([small, large])).toMatch(
      /^Repairing 2 recordings rewrites .* and should take around 1h 00m 10s,/,
    )
    expect(repairEstimateMessage([small])).toContain('you can stop the repair at any time.')
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

  it('downloads a finished file as it is, the file being written as a snapshot, and nothing while repairing', () => {
    expect(recordingDownload(file({ state: 'ready' }))).toBe('direct')
    expect(recordingDownload(file({ state: 'needs_repair' }))).toBe('direct')
    expect(recordingDownload(file({ state: 'recording' }))).toBe('snapshot')
    expect(recordingDownload(file({ state: 'repairing' }))).toBeNull()
    expect(canDownloadRecording(file({ state: 'needs_repair' }))).toBe(true)
    expect(canDownloadRecording(file({ state: 'repairing' }))).toBe(false)
    expect(downloadTooltip(file({ state: 'repairing' }))).toBe('Wait until repair finishes before downloading')
    expect(downloadTooltip(file({ state: 'ready', name: 'dive.mcap' }))).toBe('Download dive.mcap')
    expect(downloadTooltip(file({ state: 'recording' }))).toBe('Download what has been written so far')
    expect(downloadTooltip(file({ state: 'needs_repair', name: 'dive.mcap' })))
      .toBe('Download dive.mcap. It has no index: Repair makes it seekable')
  })

  it('never shows a snapshot as an operation, nor reports its Job, which the Download reports itself', () => {
    expect(Object.keys(RECORDING_OPERATION_UI)).not.toContain(SNAPSHOT_RECORDING)
    expect(operationButtons(file({ allowed_operations: [SNAPSHOT_RECORDING, DELETE_RECORDING] })))
      .toEqual([DELETE_RECORDING])
    const result = { path: 'live.mcap', output_path: '' }
    expect(jobFailureMessage(jobResult(SNAPSHOT_RECORDING, JobStatusStatus.Aborted, result, 'disk full'))).toBeNull()
    expect(jobCanceledMessage(jobResult(SNAPSHOT_RECORDING, JobStatusStatus.Canceled, result))).toBeNull()
  })

  it('keeps Delete and a needed Repair as buttons, disabled with the reason the recorder refuses them', () => {
    const live = file({ state: 'recording', allowed_operations: [SNAPSHOT_RECORDING] })
    expect(operationButtons(live)).toEqual([DELETE_RECORDING])
    expect(operationDisabledReason(live, DELETE_RECORDING)).toBe('Cannot delete while the vehicle is still recording')

    const repairing = file({ state: 'repairing', allowed_operations: [CANCEL_JOB] })
    expect(operationButtons(repairing)).toEqual([CANCEL_JOB, DELETE_RECORDING])
    expect(operationDisabledReason(repairing, CANCEL_JOB)).toBeNull()
    expect(operationDisabledReason(repairing, DELETE_RECORDING))
      .toBe('Cannot delete while the recording is being repaired')

    const broken = file({
      state: 'needs_repair', allowed_operations: [DELETE_RECORDING], repair_error: 'Not an MCAP file',
    })
    expect(operationButtons(broken)).toEqual([REPAIR_RECORDING, DELETE_RECORDING])
    expect(operationDisabledReason(broken, REPAIR_RECORDING)).toBe('Not an MCAP file')
    expect(operationDisabledReason(file({ state: 'needs_repair', allowed_operations: [] }), REPAIR_RECORDING))
      .toBe('Wait until the file is finished before repairing')
    expect(operationDisabledReason(broken, DELETE_RECORDING)).toBeNull()
  })

  it('asks before deleting one recording or many', () => {
    expect(deleteConfirmationMessage([file({ name: 'dive.mcap' })])).toBe('Delete dive.mcap? This cannot be undone.')
    expect(deleteConfirmationMessage([file(), file({ path: 'other.mcap' })]))
      .toBe('Delete 2 recordings? This cannot be undone.')
  })
})

describe('withLiveDuration', () => {
  it('times the file being written from its start to now, and leaves the others as the library has them', () => {
    const finished = file({ path: 'old.mcap', state: 'ready', duration_seconds: 30 })
    const [live, same] = withLiveDuration([file({ state: 'recording', created: 1_000 }), finished], 1_090)
    expect(live.duration_seconds).toBe(90)
    expect(same).toBe(finished)
  })

  it('returns the same list when nothing is being written', () => {
    const files = [file({ state: 'ready' }), file({ path: 'b.mcap', state: 'needs_repair' })]
    expect(withLiveDuration(files, 1_090)).toBe(files)
  })
})

describe('durationLabel, tracksLabel and recordingEndSeconds', () => {
  it('shows a known duration and a dash for an unknown one', () => {
    expect(durationLabel(file({ duration_seconds: 125 }))).toBe('2m 05s')
    expect(durationLabel(file({ duration_seconds: 0 }))).toBe('0m 00s')
    expect(durationLabel(file({ duration_seconds: null }))).toBe('-')
  })

  it('names the video topics and counts the others, and says when the contents are unknown', () => {
    expect(tracksLabel(file({ video_topics: ['video/a/stream', 'video/b/stream'], other_topic_count: 3 })))
      .toBe('video/a/stream, video/b/stream \u00B7 3 other topics')
    expect(tracksLabel(file({ video_topics: [], other_topic_count: 1 }))).toBe('1 other topic')
    expect(tracksLabel(file({ video_topics: ['video/a/stream'], other_topic_count: 0 }))).toBe('video/a/stream')
    expect(tracksLabel(file({ video_topics: [], other_topic_count: 0 }))).toBe('no topics')
    expect(tracksLabel(file({ video_topics: null, other_topic_count: null }))).toBe('unknown')
  })

  it('ends a finished recording its duration after its start, and not one being written or of unknown length', () => {
    expect(recordingEndSeconds(file({ state: 'ready', created: 1_000, duration_seconds: 90 }))).toBe(1_090)
    expect(recordingEndSeconds(file({ state: 'recording', created: 1_000, duration_seconds: 90 }))).toBeNull()
    expect(recordingEndSeconds(file({ state: 'needs_repair', duration_seconds: null }))).toBeNull()
  })
})

describe('recordingCaption', () => {
  it('says the file is being written, or why it needs repair', () => {
    expect(recordingCaption(file({ state: 'recording' }))).toBe('recording...')
    expect(recordingCaption(file({ state: 'needs_repair' }))).toBe('Recording index is missing')
    expect(recordingCaption(file({ state: 'needs_repair', repair_error: 'Not an MCAP file' }))).toBe('Not an MCAP file')
    expect(recordingCaption(file({ state: 'ready' }))).toBeNull()
  })
})

describe('recordingMetaLabel', () => {
  it('sums up the duration, the video streams and the size of the recording in the player', () => {
    expect(recordingMetaLabel(file({
      duration_seconds: 125, video_topics: ['video/a/stream', 'video/b/stream'], size_bytes: 2048,
    }))).toBe('2m 05s \u00B7 2 streams \u00B7 2.0 kB')
    expect(recordingMetaLabel(file({ duration_seconds: null, video_topics: ['video/a/stream'], size_bytes: 2048 })))
      .toBe('1 stream \u00B7 2.0 kB')
    expect(recordingMetaLabel(file({ duration_seconds: null, video_topics: null, size_bytes: 2048 }))).toBe('2.0 kB')
  })
})

describe('recorderMetrics', () => {
  function counter(name: string, lane: string, value: number) {
    return { name, labels: [{ name: 'lane', value: lane }], value }
  }

  it('lays the per-lane counters out in a fixed lane order, with zero for a counter not published yet', () => {
    const metrics = recorderMetrics({
      counters: [
        counter('bytes_written', 'other', 700),
        counter('samples_written', 'other', 7),
        counter('bytes_written', 'mavlink', 50),
        counter('samples_dropped', 'video', 4),
      ],
      gauges: [],
      histograms: [],
    })

    expect(metrics.lanes).toEqual([
      { lane: 'mavlink', bytesWritten: 50, samplesWritten: 0, samplesDropped: 0 },
      { lane: 'video', bytesWritten: 0, samplesWritten: 0, samplesDropped: 4 },
      { lane: 'other', bytesWritten: 700, samplesWritten: 7, samplesDropped: 0 },
    ])
  })

  it('reads the Inbox depth and the mean Inbox step time, and none while the Kernel has not stepped', () => {
    const metrics = recorderMetrics({
      counters: [],
      gauges: [{ name: 'inbox_depth', labels: [], value: 3 }],
      histograms: [{
        name: 'inbox_step_seconds',
        labels: [],
        count: 4,
        sum: 0.002,
        bucket_bounds: [0.001],
        bucket_counts: [4, 0],
      }],
    })
    expect(metrics.inboxDepth).toBe(3)
    expect(metrics.inboxStepSeconds).toBeCloseTo(0.0005)

    const idle = recorderMetrics({ counters: [], gauges: [], histograms: [] })
    expect(idle.inboxDepth).toBeNull()
    expect(idle.inboxStepSeconds).toBeNull()
  })
})
