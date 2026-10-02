import type { Log } from '@blueos-idl/messages'

import { decodeSample } from './cdr'
import { extensionLogKey, httpGatewayPrefix, logKey } from './keys'
import type { Subscription, Transport } from './transport'
import { LOG_SCHEMA } from './types'

export interface LogObserver {
  onLog(entry: Log): void
  onError?(error: unknown): void
}

function subscribeLogs(transport: Transport, key: string, observer: LogObserver): Promise<Subscription> {
  return transport.subscribe(key, (sample) => {
    try {
      observer.onLog(decodeSample(sample, LOG_SCHEMA))
    } catch (error) {
      observer.onError?.(error)
    }
  })
}

/** Subscribes to a service `log` stream and decodes each sample as Foxglove `Log` (D-13). */
export async function watchLogs(
  transport: Transport,
  service: string,
  observer: LogObserver,
): Promise<Subscription> {
  return subscribeLogs(transport, logKey(service), observer)
}

/** Subscribes to Kraken extension log stream on `blueos/v1/<service>/log/extension/<id>`. */
export async function watchExtensionLogs(
  transport: Transport,
  service: string,
  extensionIdentifier: string,
  observer: LogObserver,
): Promise<Subscription> {
  return subscribeLogs(transport, extensionLogKey(service, extensionIdentifier), observer)
}

/** One historical log line from Kraken's `extension/logs/request` queryable. */
export interface ExtensionHistoricalLogLine {
  level?: number
  message: string
}

/** JSON body Kraken's `extension/logs/request` queryable returns. */
export interface ExtensionLogsRequestResult {
  status?: string
  messages?: ExtensionHistoricalLogLine[]
  topic?: string
  total_lines?: number
  error?: string
  error_type?: string
}

export function extensionLogsRequestKey(service: string, extensionIdentifier: string): string {
  const query = `extension_name=${encodeURIComponent(extensionIdentifier)}`
  return `${httpGatewayPrefix(service)}/extension/logs/request?${query}`
}

/** Fetches buffered extension logs through the HTTP gateway queryable. */
export async function requestExtensionLogs(
  transport: Transport,
  service: string,
  extensionIdentifier: string,
): Promise<ExtensionLogsRequestResult | null> {
  const replies = await transport.get(extensionLogsRequestKey(service, extensionIdentifier))
  const sample = replies.find((reply) => reply.kind === 'sample')?.sample
  if (!sample) {
    return null
  }
  try {
    const text = new TextDecoder().decode(sample.payload)
    return JSON.parse(text) as ExtensionLogsRequestResult
  } catch {
    return null
  }
}
