import type { CommandAck, JobList, Log } from '@blueos-idl/messages'

/* eslint-disable import/prefer-default-export, object-curly-newline */
import {
  getServiceInfo,
  query,
  sendCommand,
  watchJobs,
  watchLogs,
  watchServiceAlive,
  watchState,
} from './index'
import type { MessageForSchema, SchemaName } from './types'
import type { Unsubscribe } from './zenoh-helpers'

type BlueosApiMixinData = {
  blueosApiUnsubscribers: Unsubscribe[]
}

/**
 * Vue 2 mixin: ties blueos-api subscriptions to component lifecycle (D-14).
 * Replace with a Vue 3 composable that calls the same functions from `./index` without changing the core.
 */
export const blueosApiMixin = {
  data(): BlueosApiMixinData {
    return {
      blueosApiUnsubscribers: [],
    }
  },

  methods: {
    blueosTrackUnsubscribe(unsubscribe: Unsubscribe): Unsubscribe {
      this.blueosApiUnsubscribers.push(unsubscribe)
      return unsubscribe
    },

    blueosWatchState<Schema extends SchemaName>(
      service: string,
      name: string,
      schemaName: Schema,
      onValue: (value: MessageForSchema<Schema>) => void,
    ): Unsubscribe {
      return this.blueosTrackUnsubscribe(watchState(service, name, schemaName, onValue))
    },

    blueosWatchJobs(service: string, onJobs: (jobs: JobList) => void): Unsubscribe {
      return this.blueosTrackUnsubscribe(watchJobs(service, onJobs))
    },

    blueosWatchServiceAlive(service: string, onAlive: (alive: boolean) => void): Unsubscribe {
      return this.blueosTrackUnsubscribe(watchServiceAlive(service, onAlive))
    },

    blueosWatchLogs(service: string, onLog: (entry: Log) => void): Unsubscribe {
      return this.blueosTrackUnsubscribe(watchLogs(service, onLog))
    },

    blueosSendCommand<RequestSchema extends SchemaName>(
      service: string,
      name: string,
      requestSchemaName: RequestSchema,
      request: MessageForSchema<RequestSchema>,
      timeoutMs?: number,
    ): Promise<CommandAck> {
      return sendCommand(service, name, requestSchemaName, request, timeoutMs)
    },

    blueosQuery<
      RequestSchema extends SchemaName | undefined,
      ResponseSchema extends SchemaName,
    >(
      service: string,
      name: string,
      responseSchemaName: ResponseSchema,
      requestSchemaName?: RequestSchema,
      request?: RequestSchema extends SchemaName ? MessageForSchema<RequestSchema> : never,
      timeoutMs?: number,
    ): Promise<MessageForSchema<ResponseSchema>> {
      return query(service, name, responseSchemaName, requestSchemaName, request, timeoutMs)
    },

    blueosGetServiceInfo(service: string, timeoutMs?: number) {
      return getServiceInfo(service, timeoutMs)
    },
  },

  beforeDestroy(): void {
    for (const unsubscribe of this.blueosApiUnsubscribers) {
      unsubscribe()
    }
    this.blueosApiUnsubscribers = []
  },
}
