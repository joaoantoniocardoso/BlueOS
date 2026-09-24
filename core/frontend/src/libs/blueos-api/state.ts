/* eslint-disable import/prefer-default-export, no-void */
import {
  LinkEvent,
  Sample,
  SampleKind,
  Subscriber,
} from '@eclipse-zenoh/zenoh-ts'

import { stateKey } from './keys'
import type { MessageForSchema, SchemaName } from './types'
import {
  decodeSample,
  getZenohSession,
  type Unsubscribe,
  zenohQueryCdr,
} from './zenoh-helpers'

export function watchState<Schema extends SchemaName>(
  service: string,
  name: string,
  schemaName: Schema,
  onValue: (value: MessageForSchema<Schema>) => void,
): Unsubscribe {
  const key = stateKey(service, name)
  let cancelled = false
  let subscriber: Subscriber | undefined
  let linkListener: { undeclare: () => Promise<void> } | undefined

  async function queryCurrent(): Promise<void> {
    if (cancelled) {
      return
    }
    try {
      const value = await zenohQueryCdr(key, schemaName)
      if (!cancelled) {
        onValue(value)
      }
    } catch {
      // State may be unavailable until the service starts.
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
        onValue(decodeSample(sample, schemaName))
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
