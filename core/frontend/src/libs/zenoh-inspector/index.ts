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

export function createInspectorController(callbacks: InspectorControllerCallbacks): InspectorController {
  const schemaProvider = createSchemaProvider()
  return new InspectorController({
    source: createZenohSource(),
    apiClient: createInspectorApiClient(schemaProvider),
    schemaProvider,
    codec: cdrCodec,
    scheduler: browserFrameScheduler,
  }, callbacks)
}
