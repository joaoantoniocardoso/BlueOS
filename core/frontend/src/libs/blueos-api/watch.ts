import { decodeSample } from './cdr'
import type { StateEndpoint } from './endpoints'
import { QueryFailedError } from './errors'
import type {
  Reply, Sample, Subscription, Transport,
} from './transport'
import type { MessageForSchema, SchemaName } from './types'

/** Receives every value of a State with the key it came on, and every sample or reply that could not be used. */
export interface Observer<Message> {
  onValue: (message: Message, key: string) => void
  onError: (error: unknown) => void
}

/**
 * Watches a State: subscribes first, then queries its current value, so an update published between the two is never
 * lost (D-10). The reply for a key that already had an update is ignored, because the update is newer. `state.key`
 * may hold wildcards: each matching key is followed on its own. Settings and Jobs are States too: watch
 * `settingsState(service)` and `jobsState(service)`.
 */
export async function watchState<Schema extends SchemaName>(
  transport: Transport,
  state: StateEndpoint<Schema>,
  observer: Observer<MessageForSchema<Schema>>,
): Promise<Subscription> {
  const updated = new Set<string>()
  function deliver(sample: Sample): void {
    updated.add(sample.key)
    let message: MessageForSchema<Schema>
    try {
      message = decodeSample(sample, state.messageSchema)
    } catch (error) {
      observer.onError(error)
      return
    }
    observer.onValue(message, sample.key)
  }

  const subscription = await transport.subscribe(state.key, deliver)
  let replies: Reply[]
  try {
    replies = await transport.get(state.key)
  } catch (error) {
    await subscription.close()
    throw error
  }
  for (const reply of replies) {
    if (reply.kind === 'error') {
      observer.onError(new QueryFailedError(state.key, new TextDecoder().decode(reply.payload)))
    } else if (!updated.has(reply.sample.key)) {
      deliver(reply.sample)
    }
  }
  return subscription
}
