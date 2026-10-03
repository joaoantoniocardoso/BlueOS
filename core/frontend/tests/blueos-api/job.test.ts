/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus } from '@blueos-idl/constants'
import type { JobList } from '@blueos-idl/messages'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { jobsState } from '@/libs/blueos-api/endpoints'
import { isFinishedJobStatus, jobFromList, watchJob } from '@/libs/blueos-api/job'
import { cdrEncoding, jobsKey } from '@/libs/blueos-api/keys'
import type { MessageForSchema } from '@/libs/blueos-api/types'

import FakeTransport from './fake-transport'

function jobList(jobs: JobList['jobs']): MessageForSchema<'blueos_msgs/msg/JobList'> {
  return { jobs }
}

function jobsSample(list: JobList): ReturnType<typeof sample> {
  return sample(jobsKey('tank'), 'blueos_msgs/msg/JobList', list)
}

function sample<Schema extends 'blueos_msgs/msg/JobList'>(
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
