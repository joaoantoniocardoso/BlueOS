/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus } from '@blueos-idl/constants'
import type { JobList, JobStatus } from '@blueos-idl/messages'

import { JOB_ID_NONE } from './command'
import { jobsState } from './endpoints'
import type { Subscription, Transport } from './transport'
import { watchState } from './watch'

/** Receives every update of one root Job until it finishes or leaves the jobs State. */
export interface JobObserver {
  onJob: (job: JobStatus | undefined) => void
  onError: (error: unknown) => void
}

/** True when the Job will not change again in the jobs State. */
export function isFinishedJobStatus(status: number): boolean {
  return status === JobStatusStatus.Succeeded
    || status === JobStatusStatus.Failed
    || status === JobStatusStatus.Cancelled
}

/** Finds one Job in a `JobList` snapshot by its root id. */
export function jobFromList(list: JobList, jobId: number): JobStatus | undefined {
  return list.jobs.find((job) => job.job_id === jobId)
}

/**
 * Follows the root `job_id` from a Command ack through the Service's `jobs` State until the Job reaches Succeeded,
 * Failed or Cancelled, or disappears when retention drops it (D-12). When `jobId` is `JOB_ID_NONE`, nothing is
 * watched.
 */
export async function watchJob(
  transport: Transport,
  service: string,
  jobId: number,
  observer: JobObserver,
): Promise<Subscription> {
  if (jobId === JOB_ID_NONE) {
    return { close: async () => Promise.resolve() }
  }

  let finished = false
  let watchReady = false
  const subscription = await watchState(transport, jobsState(service), {
    onValue: (list) => {
      if (finished) {
        return
      }
      const job = jobFromList(list, jobId)
      if (job === undefined) {
        finished = true
        observer.onJob(undefined)
        if (watchReady) {
          subscription.close().catch(() => undefined)
        }
        return
      }
      observer.onJob(job)
      if (isFinishedJobStatus(job.status)) {
        finished = true
        if (watchReady) {
          subscription.close().catch(() => undefined)
        }
      }
    },
    onError: (error) => observer.onError(error),
  })
  watchReady = true
  if (finished) {
    await subscription.close()
  }
  return subscription
}
