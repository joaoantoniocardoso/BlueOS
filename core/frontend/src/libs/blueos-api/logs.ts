import type { Log } from '@blueos-idl/messages'

import { decodeSample } from './cdr'
import { logKey } from './keys'
import type { Subscription, Transport } from './transport'
import { LOG_SCHEMA } from './types'

export interface LogObserver {
  onLog(entry: Log): void
  onError?(error: unknown): void
}

/** Subscribes to a service `log` stream and decodes each sample as Foxglove `Log` (D-13). */
export async function watchLogs(
  transport: Transport,
  service: string,
  observer: LogObserver,
): Promise<Subscription> {
  const key = logKey(service)
  return transport.subscribe(key, (sample) => {
    try {
      observer.onLog(decodeSample(sample, LOG_SCHEMA))
    } catch (error) {
      observer.onError?.(error)
    }
  })
}
