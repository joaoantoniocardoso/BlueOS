/* eslint-disable import/no-extraneous-dependencies */
import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding, serviceLivelinessKey } from '@/libs/blueos-api/keys'
import {
  CancelRepair,
  DeleteRecording,
  library,
  operation,
  RepairRecording,
  SnapshotRecording,
} from '@/libs/blueos-api/services/recorder'
import { createRecorderClient, SNAPSHOT_WAIT_TIMEOUT_MS } from '@/libs/recorder/client'

import FakeTransport from '../blueos-api/fake-transport'

const idleLibrary = { files: [] as never[] }

function recordingFile(overrides: Record<string, unknown> = {}): Record<string, unknown> {
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
    allowed_operations: ['SnapshotRecording'],
    ...overrides,
  }
}

function operationEvent(outputPath: string): Record<string, unknown> {
  return {
    operation: 1,
    path: 'live.mcap',
    output_path: outputPath,
    succeeded: true,
    cancelled: false,
    error: '',
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
        files: [recordingFile({
          path: 'a.mcap', name: 'a.mcap', state: 1, allowed_operations: ['DeleteRecording'],
        })],
      }),
      encoding: cdrEncoding(library.messageSchema),
    })

    expect(running).toEqual([true])
    expect(recordings).toEqual([[], ['a.mcap']])
  })

  it('resolves snapshot before the command ack when the operation arrives early', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const operationWatch = client.watchOperations(() => undefined)
    await operationWatch

    const newSnapshot = 'live.snapshot-2024-01-02T03-04-05Z.mcap'
    const snapshot = client.snapshotRecording('live.mcap')
    const snapshotQuery = await transport.nextQuery()
    expect(snapshotQuery.key).toBe(SnapshotRecording.key)

    transport.publish({
      key: operation.key,
      payload: encodeCdr(operation.messageSchema, operationEvent(newSnapshot)),
      encoding: cdrEncoding(operation.messageSchema),
    })

    snapshotQuery.reply({
      kind: 'sample',
      sample: {
        key: SnapshotRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: 3,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    await expect(snapshot).resolves.toBe(newSnapshot)
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
          job_id: 3,
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
          job_id: 3,
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
          job_id: 0,
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
          job_id: 3,
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
          job_id: 1,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })
    expect(await repair).toMatchObject({ accepted: true })

    const cancel = client.cancelRepair('broken.mcap')
    const cancelQuery = await transport.nextQuery()
    expect(cancelQuery.key).toBe(CancelRepair.key)
    cancelQuery.reply({
      kind: 'sample',
      sample: {
        key: CancelRepair.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: false,
          job_id: 0,
          reason: 'not repairing',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })
    expect(await cancel).toMatchObject({ accepted: false, reason: 'not repairing' })

    const deleted = client.deleteRecording('old.mcap')
    const deleteQuery = await transport.nextQuery()
    expect(deleteQuery.key).toBe(DeleteRecording.key)
    deleteQuery.reply({
      kind: 'sample',
      sample: {
        key: DeleteRecording.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: 2,
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
})
