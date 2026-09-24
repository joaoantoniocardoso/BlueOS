/* eslint-disable import/prefer-default-export, no-void */
import {
  LinkEvent,
  Sample,
  SampleKind,
  Subscriber,
} from '@eclipse-zenoh/zenoh-ts'

import { settingsKey } from './keys'
import type { MessageForSchema, SchemaName } from './types'
import {
  decodeSample,
  getZenohSession,
  type Unsubscribe,
  zenohQueryCdr,
} from './zenoh-helpers'

export function watchSettings<Schema extends SchemaName = 'blueos_msgs/msg/SettingsEnvelope'>(
  service: string,
  onSettings: (envelope: MessageForSchema<Schema>) => void,
  schemaName: Schema = 'blueos_msgs/msg/SettingsEnvelope' as Schema,
): Unsubscribe {
  const key = settingsKey(service)
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
        onSettings(value)
      }
    } catch {
      // Settings may be unavailable until the service starts.
    }
  }

  void (async () => {
    await queryCurrent()
    if (cancelled) {
      return
    }
    const session = await getZenohSession()
    subscriber = await session.declareSubscriber(key, {
      handler: (sample: Sample) => {
        if (sample.kind !== SampleKind.PUT) {
          return
        }
        try {
          const value = decodeSample(schemaName, sample)
          onSettings(value)
        } catch {
          // ignore decode errors on stream
        }
      },
    })
    linkListener = session.declareLinkListener((event) => {
      if (event === LinkEvent.UP) {
        void queryCurrent()
      }
    })
  })()

  return () => {
    cancelled = true
    void subscriber?.undeclare()
    void linkListener?.undeclare()
  }
}
