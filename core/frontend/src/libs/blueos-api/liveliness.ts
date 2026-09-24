import { Sample, SampleKind, Subscriber } from '@eclipse-zenoh/zenoh-ts'

import { serviceLivelinessKey } from './keys'
import { getZenohSession, type Unsubscribe } from './zenoh-helpers'

/* eslint-disable import/prefer-default-export, no-void */
export function watchServiceAlive(
  service: string,
  onAlive: (alive: boolean) => void,
): Unsubscribe {
  const key = serviceLivelinessKey(service)
  let cancelled = false
  let subscriber: Subscriber | undefined

  async function start(): Promise<void> {
    const session = await getZenohSession()
    subscriber = await session.liveliness().declareSubscriber(key, {
      history: true,
      handler: async (sample: Sample) => {
        if (!cancelled) {
          onAlive(sample.kind() === SampleKind.PUT)
        }
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
