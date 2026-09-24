import type { Log } from '@blueos-idl/messages'
import { Sample, SampleKind, Subscriber } from '@eclipse-zenoh/zenoh-ts'

import { logKey } from './keys'
import { LOG_SCHEMA } from './types'
import { decodeSample, getZenohSession, type Unsubscribe } from './zenoh-helpers'

/* eslint-disable import/prefer-default-export, no-void */
export function watchLogs(
  service: string,
  onLog: (entry: Log) => void,
): Unsubscribe {
  const key = logKey(service)
  let cancelled = false
  let subscriber: Subscriber | undefined

  async function start(): Promise<void> {
    const session = await getZenohSession()
    subscriber = await session.declareSubscriber(key, {
      handler: async (sample: Sample) => {
        if (cancelled || sample.kind() === SampleKind.DELETE) {
          return Promise.resolve()
        }
        onLog(decodeSample(sample, LOG_SCHEMA))
        return Promise.resolve()
      },
    })
  }

  start().catch(() => undefined)

  return () => {
    cancelled = true
    subscriber?.undeclare().catch(() => undefined)
  }
}
