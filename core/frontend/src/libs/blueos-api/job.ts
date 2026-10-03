/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus } from '@blueos-idl/constants'
import type { JobList, JobStatus } from '@blueos-idl/messages'

import { jobsState } from './endpoints'
import type { Subscription, Transport } from './transport'
import { watchState } from './watch'

/** Receives every update of one Job until it ends or leaves the jobs State. */
export interface JobObserver {
  onJob: (job: JobStatus | undefined) => void
  onError: (error: unknown) => void
}

/** True when the Job has ended and will not change again in the jobs State. */
export function isFinishedJobStatus(status: number): boolean {
  return status === JobStatusStatus.Succeeded
    || status === JobStatusStatus.Canceled
    || status === JobStatusStatus.Aborted
}

/** Finds one Job in a `JobList` snapshot by its id. */
export function jobFromList(list: JobList, jobId: string): JobStatus | undefined {
  return list.jobs.find((job) => job.job_id === jobId)
}

/**
 * Follows the `job_id` of a Command ack through the Service's `jobs` State until the Job ends Succeeded, Canceled or
 * Aborted, or disappears when retention drops it (D-12, D-36). An empty `jobId`, the ack of a Command that is no Job,
 * watches nothing.
 */
export async function watchJob(
  transport: Transport,
  service: string,
  jobId: string,
  observer: JobObserver,
): Promise<Subscription> {
  if (jobId === '') {
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
