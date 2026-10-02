import { decodeSample } from './cdr'
import type { Subscription, Transport } from './transport'
import type { MessageForSchema, SchemaName } from './types'
import type { Observer } from './watch'

/** An Event endpoint, such as the generated `recorder.operation`. */
export interface EventEndpoint<Schema extends SchemaName> {
  key: string
  messageSchema: Schema
}

/** Subscribes to an Event and decodes each sample. */
export function watchEvent<Schema extends SchemaName>(
  transport: Transport,
  event: EventEndpoint<Schema>,
  observer: Observer<MessageForSchema<Schema>>,
): Promise<Subscription> {
  return transport.subscribe(event.key, (sample) => {
    try {
      observer.onValue(decodeSample(sample, event.messageSchema), sample.key)
    } catch (error) {
      observer.onError(error)
    }
  })
}
