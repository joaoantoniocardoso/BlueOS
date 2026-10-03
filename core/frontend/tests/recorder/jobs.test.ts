/* eslint-disable import/no-extraneous-dependencies */
import { CommandAckStatus, JobStatusStatus } from '@blueos-idl/constants'
import type { JobStatus } from '@blueos-idl/messages'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import {
  cdrEncoding, jobFeedbackKey, jobResultKey, jobsKey,
} from '@/libs/blueos-api/keys'
import {
  DeleteRecording, NAME, RepairRecording, Start, Stop,
} from '@/libs/blueos-api/services/recorder'
import type { Sample } from '@/libs/blueos-api/transport'
import { createRecorderClient, type RecorderClient } from '@/libs/recorder/client'
import type { LibraryRecording, RecorderCommandResult } from '@/libs/recorder/types'
import { withRepairJobs } from '@/libs/recorder/view-logic'

import FakeTransport from '../blueos-api/fake-transport'

const JOB_ID = '0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10'
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/

function ack(accepted: boolean, status: number, reason = ''): Uint8Array {
  return encodeCdr('blueos_msgs/msg/CommandAck', {
    accepted, job_id: JOB_ID, status, reason,
  })
}

type SampleSchema = 'blueos_msgs/msg/JobFeedbackList' | 'blueos_msgs/msg/JobList' | 'blueos_msgs/msg/JobResult'

function sample(key: string, schema: SampleSchema, message: object): Sample {
  return {
    key,
    payload: encodeCdr(schema, message as never),
    encoding: cdrEncoding(schema),
  }
}

function repairFeedback(entries: { job_id: string, bytes_processed: number, total_bytes: number }[]): Sample {
  return sample(jobFeedbackKey(NAME, RepairRecording.name), 'blueos_msgs/msg/JobFeedbackList', {
    jobs: entries.map(({ job_id, bytes_processed, total_bytes }) => ({
      job_id,
      feedback: Array.from(encodeCdr(RepairRecording.feedbackSchema, { bytes_processed, total_bytes })),
    })),
  })
}

function libraryRow(overrides: Partial<LibraryRecording> = {}): LibraryRecording {
  return {
    path: 'broken.mcap',
    name: 'broken.mcap',
    size_bytes: 100,
    created: 1,
    state: 'repairing',
    repair_bytes_processed: 0,
    repair_total_bytes: 0,
    repair_bytes_per_second: 0,
    repair_error: '',
    repair_job_id: JOB_ID,
    allowed_operations: ['CancelJob'],
    ...overrides,
  }
}

const SUBMISSIONS: [string, (client: RecorderClient) => Promise<RecorderCommandResult>, string][] = [
  ['repairRecording', (client) => client.repairRecording('a.mcap'), RepairRecording.key],
  ['deleteRecording', (client) => client.deleteRecording('a.mcap'), DeleteRecording.key],
  ['startRecording', (client) => client.startRecording(true), Start.key],
  ['stopRecording', (client) => client.stopRecording(), Stop.key],
]

describe('recorder Job submission', () => {
  it.each(SUBMISSIONS)('%s submits a Job under a generated id, returns the Job id', async (_name, submit, key) => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    const result = submit(client)
    const sent = await transport.nextQuery()
    expect(sent.key).toBe(key)
    expect(new TextDecoder().decode(sent.body?.attachment)).toMatch(UUID)
    sent.reply({
      kind: 'sample',
      sample: {
        key,
        payload: ack(true, CommandAckStatus.Executing),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    expect(await result).toEqual({
      accepted: true, reason: '', job_id: JOB_ID, status: CommandAckStatus.Executing,
    })
  })

  it('submits one Job per call, each under its own id', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    client.deleteRecording('a.mcap').catch(() => undefined)
    client.deleteRecording('b.mcap').catch(() => undefined)
    const first = await transport.nextQuery()
    const second = await transport.nextQuery()

    expect(new TextDecoder().decode(first.body?.attachment))
      .not.toBe(new TextDecoder().decode(second.body?.attachment))
  })

  it('returns a rejected submission with its reason', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    const result = client.repairRecording('../outside.mcap')
    const sent = await transport.nextQuery()
    sent.reply({
      kind: 'sample',
      sample: {
        key: sent.key,
        payload: ack(false, CommandAckStatus.StatusUnknown, 'Invalid recording path.'),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    expect(await result).toMatchObject({ accepted: false, reason: 'Invalid recording path.' })
  })
})

describe('recorder Job watching', () => {
  it('sees repair Feedback of a Job already running when the page opens, then every update', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const seen: Record<string, { bytes_processed: number, total_bytes: number }>[] = []

    const watching = client.watchRepairProgress((progress) => seen.push(progress))
    const query = await transport.nextQuery()
    expect(query.key).toBe(jobFeedbackKey(NAME, RepairRecording.name))
    query.reply({
      kind: 'sample',
      sample: repairFeedback([{ job_id: JOB_ID, bytes_processed: 30, total_bytes: 90 }]),
    })
    await watching
    transport.publish(repairFeedback([{ job_id: JOB_ID, bytes_processed: 60, total_bytes: 90 }]))
    transport.publish(repairFeedback([]))

    expect(seen).toEqual([
      { [JOB_ID]: { bytes_processed: 30, total_bytes: 90 } },
      { [JOB_ID]: { bytes_processed: 60, total_bytes: 90 } },
      {},
    ])
  })

  it('forwards a Feedback that does not decode to onError', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const errors: unknown[] = []

    const watching = client.watchRepairProgress(() => undefined, (error) => errors.push(error))
    const query = await transport.nextQuery()
    query.reply({ kind: 'sample', sample: repairFeedback([]) })
    await watching
    transport.publish(sample(
      jobFeedbackKey(NAME, RepairRecording.name),
      'blueos_msgs/msg/JobFeedbackList',
      { jobs: [{ job_id: JOB_ID, feedback: [1, 2] }] },
    ))

    expect(errors).toHaveLength(1)
  })

  it('watches the jobs State', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const seen: { job_id: string, status: number }[][] = []
    function jobStatus(status: number): { jobs: JobStatus[] } {
      return {
        jobs: [{
          job_id: JOB_ID, job_type: RepairRecording.name, status, reason: '',
        }],
      }
    }

    const watching = client.watchJobs((jobs) => seen.push(jobs.map(({ job_id, status }) => ({ job_id, status }))))
    const query = await transport.nextQuery()
    expect(query.key).toBe(jobsKey(NAME))
    query.reply({
      kind: 'sample',
      sample: sample(jobsKey(NAME), 'blueos_msgs/msg/JobList', jobStatus(JobStatusStatus.Executing)),
    })
    await watching
    transport.publish(sample(jobsKey(NAME), 'blueos_msgs/msg/JobList', jobStatus(JobStatusStatus.Canceling)))

    expect(seen).toEqual([
      [{ job_id: JOB_ID, status: JobStatusStatus.Executing }],
      [{ job_id: JOB_ID, status: JobStatusStatus.Canceling }],
    ])
  })

  it('reports how a repair Job ended, with its reason', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const operations: unknown[] = []

    await client.watchOperations((operation) => operations.push(operation))
    transport.publish(sample(jobResultKey(NAME, RepairRecording.name), 'blueos_msgs/msg/JobResult', {
      job: {
        job_id: JOB_ID, job_type: RepairRecording.name, status: JobStatusStatus.Aborted, reason: 'disk full',
      },
      result: Array.from(encodeCdr(RepairRecording.resultSchema, { path: 'broken.mcap' })),
    }))

    expect(operations).toEqual([{
      job: {
        job_id: JOB_ID, job_type: RepairRecording.name, status: JobStatusStatus.Aborted, reason: 'disk full',
      },
      result: { path: 'broken.mcap' },
    }])
  })
})

describe('withRepairJobs', () => {
  it('shows the Feedback of the repair Job in the repairing row', () => {
    const [row] = withRepairJobs(
      [libraryRow()],
      { [JOB_ID]: { bytes_processed: 30, total_bytes: 90 } },
      [],
    )

    expect(row).toMatchObject({ repair_bytes_processed: 30, repair_total_bytes: 90 })
  })

  it('leaves a row without a repair Job as it is', () => {
    const idle = libraryRow({ repair_job_id: '', state: 'ready', allowed_operations: ['DeleteRecording'] })

    expect(withRepairJobs([idle], { [JOB_ID]: { bytes_processed: 1, total_bytes: 2 } }, [])).toEqual([idle])
  })

  it('offers no second cancel while the repair Job is canceling', () => {
    const jobs = [{
      job_id: JOB_ID, job_type: RepairRecording.name, status: JobStatusStatus.Canceling, reason: '',
    }]

    expect(withRepairJobs([libraryRow()], {}, jobs)[0].allowed_operations).toEqual([])
  })
})
