/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus } from '@blueos-idl/constants'
import type { JobList } from '@blueos-idl/messages'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { JOB_ID_NONE } from '@/libs/blueos-api/command'
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
  it('finds a Job by its root id', () => {
    const brew = {
      job_id: 1, parent_job_id: 0, status: JobStatusStatus.Running, name: 'brew',
    }

    expect(jobFromList(jobList([brew]), 1)).toEqual(brew)
    expect(jobFromList(jobList([brew]), 2)).toBeUndefined()
  })
})

describe('isFinishedJobStatus', () => {
  it('is true only for Succeeded, Failed and Cancelled', () => {
    expect(isFinishedJobStatus(JobStatusStatus.Succeeded)).toBe(true)
    expect(isFinishedJobStatus(JobStatusStatus.Failed)).toBe(true)
    expect(isFinishedJobStatus(JobStatusStatus.Cancelled)).toBe(true)
    expect(isFinishedJobStatus(JobStatusStatus.Running)).toBe(false)
    expect(isFinishedJobStatus(JobStatusStatus.Queued)).toBe(false)
    expect(isFinishedJobStatus(JobStatusStatus.Cancelling)).toBe(false)
  })
})

describe('watchJob', () => {
  it('does not subscribe when the ack carried no Job', async () => {
    const transport = new FakeTransport()
    const seen: (number | undefined)[] = []

    const watching = await watchJob(transport, 'tank', JOB_ID_NONE, {
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
      job_id: 4, parent_job_id: 0, status: JobStatusStatus.Succeeded, name: 'done',
    }

    const watching = watchJob(transport, 'tank', 4, {
      onJob: () => undefined,
      onError: () => undefined,
    })
    const jobsQuery = await transport.nextQuery()
    jobsQuery.reply({ kind: 'sample', sample: jobsSample(jobList([succeeded])) })
    await watching

    expect(transport.subscribers).toHaveLength(1)
    expect(transport.subscribers[0].open).toBe(false)
  })

  it('follows a Job from Running through Succeeded and then stops', async () => {
    const transport = new FakeTransport()
    const running = {
      job_id: 1, parent_job_id: 0, status: JobStatusStatus.Running, name: 'brew',
    }
    const succeeded = { ...running, status: JobStatusStatus.Succeeded }
    const statuses: number[] = []

    const watching = watchJob(transport, 'tank', 1, {
      onJob: (job) => statuses.push(job?.status ?? -1),
      onError: () => undefined,
    })
    const jobsQuery = await transport.nextQuery()
    expect(jobsQuery.key).toBe(jobsState('tank').key)
    jobsQuery.reply({ kind: 'sample', sample: jobsSample(jobList([running])) })
    const subscription = await watching

    transport.publish(jobsSample(jobList([succeeded])))
    await Promise.resolve()

    expect(statuses).toEqual([JobStatusStatus.Running, JobStatusStatus.Succeeded])
    await subscription.close()
    statuses.length = 0
    transport.publish(jobsSample(jobList([{ ...running, status: JobStatusStatus.Failed }])))
    await Promise.resolve()
    expect(statuses).toEqual([])
  })

  it('ends when the Job leaves the jobs State', async () => {
    const transport = new FakeTransport()
    const running = {
      job_id: 2, parent_job_id: 0, status: JobStatusStatus.Running, name: 'drain',
    }
    const seen: (number | undefined)[] = []

    const watching = watchJob(transport, 'tank', 2, {
      onJob: (job) => seen.push(job?.job_id),
      onError: () => undefined,
    })
    const jobsQuery = await transport.nextQuery()
    jobsQuery.reply({ kind: 'sample', sample: jobsSample(jobList([running])) })
    const subscription = await watching

    transport.publish(jobsSample(jobList([])))
    await Promise.resolve()

    expect(seen).toEqual([2, undefined])
    await subscription.close()
  })

  it('follows a Job through Failed and Cancelled', async () => {
    const root = {
      job_id: 3, parent_job_id: 0, status: JobStatusStatus.Running, name: 'step',
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

    await runToTerminal(JobStatusStatus.Failed)
    await runToTerminal(JobStatusStatus.Cancelled)

    expect(terminalStatuses).toEqual([JobStatusStatus.Failed, JobStatusStatus.Cancelled])
  })
})
