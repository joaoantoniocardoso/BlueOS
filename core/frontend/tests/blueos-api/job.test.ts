/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus } from '@blueos-idl/constants'
import type { JobList } from '@blueos-idl/messages'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { jobsState } from '@/libs/blueos-api/endpoints'
import {
  isFinishedJobStatus, type JobFeedbackEntry, jobFromList, type JobResultEntry, watchJob, watchJobFeedback,
  watchJobResults,
} from '@/libs/blueos-api/job'
import {
  cdrEncoding, jobFeedbackKey, jobResultKey, jobsKey,
} from '@/libs/blueos-api/keys'
import type { MessageForSchema, SchemaName } from '@/libs/blueos-api/types'

import FakeTransport from './fake-transport'

function jobList(jobs: JobList['jobs']): MessageForSchema<'blueos_msgs/msg/JobList'> {
  return { jobs }
}

function jobsSample(list: JobList): ReturnType<typeof sample> {
  return sample(jobsKey('tank'), 'blueos_msgs/msg/JobList', list)
}

const LEVEL = 'blueos_example_msgs/msg/LevelQueryResponse'
const SET_LEVEL = 'blueos_example_msgs/msg/SetLevelRequest'

function feedbackSample(jobs: { job_id: string, level: number }[]): ReturnType<typeof sample> {
  return sample(jobFeedbackKey('tank', 'Fill'), 'blueos_msgs/msg/JobFeedbackList', {
    jobs: jobs.map(({ job_id, level }) => ({
      job_id, feedback: Array.from(encodeCdr(LEVEL, { level, max_level: 3 })),
    })),
  })
}

function sample<Schema extends SchemaName>(
  key: string,
  schema: Schema,
  message: MessageForSchema<Schema>,
): { key: string, payload: Uint8Array, encoding: string } {
  return { key, payload: encodeCdr(schema, message), encoding: cdrEncoding(schema) }
}

describe('jobFromList', () => {
  it('finds a Job by its id, a pending permission request included', () => {
    const pour = {
      job_id: 'job-1', job_type: 'Pour', status: JobStatusStatus.WaitingForPermission, reason: '',
    }

    expect(jobFromList(jobList([pour]), 'job-1')).toEqual(pour)
    expect(jobFromList(jobList([pour]), 'job-2')).toBeUndefined()
  })
})

describe('isFinishedJobStatus', () => {
  it('is true only for Succeeded, Canceled and Aborted', () => {
    expect(isFinishedJobStatus(JobStatusStatus.Succeeded)).toBe(true)
    expect(isFinishedJobStatus(JobStatusStatus.Canceled)).toBe(true)
    expect(isFinishedJobStatus(JobStatusStatus.Aborted)).toBe(true)
    expect(isFinishedJobStatus(JobStatusStatus.Accepted)).toBe(false)
    expect(isFinishedJobStatus(JobStatusStatus.WaitingForPermission)).toBe(false)
    expect(isFinishedJobStatus(JobStatusStatus.WaitingForResource)).toBe(false)
    expect(isFinishedJobStatus(JobStatusStatus.Executing)).toBe(false)
    expect(isFinishedJobStatus(JobStatusStatus.Paused)).toBe(false)
    expect(isFinishedJobStatus(JobStatusStatus.Canceling)).toBe(false)
  })
})

describe('watchJob', () => {
  it('does not subscribe when the ack carried no Job', async () => {
    const transport = new FakeTransport()
    const seen: (string | undefined)[] = []

    const watching = await watchJob(transport, 'tank', '', {
      onJob: (job) => seen.push(job?.job_id),
      onError: () => undefined,
    })

    expect(transport.subscribers).toEqual([])
    expect(seen).toEqual([])
    await watching.close()
  })

  it('closes the subscriber when the first reply already shows the Job finished', async () => {
    const transport = new FakeTransport()
    const succeeded = {
      job_id: 'job-4', job_type: 'Done', status: JobStatusStatus.Succeeded, reason: '',
    }

    const watching = watchJob(transport, 'tank', 'job-4', {
      onJob: () => undefined,
      onError: () => undefined,
    })
    const jobsQuery = await transport.nextQuery()
    jobsQuery.reply({ kind: 'sample', sample: jobsSample(jobList([succeeded])) })
    await watching

    expect(transport.subscribers).toHaveLength(1)
    expect(transport.subscribers[0].open).toBe(false)
  })

  it('follows a Job from Executing through Succeeded and then stops', async () => {
    const transport = new FakeTransport()
    const running = {
      job_id: 'job-1', job_type: 'Brew', status: JobStatusStatus.Executing, reason: '',
    }
    const succeeded = { ...running, status: JobStatusStatus.Succeeded }
    const statuses: number[] = []

    const watching = watchJob(transport, 'tank', 'job-1', {
      onJob: (job) => statuses.push(job?.status ?? -1),
      onError: () => undefined,
    })
    const jobsQuery = await transport.nextQuery()
    expect(jobsQuery.key).toBe(jobsState('tank').key)
    jobsQuery.reply({ kind: 'sample', sample: jobsSample(jobList([running])) })
    const subscription = await watching

    transport.publish(jobsSample(jobList([succeeded])))
    await Promise.resolve()

    expect(statuses).toEqual([JobStatusStatus.Executing, JobStatusStatus.Succeeded])
    await subscription.close()
    statuses.length = 0
    transport.publish(jobsSample(jobList([{ ...running, status: JobStatusStatus.Aborted }])))
    await Promise.resolve()
    expect(statuses).toEqual([])
  })

  it('ends when the Job leaves the jobs State', async () => {
    const transport = new FakeTransport()
    const running = {
      job_id: 'job-2', job_type: 'Drain', status: JobStatusStatus.Executing, reason: '',
    }
    const seen: (string | undefined)[] = []

    const watching = watchJob(transport, 'tank', 'job-2', {
      onJob: (job) => seen.push(job?.job_id),
      onError: () => undefined,
    })
    const jobsQuery = await transport.nextQuery()
    jobsQuery.reply({ kind: 'sample', sample: jobsSample(jobList([running])) })
    const subscription = await watching

    transport.publish(jobsSample(jobList([])))
    await Promise.resolve()

    expect(seen).toEqual(['job-2', undefined])
    await subscription.close()
  })

  it('follows a Job through Aborted and Canceled', async () => {
    const root = {
      job_id: 'job-3', job_type: 'Step', status: JobStatusStatus.Executing, reason: '',
    }
    const terminalStatuses: number[] = []

    async function runToTerminal(status: number): Promise<void> {
      const jobTransport = new FakeTransport()
      const watching = watchJob(jobTransport, 'tank', root.job_id, {
        onJob: (job) => {
          if (job !== undefined && isFinishedJobStatus(job.status)) {
            terminalStatuses.push(job.status)
          }
        },
        onError: () => undefined,
      })
      const jobsQuery = await jobTransport.nextQuery()
      jobsQuery.reply({ kind: 'sample', sample: jobsSample(jobList([root])) })
      const subscription = await watching
      jobTransport.publish(jobsSample(jobList([{ ...root, status }])))
      await Promise.resolve()
      await subscription.close()
    }

    await runToTerminal(JobStatusStatus.Aborted)
    await runToTerminal(JobStatusStatus.Canceled)

    expect(terminalStatuses).toEqual([JobStatusStatus.Aborted, JobStatusStatus.Canceled])
  })
})

describe('watchJobFeedback', () => {
  it('sees the latest Feedback of each active Job when it opens mid-Job, then every update', async () => {
    const transport = new FakeTransport()
    const seen: JobFeedbackEntry<MessageForSchema<typeof LEVEL>>[][] = []

    const watching = watchJobFeedback(transport, 'tank', 'Fill', LEVEL, {
      onValue: (feedback) => seen.push(feedback),
      onError: (error) => { throw error },
    })
    const feedbackQuery = await transport.nextQuery()
    expect(feedbackQuery.key).toBe(jobFeedbackKey('tank', 'Fill'))
    feedbackQuery.reply({ kind: 'sample', sample: feedbackSample([{ job_id: 'job-1', level: 2 }]) })
    const subscription = await watching
    transport.publish(feedbackSample([{ job_id: 'job-1', level: 3 }, { job_id: 'job-2', level: 1 }]))
    transport.publish(feedbackSample([]))

    expect(seen).toEqual([
      [{ jobId: 'job-1', feedback: { level: 2, max_level: 3 } }],
      [
        { jobId: 'job-1', feedback: { level: 3, max_level: 3 } },
        { jobId: 'job-2', feedback: { level: 1, max_level: 3 } },
      ],
      [],
    ])
    await subscription.close()
  })
})

describe('watchJobResults', () => {
  it('decodes how each Job ended and its Job result', async () => {
    const transport = new FakeTransport()
    const seen: JobResultEntry<MessageForSchema<typeof SET_LEVEL>>[] = []
    const canceled = {
      job_id: 'job-1', job_type: 'Fill', status: JobStatusStatus.Canceled, reason: '',
    }

    const subscription = await watchJobResults(transport, 'tank', 'Fill', SET_LEVEL, {
      onValue: (result) => seen.push(result),
      onError: (error) => { throw error },
    })
    transport.publish(sample(jobResultKey('tank', 'Fill'), 'blueos_msgs/msg/JobResult', {
      job: canceled, result: Array.from(encodeCdr(SET_LEVEL, { level: 2 })),
    }))

    expect(transport.subscribers[0].key).toBe(jobResultKey('tank', 'Fill'))
    expect(seen).toEqual([{ job: canceled, result: { level: 2 } }])
    await subscription.close()
  })
})
