/* eslint-disable import/no-extraneous-dependencies */
import { CommandAckStatus, JobStatusStatus } from '@blueos-idl/constants'
import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { jobResultEvent, metricsState } from '@/libs/blueos-api/endpoints'
import { cdrEncoding, commandKey, serviceLivelinessKey } from '@/libs/blueos-api/keys'
import {
  DeleteRecording,
  library,
  NAME,
  RepairRecording,
  SnapshotRecording,
} from '@/libs/blueos-api/services/recorder'
import { SNAPSHOT_WAIT_TIMEOUT_MS, createRecorderClient } from '@/libs/recorder/client'

import FakeTransport from '../blueos-api/fake-transport'

const JOB_ID = '0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10'

const idleLibrary = { files: [] as never[] }

function recordingFile(overrides: Record<string, unknown> = {}) {
  return {
    path: 'live.mcap',
    name: 'live.mcap',
    size_bytes: 10,
    created: { sec: 1, nanosec: 0 },
    state: 0,
    repair_bytes_processed: 0,
    repair_total_bytes: 0,
    repair_bytes_per_second: 0,
    repair_error: '',
    repair_job_id: '',
    allowed_operations: ['SnapshotRecording'],
    ...overrides,
  }
}

function snapshotJobResult(outputPath: string) {
  return {
    job: {
      job_id: JOB_ID,
      job_type: SnapshotRecording.name,
      status: JobStatusStatus.Succeeded,
      reason: '',
    },
    result: Array.from(encodeCdr(SnapshotRecording.resultSchema, { path: 'live.mcap', output_path: outputPath })),
  }
}

function counter(name: string, lane: string, value: number) {
  return { name, labels: [{ name: 'lane', value: lane }], value }
}

function recorderMetricsMessage(videoBytes: number) {
  return {
    counters: [
      counter('bytes_written', 'video', videoBytes),
      counter('samples_written', 'video', 3),
      counter('samples_dropped', 'video', 1),
    ],
    gauges: [{ name: 'inbox_depth', labels: [], value: 2 }],
    histograms: [],
  }
}

describe('createRecorderClient', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('watches library state and service liveliness without polling', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const recordings: string[][] = []
    const running: boolean[] = []

    const libraryWatch = client.watchLibrary((files) => {
      recordings.push(files.map((file) => file.path))
    })
    const runningWatch = client.watchServiceRunning((alive) => running.push(alive))

    const stateQuery = await transport.nextQuery()
    expect(stateQuery.key).toBe(library.key)
    stateQuery.reply({
      kind: 'sample',
      sample: {
        key: library.key,
        payload: encodeCdr(library.messageSchema, idleLibrary),
        encoding: cdrEncoding(library.messageSchema),
      },
    })
    await libraryWatch
    await runningWatch

    transport.publishLiveliness(serviceLivelinessKey('recorder'), true)
    transport.publish({
      key: library.key,
      payload: encodeCdr(library.messageSchema, {
        files: [recordingFile({ path: 'a.mcap', name: 'a.mcap', state: 1, allowed_operations: ['DeleteRecording'] })],
      }),
      encoding: cdrEncoding(library.messageSchema),
    })

    expect(running).toEqual([true])
    expect(recordings).toEqual([[], ['a.mcap']])
  })

  it('resolves snapshot before the command ack when the Job result arrives early', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const operationWatch = client.watchOperations(() => undefined)
    await operationWatch

    const newSnapshot = 'live.snapshot-2024-01-02T03-04-05Z.mcap'
    const snapshot = client.snapshotRecording('live.mcap')
    const snapshotQuery = await transport.nextQuery()
    expect(snapshotQuery.key).toBe(SnapshotRecording.key)

    const results = jobResultEvent(NAME, SnapshotRecording.name)
    transport.publish({
      key: results.key,
      payload: encodeCdr(results.messageSchema, snapshotJobResult(newSnapshot)),
      encoding: cdrEncoding(results.messageSchema),
    })

    snapshotQuery.reply({
      kind: 'sample',
      sample: {
        key: SnapshotRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: JOB_ID,
          status: CommandAckStatus.Succeeded,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    await expect(snapshot).resolves.toBe(newSnapshot)
  })

  it('reports the Job result of a delete as well as of a repair', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const reported: unknown[] = []
    await client.watchOperations((entry) => reported.push(entry))
    const aborted = {
      job_id: JOB_ID, job_type: DeleteRecording.name, status: JobStatusStatus.Aborted, reason: 'invalid recording path',
    }

    for (const [operation, result] of [
      [DeleteRecording, { path: 'old.mcap' }],
      [RepairRecording, { path: 'old.mcap' }],
    ] as const) {
      const results = jobResultEvent(NAME, operation.name)
      transport.publish({
        key: results.key,
        payload: encodeCdr(results.messageSchema, {
          job: { ...aborted, job_type: operation.name },
          result: Array.from(encodeCdr(operation.resultSchema, result)),
        }),
        encoding: cdrEncoding(results.messageSchema),
      })
    }

    expect(reported).toEqual([
      { job: aborted, result: { path: 'old.mcap' } },
      { job: { ...aborted, job_type: RepairRecording.name }, result: { path: 'old.mcap' } },
    ])
  })

  it('resolves snapshot before the command ack when the library update arrives early', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const libraryWatch = client.watchLibrary(() => undefined)
    const stateQuery = await transport.nextQuery()
    stateQuery.reply({
      kind: 'sample',
      sample: {
        key: library.key,
        payload: encodeCdr(library.messageSchema, { files: [recordingFile()] }),
        encoding: cdrEncoding(library.messageSchema),
      },
    })
    await libraryWatch

    const newSnapshot = 'live.snapshot-2024-01-02T03-04-05Z.mcap'
    const snapshot = client.snapshotRecording('live.mcap')
    const snapshotQuery = await transport.nextQuery()

    transport.publish({
      key: library.key,
      payload: encodeCdr(library.messageSchema, {
        files: [
          recordingFile(),
          recordingFile({
            path: newSnapshot,
            name: newSnapshot,
            state: 1,
            allowed_operations: ['DeleteRecording'],
          }),
        ],
      }),
      encoding: cdrEncoding(library.messageSchema),
    })

    snapshotQuery.reply({
      kind: 'sample',
      sample: {
        key: SnapshotRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: JOB_ID,
          status: CommandAckStatus.Succeeded,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    await expect(snapshot).resolves.toBe(newSnapshot)
  })

  it('ignores an older snapshot already in the library when waiting for a new one', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const oldSnapshot = 'live.snapshot-2024-01-01T00-00-00Z.mcap'
    const libraryWatch = client.watchLibrary(() => undefined)
    const stateQuery = await transport.nextQuery()
    stateQuery.reply({
      kind: 'sample',
      sample: {
        key: library.key,
        payload: encodeCdr(library.messageSchema, {
          files: [
            recordingFile(),
            recordingFile({
              path: oldSnapshot,
              name: oldSnapshot,
              state: 1,
              allowed_operations: ['DeleteRecording'],
            }),
          ],
        }),
        encoding: cdrEncoding(library.messageSchema),
      },
    })
    await libraryWatch

    const snapshot = client.snapshotRecording('live.mcap')
    const snapshotQuery = await transport.nextQuery()
    snapshotQuery.reply({
      kind: 'sample',
      sample: {
        key: SnapshotRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: JOB_ID,
          status: CommandAckStatus.Succeeded,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    await Promise.resolve()
    let settled = false
    snapshot.then(() => { settled = true }).catch(() => { settled = true })
    await Promise.resolve()
    expect(settled).toBe(false)

    const newSnapshot = 'live.snapshot-2024-01-02T03-04-05Z.mcap'
    transport.publish({
      key: library.key,
      payload: encodeCdr(library.messageSchema, {
        files: [
          recordingFile(),
          recordingFile({
            path: oldSnapshot,
            name: oldSnapshot,
            state: 1,
            allowed_operations: ['DeleteRecording'],
          }),
          recordingFile({
            path: newSnapshot,
            name: newSnapshot,
            state: 1,
            allowed_operations: ['DeleteRecording'],
          }),
        ],
      }),
      encoding: cdrEncoding(library.messageSchema),
    })
    await expect(snapshot).resolves.toBe(newSnapshot)
  })

  it('rejects snapshotRecording when the command is refused and clears the waiter', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    const snapshot = client.snapshotRecording('live.mcap')
    const snapshotQuery = await transport.nextQuery()
    snapshotQuery.reply({
      kind: 'sample',
      sample: {
        key: SnapshotRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: false,
          job_id: JOB_ID,
          status: CommandAckStatus.StatusUnknown,
          reason: 'still processing',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    await expect(snapshot).rejects.toThrow('still processing')
  })

  it('times out when no snapshot file appears', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    const snapshot = client.snapshotRecording('live.mcap')
    const snapshotQuery = await transport.nextQuery()
    snapshotQuery.reply({
      kind: 'sample',
      sample: {
        key: SnapshotRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: JOB_ID,
          status: CommandAckStatus.Succeeded,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    vi.advanceTimersByTime(SNAPSHOT_WAIT_TIMEOUT_MS + 1)
    await expect(snapshot).rejects.toThrow(/Timed out waiting for the snapshot file/)
  })

  it('sends repair and delete commands', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    const repair = client.repairRecording('broken.mcap')
    const repairQuery = await transport.nextQuery()
    expect(repairQuery.key).toBe(RepairRecording.key)
    repairQuery.reply({
      kind: 'sample',
      sample: {
        key: RepairRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: JOB_ID,
          status: CommandAckStatus.Executing,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })
    expect(await repair).toMatchObject({ accepted: true })

    const cancel = client.cancelRepair(JOB_ID)
    const cancelQuery = await transport.nextQuery()
    expect(cancelQuery.key).toBe(commandKey(NAME, 'CancelJob'))
    expect(new TextDecoder().decode(cancelQuery.body?.attachment)).toBe(JOB_ID)
    cancelQuery.reply({
      kind: 'sample',
      sample: {
        key: cancelQuery.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: false,
          job_id: JOB_ID,
          status: CommandAckStatus.StatusUnknown,
          reason: 'no such Job',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })
    expect(await cancel).toMatchObject({ accepted: false, reason: 'no such Job' })

    const deleted = client.deleteRecording('old.mcap')
    const deleteQuery = await transport.nextQuery()
    expect(deleteQuery.key).toBe(DeleteRecording.key)
    deleteQuery.reply({
      kind: 'sample',
      sample: {
        key: DeleteRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: JOB_ID,
          status: CommandAckStatus.Succeeded,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })
    expect(await deleted).toMatchObject({ accepted: true })
  })

  it('forwards library decode errors to onError', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const errors: unknown[] = []
    const libraryWatch = client.watchLibrary(() => undefined, (error) => errors.push(error))
    const stateQuery = await transport.nextQuery()
    stateQuery.reply({
      kind: 'sample',
      sample: {
        key: library.key,
        payload: encodeCdr(library.messageSchema, idleLibrary),
        encoding: cdrEncoding(library.messageSchema),
      },
    })
    await libraryWatch

    transport.publish({
      key: library.key,
      payload: new Uint8Array([0, 1, 2]),
      encoding: cdrEncoding(library.messageSchema),
    })

    expect(errors.length).toBeGreaterThan(0)
  })

  it('hands the Recorder metrics to the observer at once for a page opened mid-recording, then each change', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const { key, messageSchema } = metricsState(NAME)
    const videoBytes: number[] = []

    const watching = client.watchMetrics((metrics) => {
      videoBytes.push(metrics.lanes.find(({ lane }) => lane === 'video')?.bytesWritten ?? -1)
    })
    const query = await transport.nextQuery()
    expect(query.key).toBe('blueos/v1/recorder/state/metrics')
    query.reply({
      kind: 'sample',
      sample: {
        key, payload: encodeCdr(messageSchema, recorderMetricsMessage(100)), encoding: cdrEncoding(messageSchema),
      },
    })
    await watching
    expect(videoBytes).toEqual([100])

    transport.publish({
      key, payload: encodeCdr(messageSchema, recorderMetricsMessage(250)), encoding: cdrEncoding(messageSchema),
    })
    expect(videoBytes).toEqual([100, 250])
  })
})
