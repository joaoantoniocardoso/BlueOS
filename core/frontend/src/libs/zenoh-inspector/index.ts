import type { Transport } from '@/libs/blueos-api/transport'
import zenohTransport from '@/libs/blueos-api/zenoh-transport'
import zenoh from '@/libs/zenoh'

import { createInspectorApiClient } from './adapters/api-client'
import { cdrCodec } from './adapters/cdr-codec'
import { browserFrameScheduler } from './adapters/frame-scheduler'
import { createSchemaProvider } from './adapters/schema-provider'
import { createZenohSource } from './adapters/zenoh-source'
import { InspectorController, type InspectorControllerCallbacks } from './inspector-controller'

export {
  InspectorController,
  type InspectorControllerCallbacks,
  type InspectorRequestResult,
  type InspectorViewState,
  type LastRequestResult,
} from './inspector-controller'

let sharedTransport: Promise<Transport> | null = null

function transportProvider(): Promise<Transport> {
  if (sharedTransport === null) {
    sharedTransport = zenoh.getSession().then((session) => zenohTransport(session))
  }
  return sharedTransport
}

export function createInspectorController(callbacks: InspectorControllerCallbacks): InspectorController {
  const schemaProvider = createSchemaProvider()
  return new InspectorController({
    source: createZenohSource(transportProvider),
    apiClient: createInspectorApiClient(transportProvider, schemaProvider),
    schemaProvider,
    codec: cdrCodec,
    scheduler: browserFrameScheduler,
  }, callbacks)
}
