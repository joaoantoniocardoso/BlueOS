/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus } from '@blueos-idl/constants'
import type { JobList, JobStatus } from '@blueos-idl/messages'

import { decodeCdr } from './cdr'
import { jobFeedbackState, jobResultEvent, jobsState } from './endpoints'
import type { Subscription, Transport } from './transport'
import type { MessageForSchema, SchemaName } from './types'
import { type Observer, watchState } from './watch'
import { watchEvent } from './watch-event'

/** Receives every update of one Job until it ends or leaves the jobs State. */
export interface JobObserver {
  onJob: (job: JobStatus | undefined) => void
  onError: (error: unknown) => void
}

/** The latest Feedback of one active Job, decoded as its Job type's Feedback message. */
export interface JobFeedbackEntry<Message> {
  jobId: string
  feedback: Message
}

/** How one Job ended, with its Job result decoded as its Job type's result message. */
export interface JobResultEntry<Message> {
  job: JobStatus
  result: Message
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

/**
 * Watches the Feedback State of the Job type `jobType`: the latest Feedback of each of its active Jobs, decoded as
 * `feedbackSchema`, in the order the Jobs were submitted. A client that opens it mid-Job sees the latest Feedback at
 * once, and a Job leaves the list when it ends (D-12, D-36).
 */
export async function watchJobFeedback<Schema extends SchemaName>(
  transport: Transport,
  service: string,
  jobType: string,
  feedbackSchema: Schema,
  observer: Observer<JobFeedbackEntry<MessageForSchema<Schema>>[]>,
): Promise<Subscription> {
  return watchState(transport, jobFeedbackState(service, jobType), {
    onValue: (list, key) => {
      let entries: JobFeedbackEntry<MessageForSchema<Schema>>[]
      try {
        entries = list.jobs.map((job) => ({
          jobId: job.job_id,
          feedback: decodeCdr(feedbackSchema, job.feedback),
        }))
      } catch (error) {
        observer.onError(error)
        return
      }
      observer.onValue(entries, key)
    },
    onError: (error) => observer.onError(error),
  })
}

/** Receives how each Job of the Job type `jobType` ends, with its Job result decoded as `resultSchema` (D-12, D-36). */
export async function watchJobResults<Schema extends SchemaName>(
  transport: Transport,
  service: string,
  jobType: string,
  resultSchema: Schema,
  observer: Observer<JobResultEntry<MessageForSchema<Schema>>>,
): Promise<Subscription> {
  return watchEvent(transport, jobResultEvent(service, jobType), {
    onValue: ({ job, result }, key) => {
      let decoded: MessageForSchema<Schema>
      try {
        decoded = decodeCdr(resultSchema, result)
      } catch (error) {
        observer.onError(error)
        return
      }
      observer.onValue({ job, result: decoded }, key)
    },
    onError: (error) => observer.onError(error),
  })
}
