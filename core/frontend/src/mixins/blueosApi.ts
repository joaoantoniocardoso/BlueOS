import type { StateEndpoint } from '@/libs/blueos-api/endpoints'
import type { JobObserver } from '@/libs/blueos-api/job'
import { watchJob } from '@/libs/blueos-api/job'
import type { Subscription, Transport } from '@/libs/blueos-api/transport'
import type { MessageForSchema, SchemaName } from '@/libs/blueos-api/types'
import { watchState } from '@/libs/blueos-api/watch'

type BlueosApiMixinData = {
  blueosApiSubscriptions: Subscription[]
}

/**
 * Vue 2 mixin: ties blueos-api subscriptions to component lifecycle (D-14). Replace with a Vue 3 composable that
 * calls the same library functions without changing the core.
 */
// eslint-disable-next-line import/prefer-default-export
export const blueosApiMixin = {
  data(): BlueosApiMixinData {
    return {
      blueosApiSubscriptions: [],
    }
  },

  methods: {
    async blueosTrackSubscription(subscriptionPromise: Promise<Subscription>): Promise<Subscription> {
      const subscription = await subscriptionPromise
      // Vue 2 only exposes destruction on the instance as `_isDestroyed`.
      // eslint-disable-next-line no-underscore-dangle
      if (this._isDestroyed) {
        await subscription.close()
        return subscription
      }
      this.blueosApiSubscriptions.push(subscription)
      return subscription
    },

    async blueosWatchState<Schema extends SchemaName>(
      transport: Transport,
      state: StateEndpoint<Schema>,
      onValue: (message: MessageForSchema<Schema>, key: string) => void,
      onError?: (error: unknown) => void,
    ): Promise<Subscription> {
      return this.blueosTrackSubscription(watchState(transport, state, {
        onValue,
        onError: onError ?? (() => undefined),
      }))
    },

    async blueosWatchJob(
      transport: Transport,
      service: string,
      jobId: string,
      observer: JobObserver,
    ): Promise<Subscription> {
      return this.blueosTrackSubscription(watchJob(transport, service, jobId, observer))
    },
  },

  beforeDestroy(): void {
    for (const subscription of this.blueosApiSubscriptions) {
      subscription.close().catch(() => undefined)
    }
    this.blueosApiSubscriptions = []
  },
}
