import { Sample, SampleKind, Subscriber } from '@eclipse-zenoh/zenoh-ts'

import { eventKey } from '@/libs/blueos-api'
import { decodeSample, getZenohSession, type Unsubscribe } from '@/libs/blueos-api/zenoh-helpers'

import { RECORDER_SERVICE, RECORDING_OPERATION_SCHEMA } from './constants'
import { mapRecordingOperation } from './map'
import type { RecordingOperationEvent } from './types'

/* eslint-disable import/prefer-default-export, no-void */
export function watchRecordingOperations(
  onOperation: (event: RecordingOperationEvent) => void,
): Unsubscribe {
  const key = eventKey(RECORDER_SERVICE, 'operation')
  let cancelled = false
  let subscriber: Subscriber | undefined

  async function start(): Promise<void> {
    const session = await getZenohSession()
    subscriber = await session.declareSubscriber(key, {
      handler: async (sample: Sample) => {
        if (cancelled || sample.kind() === SampleKind.DELETE) {
          return Promise.resolve()
        }
        onOperation(mapRecordingOperation(decodeSample(sample, RECORDING_OPERATION_SCHEMA)))
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
