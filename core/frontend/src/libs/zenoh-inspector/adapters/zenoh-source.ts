/* eslint-disable import/no-extraneous-dependencies, import/prefer-default-export */
import {
  Reply, ReplyError, Sample, SampleKind, Subscriber,
} from '@eclipse-zenoh/zenoh-ts'

import { KEY_PREFIX } from '@/libs/blueos-api/keys'
import {
  getZenohSession,
  samplePayloadBytes,
  type Unsubscribe,
} from '@/libs/blueos-api/zenoh-helpers'

import type { InspectorSource, InspectorSourceHandlers } from '../inspector-controller'
import type { SampleRecord } from '../logic/types'

const BLUEOS_SERVICE_LIVELINESS = `${KEY_PREFIX}/services/*`
const RMW_ZENOH_LIVELINESS = '@ros2_lv/**'
const ROS2DDS_LIVELINESS = '@/*/@ros2_lv/**'
const LIVELINESS_PATTERNS = [BLUEOS_SERVICE_LIVELINESS, RMW_ZENOH_LIVELINESS, ROS2DDS_LIVELINESS]
const STATE_SELECTORS = [`${KEY_PREFIX}/*/state/*`, `${KEY_PREFIX}/*/settings`, `${KEY_PREFIX}/*/jobs`]

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

function sampleToRecord(sample: Sample, clock: () => number): SampleRecord {
  return {
    key: sample.keyexpr().toString(),
    payload: samplePayloadBytes(sample),
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

export function createZenohSource(clock: () => number = Date.now): InspectorSource {
  return {
    start(handlers: InspectorSourceHandlers): Unsubscribe {
      let cancelled = false
      let dataSubscriber: Subscriber | undefined
      const livelinessSubscribers: Subscriber[] = []

      function isCancelled(): boolean {
        return cancelled
      }

      async function setup(): Promise<void> {
        const session = await getZenohSession()
        if (isCancelled()) {
          return
        }

        dataSubscriber = await session.declareSubscriber('**', {
          handler: async (sample: Sample) => {
            if (!isCancelled()) {
              handlers.onSample(sampleToRecord(sample, clock))
            }
            return Promise.resolve()
          },
        })
        if (isCancelled()) {
          await undeclareSubscriber(dataSubscriber)
          dataSubscriber = undefined
          return
        }

        // States publish only on change; never widen these selectors, a get on a command key runs the command.
        for (const selector of STATE_SELECTORS) {
          // eslint-disable-next-line no-await-in-loop
          await session.get(selector, {
            handler: async (reply: Reply) => {
              const result = reply.result()
              if (!isCancelled() && !(result instanceof ReplyError)) {
                handlers.onSample(sampleToRecord(result, clock))
              }
              return Promise.resolve()
            },
          })
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
