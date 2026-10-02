/* eslint-disable import/no-extraneous-dependencies, import/prefer-default-export */
import {
  Sample, SampleKind, Subscriber,
} from '@eclipse-zenoh/zenoh-ts'

import { KEY_PREFIX } from '@/libs/blueos-api/keys'
import type { Transport } from '@/libs/blueos-api/transport'
import zenoh from '@/libs/zenoh'

import type { InspectorSource, InspectorSourceHandlers } from '../inspector-controller'
import type { SampleRecord, Unsubscribe } from '../logic/types'
import { fetchInitialStateSamples } from './initial-state-queries'

const BLUEOS_SERVICE_LIVELINESS = `${KEY_PREFIX}/services/*`
const RMW_ZENOH_LIVELINESS = '@ros2_lv/**'
const ROS2DDS_LIVELINESS = '@/*/@ros2_lv/**'
const LIVELINESS_PATTERNS = [BLUEOS_SERVICE_LIVELINESS, RMW_ZENOH_LIVELINESS, ROS2DDS_LIVELINESS]

function serviceNameFromLivelinessKey(key: string): string | undefined {
  const prefix = `${KEY_PREFIX}/services/`
  if (!key.startsWith(prefix)) {
    return undefined
  }
  const service = key.slice(prefix.length)
  if (!service || service.includes('/')) {
    return undefined
  }
  return service
}

function zenohSampleToRecord(sample: Sample, clock: () => number): SampleRecord {
  return {
    key: sample.keyexpr().toString(),
    payload: sample.payload().toBytes(),
    encoding: sample.encoding().toString(),
    receivedAt: clock(),
    kind: sample.kind() === SampleKind.PUT ? 'put' : 'delete',
  }
}

function dispatchLivelinessToken(handlers: InspectorSourceHandlers, tokenKey: string, alive: boolean): void {
  const service = serviceNameFromLivelinessKey(tokenKey)
  if (service !== undefined) {
    handlers.onBlueosServiceLiveliness(service, alive)
    return
  }
  handlers.onRos2Liveliness(tokenKey, alive)
}

async function undeclareSubscriber(subscriber: Subscriber | undefined): Promise<void> {
  if (subscriber !== undefined) {
    await subscriber.undeclare().catch(() => undefined)
  }
}

export function createZenohSource(
  transportProvider: () => Promise<Transport>,
  clock: () => number = Date.now,
): InspectorSource {
  return {
    start(handlers: InspectorSourceHandlers): Unsubscribe {
      let cancelled = false
      let dataSubscriber: Subscriber | undefined
      const livelinessSubscribers: Subscriber[] = []

      function isCancelled(): boolean {
        return cancelled
      }

      async function setup(): Promise<void> {
        const session = await zenoh.getSession()
        if (isCancelled()) {
          return
        }

        const transport = await transportProvider()
        if (isCancelled()) {
          return
        }

        dataSubscriber = await session.declareSubscriber('**', {
          handler: async (sample: Sample) => {
            if (!isCancelled()) {
              handlers.onSample(zenohSampleToRecord(sample, clock))
            }
            return Promise.resolve()
          },
        })
        if (isCancelled()) {
          await undeclareSubscriber(dataSubscriber)
          dataSubscriber = undefined
          return
        }

        await fetchInitialStateSamples(transport, handlers, clock)
        if (isCancelled()) {
          await undeclareSubscriber(dataSubscriber)
          dataSubscriber = undefined
          return
        }

        for (const pattern of LIVELINESS_PATTERNS) {
          if (isCancelled()) {
            return
          }
          // eslint-disable-next-line no-await-in-loop
          const subscriber = await session.liveliness().declareSubscriber(pattern, {
            history: true,
            handler: async (sample: Sample) => {
              if (isCancelled()) {
                return Promise.resolve()
              }
              dispatchLivelinessToken(
                handlers,
                sample.keyexpr().toString(),
                sample.kind() === SampleKind.PUT,
              )
              return Promise.resolve()
            },
          })
          if (isCancelled()) {
            await undeclareSubscriber(subscriber)
            return
          }
          livelinessSubscribers.push(subscriber)
        }
      }

      setup().catch((error) => {
        const message = error instanceof Error ? error.message : String(error)
        handlers.onError(message)
      })

      return () => {
        cancelled = true
        undeclareSubscriber(dataSubscriber).catch(() => undefined)
        for (const subscriber of livelinessSubscribers) {
          subscriber.undeclare().catch(() => undefined)
        }
        dataSubscriber = undefined
        livelinessSubscribers.length = 0
      }
    },
  }
}
