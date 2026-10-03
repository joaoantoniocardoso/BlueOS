/* eslint-disable no-void */
import type { EndpointInfo, ServiceInfo } from '@blueos-idl/messages'

import type {
  InspectorApiClient, InspectorRequestKind, InspectorRequestResult as InspectorApiRequestResult,
} from './adapters/api-client'
import type { LazySchemaProvider } from './adapters/schema-provider'
import {
  applyBlueosServiceLiveliness,
  applyRos2Liveliness,
  applySample,
  availableViews,
  createCoalescer,
  createInspectorState,
  decodePayload,
  defaultRequestText,
  defaultView,
  defaultViewRegistry,
  endpointsByKind,
  endpointSchemas,
  parseRequestText,
  topicsBySource,
} from './logic'
import type {
  CdrCodec,
  DecodedPayload,
  FrameScheduler,
  InspectorState,
  SampleRecord,
  TopicGroup,
  TopicInfo,
  Unsubscribe,
  ViewDescriptor,
} from './logic/types'

export interface InspectorSourceHandlers {
  onSample: (sample: SampleRecord) => void
  onRos2Liveliness: (tokenKey: string, alive: boolean) => void
  onBlueosServiceLiveliness: (service: string, alive: boolean) => void
  onError: (message: string) => void
}

export interface InspectorSource {
  start(handlers: InspectorSourceHandlers): Unsubscribe
}

export type InspectorRequestResult =
  | InspectorApiRequestResult
  | { kind: 'replies', replies: Array<{ key: string, decoded: DecodedPayload }> }

export type LastRequestResult =
  | { status: 'success', result: InspectorRequestResult }
  | { status: 'error', message: string }

export interface InspectorViewState {
  filter: string
  topicGroups: TopicGroup[]
  selectedKey: string | null
  selectedTopic: TopicInfo | null
  availableViews: ViewDescriptor[]
  selectedViewId: string
  selectedDecoded: DecodedPayload | null
  selectedService: string | null
  serviceInfo: ServiceInfo | null
  serviceEndpoints: Record<string, EndpointInfo[]> | null
  requestText: string
  lastRequestResult: LastRequestResult | null
  catalogLoaded: boolean
  sourceError: string | null
}

export interface InspectorControllerCallbacks {
  onState: (state: InspectorViewState) => void
}

export interface InspectorControllerDependencies {
  source: InspectorSource
  apiClient: InspectorApiClient
  schemaProvider: LazySchemaProvider
  codec: CdrCodec
  scheduler: FrameScheduler
  clock?: () => number
  viewRegistry?: ViewDescriptor[]
}

function topicMatchesFilter(topic: TopicInfo, needle: string): boolean {
  if (topic.key.toLowerCase().includes(needle)) {
    return true
  }
  if (topic.schemaName?.toLowerCase().includes(needle)) {
    return true
  }
  if (topic.ros2?.topic.toLowerCase().includes(needle)) {
    return true
  }
  if (topic.blueos?.service.toLowerCase().includes(needle)) {
    return true
  }
  return false
}

function filterTopicGroups(groups: TopicGroup[], filter: string): TopicGroup[] {
  const needle = filter.trim().toLowerCase()
  if (!needle) {
    return groups
  }
  const filtered: TopicGroup[] = []
  for (const group of groups) {
    if (group.source === 'blueos') {
      const topics = group.topics.filter((topic) => topicMatchesFilter(topic, needle))
      if (topics.length > 0) {
        filtered.push({ ...group, topics })
      }
    } else {
      const topics = group.topics.filter((topic) => topicMatchesFilter(topic, needle))
      if (topics.length > 0) {
        filtered.push({ ...group, topics })
      }
    }
  }
  return filtered
}

export class InspectorController {
  private readonly dependencies: InspectorControllerDependencies

  private readonly callbacks: InspectorControllerCallbacks

  private readonly clock: () => number

  private inspectorState: InspectorState

  private filter = ''

  private selectedKey: string | null = null

  private selectedService: string | null = null

  private serviceInfo: ServiceInfo | null = null

  private requestText = ''

  private lastRequestResult: LastRequestResult | null = null

  private catalogLoaded = false

  private sourceError: string | null = null

  private sourceUnsubscribe: Unsubscribe | null = null

  private emitHandle: unknown | null = null

  private lastDecodedReceivedAt: number | null = null

  private selectedDecoded: DecodedPayload | null = null

  private selectedSampleListeners: Array<(sample: SampleRecord) => void> = []

  private readonly sampleCoalescer

  private readonly viewRegistry: ViewDescriptor[]

  private selectedViewId: string | null = null

  constructor(dependencies: InspectorControllerDependencies, callbacks: InspectorControllerCallbacks) {
    this.dependencies = dependencies
    this.callbacks = callbacks
    this.clock = dependencies.clock ?? Date.now
    this.viewRegistry = dependencies.viewRegistry ?? defaultViewRegistry
    this.inspectorState = createInspectorState()
    this.sampleCoalescer = createCoalescer(this.dependencies.scheduler, (batch) => {
      this.applySampleBatch(batch)
    })
  }

  subscribeSelectedSample(listener: (sample: SampleRecord) => void): Unsubscribe {
    this.selectedSampleListeners.push(listener)
    return () => {
      this.selectedSampleListeners = this.selectedSampleListeners.filter((entry) => entry !== listener)
    }
  }

  start(): void {
    if (this.sourceUnsubscribe !== null) {
      return
    }
    void this.dependencies.schemaProvider.ready().then(() => {
      this.catalogLoaded = true
      this.invalidateSelectedDecode()
      this.scheduleEmit()
    })
    this.sourceUnsubscribe = this.dependencies.source.start({
      onSample: (sample) => {
        this.sampleCoalescer.push(sample.key, sample)
      },
      onRos2Liveliness: (tokenKey, alive) => {
        this.inspectorState = applyRos2Liveliness(this.inspectorState, tokenKey, alive)
        this.scheduleEmit()
      },
      onBlueosServiceLiveliness: (service, alive) => {
        this.inspectorState = applyBlueosServiceLiveliness(this.inspectorState, service, alive)
        this.scheduleEmit()
      },
      onError: (message) => {
        this.sourceError = message
        this.scheduleEmit()
      },
    })
    this.scheduleEmit()
  }

  stop(): void {
    this.sourceUnsubscribe?.()
    this.sourceUnsubscribe = null
    this.sampleCoalescer.dispose()
    if (this.emitHandle !== null) {
      this.dependencies.scheduler.cancel(this.emitHandle)
      this.emitHandle = null
    }
    this.selectedSampleListeners = []
  }

  setFilter(text: string): void {
    this.filter = text
    this.scheduleEmit()
  }

  selectTopic(key: string | null): void {
    this.selectedKey = key
    this.sampleCoalescer.setSelectedKey(key)
    if (key === null) {
      this.selectedViewId = null
      this.invalidateSelectedDecode()
      this.scheduleEmit()
      return
    }
    const topic = this.inspectorState.topics[key]
    if (topic) {
      this.selectedViewId = defaultView(topic, this.viewRegistry).id
    }
    this.invalidateSelectedDecode()
    this.scheduleEmit()
  }

  selectView(id: string): void {
    const selectedTopic = this.selectedKey ? this.inspectorState.topics[this.selectedKey] : undefined
    const previousViewId = this.selectedViewId
      ?? (selectedTopic ? defaultView(selectedTopic, this.viewRegistry).id : 'json')
    this.selectedViewId = id
    if (previousViewId === 'video' && id !== 'video') {
      this.invalidateSelectedDecode()
    }
    this.scheduleEmit()
  }

  selectService(name: string | null): void {
    this.selectedService = name
    this.serviceInfo = null
    this.scheduleEmit()
    if (name === null) {
      return
    }
    void this.dependencies.apiClient.serviceInfo(name).then((info) => {
      if (this.selectedService !== name) {
        return
      }
      this.serviceInfo = info
      this.scheduleEmit()
    }).catch((error) => {
      if (this.selectedService !== name) {
        return
      }
      const message = error instanceof Error ? error.message : String(error)
      this.lastRequestResult = { status: 'error', message }
      this.scheduleEmit()
    })
  }

  setRequestText(text: string): void {
    this.requestText = text
    this.scheduleEmit()
  }

  async sendRequest(endpoint: EndpointInfo): Promise<void> {
    const parsed = parseRequestText(this.requestText)
    if (!parsed.ok) {
      this.lastRequestResult = { status: 'error', message: parsed.message }
      this.scheduleEmit()
      return
    }
    try {
      const { requestSchema, responseSchema } = endpointSchemas(endpoint)
      const result = await this.dependencies.apiClient.request(
        endpoint.key,
        endpoint.kind as InspectorRequestKind,
        requestSchema,
        responseSchema,
        requestSchema ? parsed.value : undefined,
      )
      this.lastRequestResult = { status: 'success', result }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      this.lastRequestResult = { status: 'error', message }
    }
    this.scheduleEmit()
  }

  async rawQuery(key: string, text: string): Promise<void> {
    try {
      const samples = await this.dependencies.apiClient.rawQuery(key, text)
      const replies = samples.map((sample) => {
        const topic: TopicInfo = {
          key: sample.key,
          source: 'raw',
          sampleCount: 1,
          lastSample: sample,
          encoding: sample.encoding.includes(';')
            ? sample.encoding.slice(0, sample.encoding.indexOf(';'))
            : sample.encoding,
        }
        return {
          key: sample.key,
          decoded: decodePayload(
            topic,
            undefined,
            this.dependencies.schemaProvider,
            this.dependencies.codec,
          ),
        }
      })
      this.lastRequestResult = { status: 'success', result: { kind: 'replies', replies } }
      this.scheduleEmit()
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      this.lastRequestResult = { status: 'error', message }
      this.scheduleEmit()
    }
  }

  fillDefaultRequestText(schemaName: string): void {
    const text = defaultRequestText(schemaName, this.dependencies.schemaProvider, this.dependencies.codec)
    if (text !== undefined) {
      this.requestText = text
      this.scheduleEmit()
    }
  }

  private applySampleBatch(batch: Record<string, SampleRecord>): void {
    for (const [key, sample] of Object.entries(batch)) {
      this.inspectorState = applySample(this.inspectorState, sample)
      if (key === this.selectedKey) {
        for (const listener of this.selectedSampleListeners) {
          listener(sample)
        }
        const topic = this.inspectorState.topics[key]
        const viewId = this.selectedViewId ?? (topic ? defaultView(topic, this.viewRegistry).id : 'json')
        if (viewId !== 'video') {
          this.invalidateSelectedDecode()
        }
      }
    }
    this.scheduleEmit()
  }

  private invalidateSelectedDecode(): void {
    this.lastDecodedReceivedAt = null
    this.selectedDecoded = null
  }

  private scheduleEmit(): void {
    if (this.emitHandle !== null) {
      return
    }
    this.emitHandle = this.dependencies.scheduler.schedule(() => {
      this.emitHandle = null
      this.emitState()
    })
  }

  private computeSelectedDecoded(viewId: string): DecodedPayload | null {
    if (viewId === 'video') {
      return null
    }
    if (!this.selectedKey) {
      return null
    }
    const topic = this.inspectorState.topics[this.selectedKey]
    const sample = topic?.lastSample
    if (!topic || !sample) {
      return null
    }
    if (this.lastDecodedReceivedAt === sample.receivedAt && this.selectedDecoded !== null) {
      return this.selectedDecoded
    }
    const decoded = decodePayload(
      topic,
      topic.schemaName,
      this.dependencies.schemaProvider,
      this.dependencies.codec,
    )
    this.selectedDecoded = decoded
    this.lastDecodedReceivedAt = sample.receivedAt
    return decoded
  }

  private emitState(): void {
    const selectedTopic = this.selectedKey ? this.inspectorState.topics[this.selectedKey] ?? null : null
    const viewId = this.selectedViewId
      ?? (selectedTopic ? defaultView(selectedTopic, this.viewRegistry).id : 'json')
    this.callbacks.onState({
      filter: this.filter,
      topicGroups: filterTopicGroups(topicsBySource(this.inspectorState), this.filter),
      selectedKey: this.selectedKey,
      selectedTopic,
      availableViews: selectedTopic ? availableViews(selectedTopic, this.viewRegistry) : [],
      selectedViewId: viewId,
      selectedDecoded: this.computeSelectedDecoded(viewId),
      selectedService: this.selectedService,
      serviceInfo: this.serviceInfo,
      serviceEndpoints: this.serviceInfo ? endpointsByKind(this.serviceInfo.endpoints) : null,
      requestText: this.requestText,
      lastRequestResult: this.lastRequestResult,
      catalogLoaded: this.catalogLoaded,
      sourceError: this.sourceError,
    })
  }
}
