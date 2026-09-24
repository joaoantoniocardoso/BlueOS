/* eslint-disable import/prefer-default-export, no-void */
import type { JobList } from '@blueos-idl/messages'
import {
  LinkEvent,
  Sample,
  SampleKind,
  Subscriber,
} from '@eclipse-zenoh/zenoh-ts'

import { jobsKey } from './keys'
import { JOB_LIST_SCHEMA } from './types'
import {
  decodeSample,
  getZenohSession,
  type Unsubscribe,
  zenohQueryCdr,
} from './zenoh-helpers'

export function watchJobs(
  service: string,
  onJobs: (jobs: JobList) => void,
): Unsubscribe {
  const key = jobsKey(service)
  let cancelled = false
  let subscriber: Subscriber | undefined
  let linkListener: { undeclare: () => Promise<void> } | undefined

  async function queryCurrent(): Promise<void> {
    if (cancelled) {
      return
    }
    try {
      const value = await zenohQueryCdr(key, JOB_LIST_SCHEMA)
      if (!cancelled) {
        onJobs(value)
      }
    } catch {
      // Jobs snapshot may be unavailable until the service starts.
    }
  }

  async function start(): Promise<void> {
    await queryCurrent()
    if (cancelled) {
      return
    }

    const session = await getZenohSession()
    subscriber = await session.declareSubscriber(key, {
      handler: async (sample: Sample) => {
        if (cancelled || sample.kind() === SampleKind.DELETE) {
          return Promise.resolve()
        }
        onJobs(decodeSample(sample, JOB_LIST_SCHEMA))
        return Promise.resolve()
      },
    })

    linkListener = await session.linkEventsListener({
      handler: async (event: LinkEvent) => {
        if (cancelled || event.kind() !== SampleKind.PUT) {
          return Promise.resolve()
        }
        await queryCurrent()
        return Promise.resolve()
      },
    })
  }

  start().catch(() => undefined)

  return () => {
    cancelled = true
    subscriber?.undeclare().catch(() => undefined)
    linkListener?.undeclare().catch(() => undefined)
  }
}
