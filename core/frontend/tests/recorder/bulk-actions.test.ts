import { CommandAckStatus } from '@blueos-idl/constants'
import { describe, expect, it, vi } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { DeleteRecording, RepairRecording } from '@/libs/blueos-api/services/recorder'
import { createRecorderClient } from '@/libs/recorder/client'
import {
  bulkActionTargets,
  bulkDeleteSubmissionFailureMessages,
  bulkOperationFailureMessages,
  submitBulkDelete,
  submitBulkRepair,
} from '@/libs/recorder/bulk-actions'
import { DELETE_RECORDING, REPAIR_RECORDING } from '@/libs/recorder/constants'
import type { LibraryRecording, RecorderCommandResult, RecordingOperationEvent } from '@/libs/recorder/types'
import { operationFailureMessage } from '@/libs/recorder/view-logic'

import FakeTransport from '../blueos-api/fake-transport'

const JOB_A = '11111111-1111-4111-8111-111111111111'
const JOB_B = '22222222-2222-4222-8222-222222222222'
const JOB_C = '33333333-3333-4333-8333-333333333333'

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

describe('recorder bulk submission', () => {
  it('submits one delete Job per selected path through the client', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    const pending = submitBulkDelete(['a.mcap', 'b.mcap'], (path) => client.deleteRecording(path))
    const first = await transport.nextQuery()
    first.reply({
      kind: 'sample',
      sample: {
        key: first.key,
        payload: ack(JOB_A, true),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })
    const second = await transport.nextQuery()
    second.reply({
      kind: 'sample',
      sample: {
        key: second.key,
        payload: ack(JOB_B, true),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    expect(first.key).toBe(DeleteRecording.key)
    expect(second.key).toBe(DeleteRecording.key)
    expect(await pending).toEqual([
      {
        path: 'a.mcap', accepted: true, reason: '', job_id: JOB_A, transportError: false,
      },
      {
        path: 'b.mcap', accepted: true, reason: '', job_id: JOB_B, transportError: false,
      },
    ])
  })

  it('submits one repair Job per selected path through the client', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    const pending = submitBulkRepair(['broken.mcap'], (path) => client.repairRecording(path))
    const sent = await transport.nextQuery()
    sent.reply({
      kind: 'sample',
      sample: {
        key: sent.key,
        payload: ack(JOB_C, true),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    expect(sent.key).toBe(RepairRecording.key)
    expect(await pending).toEqual([
      {
        path: 'broken.mcap', accepted: true, reason: '', job_id: JOB_C, transportError: false,
      },
    ])
  })

  it('keeps every path outcome when another submission fails or is rejected', async () => {
    const deleteRecording = vi.fn(async (path: string): Promise<RecorderCommandResult> => {
      if (path === 'a.mcap') {
        return { accepted: true, reason: '', job_id: JOB_A }
      }
      if (path === 'b.mcap') {
        return { accepted: false, reason: 'Busy.', job_id: '' }
      }
      throw new Error('Transport down')
    })

    const outcomes = await submitBulkDelete(['a.mcap', 'b.mcap', 'c.mcap'], deleteRecording)

    expect(deleteRecording).toHaveBeenCalledTimes(3)
    expect(outcomes).toEqual([
      {
        path: 'a.mcap', accepted: true, reason: '', job_id: JOB_A, transportError: false,
      },
      {
        path: 'b.mcap', accepted: false, reason: 'Busy.', job_id: '', transportError: false,
      },
      {
        path: 'c.mcap', accepted: false, reason: 'Transport down', job_id: '', transportError: true,
      },
    ])
    expect(bulkDeleteSubmissionFailureMessages(outcomes)).toEqual([
      'Delete rejected for b.mcap: Busy.',
      'Delete failed for c.mcap: Transport down',
    ])
  })
})

describe('recorder bulk Job results', () => {
  it('reports every failed Job without hiding the others', () => {
    const events: RecordingOperationEvent[] = [
      {
        operation: 'delete',
        path: 'a.mcap',
        output_path: '',
        succeeded: false,
        cancelled: false,
        error: 'disk full',
      },
      {
        operation: 'delete',
        path: 'b.mcap',
        output_path: '',
        succeeded: true,
        cancelled: false,
        error: '',
      },
      {
        operation: 'delete',
        path: 'c.mcap',
        output_path: '',
        succeeded: false,
        cancelled: false,
        error: 'permission denied',
      },
    ]
    const bulkPaths = new Set(['a.mcap', 'b.mcap', 'c.mcap'])

    expect(bulkOperationFailureMessages(events, bulkPaths)).toEqual([
      operationFailureMessage(events[0], 'a.mcap'),
      operationFailureMessage(events[2], 'c.mcap'),
    ])
  })

  it('limits bulk Job failures to the paths in the bulk action', () => {
    const event: RecordingOperationEvent = {
      operation: 'repair',
      path: 'other.mcap',
      output_path: '',
      succeeded: false,
      cancelled: false,
      error: 'nope',
    }

    expect(bulkOperationFailureMessages([event], new Set(['target.mcap']))).toEqual([])
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
