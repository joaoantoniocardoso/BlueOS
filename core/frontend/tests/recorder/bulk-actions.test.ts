/* eslint-disable import/no-extraneous-dependencies */
import { CommandAckStatus, JobStatusStatus } from '@blueos-idl/constants'
import { describe, expect, it, vi } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { DeleteRecording, RepairRecording } from '@/libs/blueos-api/services/recorder'
import { createRecorderClient } from '@/libs/recorder/client'
import {
  type BulkAction,
  bulkActionTargets,
  bulkJobEnded,
  runBulkAction,
} from '@/libs/recorder/bulk-actions'
import { DELETE_RECORDING, REPAIR_RECORDING } from '@/libs/recorder/constants'
import type { LibraryRecording, RecorderCommandResult, RecordingJobResult } from '@/libs/recorder/types'

import FakeTransport from '../blueos-api/fake-transport'

const JOB_A = '11111111-1111-4111-8111-111111111111'
const JOB_B = '22222222-2222-4222-8222-222222222222'

function ack(jobId: string, accepted: boolean, reason = ''): Uint8Array {
  return encodeCdr('blueos_msgs/msg/CommandAck', {
    accepted,
    job_id: jobId,
    status: CommandAckStatus.Executing,
    reason,
  })
}

function file(path: string, operations: string[]): LibraryRecording {
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
    allowed_operations: operations,
  }
}

function jobResult(path: string, status: JobStatusStatus, reason = ''): RecordingJobResult {
  return {
    job: {
      job_id: JOB_A, job_type: DELETE_RECORDING, status, reason,
    },
    result: { path },
  }
}

describe('recorder bulk submission', () => {
  it('submits one Job per target that allows the operation, through the client', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const action: BulkAction = { failures: [], pending: ['a.mcap', 'b.mcap'] }

    const running = runBulkAction(action, DELETE_RECORDING, (path) => client.deleteRecording(path))
    for (const jobId of [JOB_A, JOB_B]) {
      // eslint-disable-next-line no-await-in-loop
      const sent = await transport.nextQuery()
      expect(sent.key).toBe(DeleteRecording.key)
      sent.reply({
        kind: 'sample',
        sample: { key: sent.key, payload: ack(jobId, true), encoding: cdrEncoding('blueos_msgs/msg/CommandAck') },
      })
    }
    await running

    expect(action).toEqual({ failures: [], pending: ['a.mcap', 'b.mcap'] })
  })

  it('keeps going when a submission is rejected or fails, and says so', async () => {
    const repairRecording = vi.fn(async (path: string): Promise<RecorderCommandResult> => {
      if (path === 'a.mcap') {
        return {
          accepted: true, reason: '', job_id: JOB_A, status: CommandAckStatus.Executing,
        }
      }
      if (path === 'b.mcap') {
        return {
          accepted: false, reason: 'Busy.', job_id: '', status: CommandAckStatus.StatusUnknown,
        }
      }
      throw new Error('Transport down')
    })
    const action: BulkAction = { failures: [], pending: ['a.mcap', 'b.mcap', 'c.mcap'] }

    await runBulkAction(action, REPAIR_RECORDING, repairRecording)

    expect(repairRecording).toHaveBeenCalledTimes(3)
    expect(action).toEqual({
      failures: ['Repair rejected for b.mcap: Busy.', 'Repair failed for c.mcap: Transport down'],
      pending: ['a.mcap'],
    })
  })

  it('does not wait for the ack of a path whose Job already ended', async () => {
    const action: BulkAction = { failures: [], pending: ['a.mcap'] }

    await runBulkAction(action, DELETE_RECORDING, async (path) => {
      bulkJobEnded(action, jobResult(path, JobStatusStatus.Succeeded))
      return {
        accepted: true, reason: '', job_id: JOB_A, status: CommandAckStatus.Executing,
      }
    })

    expect(action).toEqual({ failures: [], pending: [] })
  })

  it('reports a repair the client says was rejected', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const action: BulkAction = { failures: [], pending: ['broken.mcap'] }

    const running = runBulkAction(action, REPAIR_RECORDING, (path) => client.repairRecording(path))
    const sent = await transport.nextQuery()
    sent.reply({
      kind: 'sample',
      sample: { key: sent.key, payload: ack(JOB_A, false, 'Busy.'), encoding: cdrEncoding('blueos_msgs/msg/CommandAck') },
    })
    await running

    expect(sent.key).toBe(RepairRecording.key)
    expect(action).toEqual({ failures: ['Repair rejected for broken.mcap: Busy.'], pending: [] })
  })
})

describe('recorder bulk Job results', () => {
  it('reports a delete whose Job aborted without hiding the others', () => {
    const action: BulkAction = { failures: [], pending: ['a.mcap', 'b.mcap', 'c.mcap'] }

    expect(bulkJobEnded(action, jobResult('a.mcap', JobStatusStatus.Aborted, 'invalid recording path'))).toBe(true)
    expect(bulkJobEnded(action, jobResult('b.mcap', JobStatusStatus.Succeeded))).toBe(true)
    expect(bulkJobEnded(action, jobResult('c.mcap', JobStatusStatus.Aborted, 'permission denied'))).toBe(true)

    expect(action).toEqual({
      failures: ['Delete failed for a.mcap: invalid recording path', 'Delete failed for c.mcap: permission denied'],
      pending: [],
    })
  })

  it('leaves a Job result of a path outside the bulk action alone', () => {
    const action: BulkAction = { failures: [], pending: ['target.mcap'] }

    expect(bulkJobEnded(action, jobResult('other.mcap', JobStatusStatus.Aborted, 'nope'))).toBe(false)
    expect(action).toEqual({ failures: [], pending: ['target.mcap'] })
  })

  it('targets only recordings that allow the bulk operation', () => {
    const recordings = [
      file('ready.mcap', [DELETE_RECORDING]),
      file('recording.mcap', []),
      file('repair.mcap', [REPAIR_RECORDING]),
    ]

    expect(bulkActionTargets(recordings, DELETE_RECORDING).map((entry) => entry.path)).toEqual(['ready.mcap'])
    expect(bulkActionTargets(recordings, REPAIR_RECORDING).map((entry) => entry.path)).toEqual(['repair.mcap'])
  })
})
