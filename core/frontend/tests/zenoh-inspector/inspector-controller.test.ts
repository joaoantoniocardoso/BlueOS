/* eslint-disable import/no-extraneous-dependencies, max-classes-per-file, no-void */
import { CATALOG_SCHEMAS } from '@blueos-idl/catalog'
import type { EndpointInfo, ServiceInfo } from '@blueos-idl/messages'
import {
  describe, expect, it, vi,
} from 'vitest'

import { encodeCdrWithSchema } from '@/libs/blueos-api/cdr'
import { cdrCodec } from '@/libs/zenoh-inspector/adapters/cdr-codec'
import {
  InspectorController,
  type InspectorRequestResult,
  type InspectorSource,
  type InspectorSourceHandlers,
  type InspectorViewState,
} from '@/libs/zenoh-inspector/inspector-controller'
import type { FrameScheduler, SampleRecord, SchemaProvider } from '@/libs/zenoh-inspector/logic/types'

const STD_STRING_SCHEMA = 'std_msgs/msg/String'
const STD_STRING_TEXT = CATALOG_SCHEMAS[STD_STRING_SCHEMA]

const ROS2DDS_CHATTER_TOKEN = [
  '@/aac3178e146ba6f1fc6e6a4085e77f21/@ros2_lv/MP/chatter/',
  'std_msgs\u00a7msg\u00a7String',
].join('')

function stringCdrPayload(data: string): Uint8Array {
  return encodeCdrWithSchema(STD_STRING_SCHEMA, STD_STRING_TEXT, { data })
}

class FakeSource implements InspectorSource {
  handlers: InspectorSourceHandlers | null = null

  unsubscribed = false

  start(handlers: InspectorSourceHandlers): () => void {
    this.handlers = handlers
    return () => {
      this.unsubscribed = true
    }
  }
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, milliseconds)
  })
}

class StoppingDuringSetupSource implements InspectorSource {
  readonly dataUndeclare = vi.fn(async () => undefined)

  readonly livelinessUndeclare = vi.fn(async () => undefined)

  start(handlers: InspectorSourceHandlers): () => void {
    let cancelled = false
    const { dataUndeclare, livelinessUndeclare } = this
    async function setup(): Promise<void> {
      await delay(40)
      if (cancelled) {
        await dataUndeclare()
        return
      }
      await delay(40)
      if (cancelled) {
        await livelinessUndeclare()
      }
    }
    setup().catch((error) => {
      const message = error instanceof Error ? error.message : String(error)
      handlers.onError(message)
    })
    return () => {
      cancelled = true
    }
  }
}

function createFakeScheduler(): { scheduler: FrameScheduler, flush: () => void } {
  const callbacks: Array<() => void> = []
  const scheduler: FrameScheduler = {
    schedule(callback: () => void): number {
      callbacks.push(callback)
      return callbacks.length
    },
    cancel(): void {
      /* no-op */
    },
  }
  return {
    scheduler,
    flush: () => {
      while (callbacks.length > 0) {
        callbacks.shift()?.()
      }
    },
  }
}

function deferredCatalogProvider(): SchemaProvider & { resolveCatalog: () => void, ready: () => Promise<void> } {
  let catalogReady = false
  let resolveReady: (() => void) | null = null
  const readyPromise = new Promise<void>((resolve) => {
    resolveReady = resolve
  })
  return {
    schemaText(schemaName: string): string | undefined {
      if (!catalogReady) {
        return undefined
      }
      return CATALOG_SCHEMAS[schemaName as keyof typeof CATALOG_SCHEMAS]
    },
    ready(): Promise<void> {
      return readyPromise
    },
    resolveCatalog(): void {
      catalogReady = true
      resolveReady?.()
    },
  }
}

function latestState(states: InspectorViewState[]): InspectorViewState {
  return states[states.length - 1]
}

describe('InspectorController', () => {
  it('updates topic groups from samples', () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    source.handlers?.onSample({
      key: 'chatter',
      payload: stringCdrPayload('hello'),
      encoding: 'zenoh/bytes',
      receivedAt: 1,
      kind: 'put',
    })
    flush()
    expect(latestState(states).topicGroups.flatMap((group) => group.topics)).toHaveLength(1)
    source.handlers?.onRos2Liveliness(ROS2DDS_CHATTER_TOKEN, true)
    flush()
    const topic = latestState(states).topicGroups
      .flatMap((group) => group.topics)
      .find((entry) => entry.key === 'chatter')
    expect(topic?.schemaName).toBe(STD_STRING_SCHEMA)
    controller.stop()
  })

  it('coalesces view state to one emission per frame', () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    flush()
    const before = states.length
    for (let index = 0; index < 50; index += 1) {
      source.handlers?.onSample({
        key: `topic/${index}`,
        payload: new Uint8Array([index]),
        encoding: 'zenoh/bytes',
        receivedAt: index,
        kind: 'put',
      })
    }
    expect(states.length).toBe(before)
    flush()
    expect(states.length).toBe(before + 1)
    controller.stop()
  })

  it('decodes only the selected topic', () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const decode = vi.fn(cdrCodec.decode)
    const codec = { ...cdrCodec, decode }

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    source.handlers?.onRos2Liveliness(ROS2DDS_CHATTER_TOKEN, true)
    source.handlers?.onSample({
      key: 'chatter',
      payload: stringCdrPayload('one'),
      encoding: 'zenoh/bytes',
      receivedAt: 1,
      kind: 'put',
    })
    source.handlers?.onSample({
      key: 'other',
      payload: stringCdrPayload('two'),
      encoding: 'zenoh/bytes',
      receivedAt: 2,
      kind: 'put',
    })
    flush()
    decode.mockClear()
    controller.selectTopic('chatter')
    flush()
    expect(decode).toHaveBeenCalledTimes(1)
    expect(latestState(states).selectedDecoded).toEqual({
      kind: 'cdr',
      schemaName: STD_STRING_SCHEMA,
      value: { data: 'one' },
    })
    controller.stop()
  })

  it('decodes ros2dds zenoh/bytes after token liveliness', () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    source.handlers?.onRos2Liveliness(ROS2DDS_CHATTER_TOKEN, true)
    source.handlers?.onSample({
      key: 'chatter',
      payload: stringCdrPayload('cdr'),
      encoding: 'zenoh/bytes',
      receivedAt: 1,
      kind: 'put',
    })
    flush()
    controller.selectTopic('chatter')
    flush()
    expect(latestState(states).selectedDecoded).toEqual({
      kind: 'cdr',
      schemaName: STD_STRING_SCHEMA,
      value: { data: 'cdr' },
    })
    controller.stop()
  })

  it('loads service endpoints when a BlueOS service is selected', async () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const serviceInfo: ServiceInfo = {
      name: 'recorder',
      version: '1',
      build: 'test',
      capabilities: [],
      endpoints: [{
        kind: 'command',
        name: 'start',
        key: 'blueos/v1/recorder/command/start',
        request_schema: 'blueos_msgs/srv/Start',
        response_schema: 'blueos_msgs/msg/CommandAck',
      }],
    }

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: {
        serviceInfo: vi.fn().mockResolvedValue(serviceInfo),
        request: vi.fn(),
        rawQuery: vi.fn(),
      },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    controller.selectService('recorder')
    await Promise.resolve()
    flush()
    expect(latestState(states).serviceEndpoints?.command).toHaveLength(1)
    controller.stop()
  })

  it('sendRequest encodes with the request schema and reports the reply', async () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const endpoint: EndpointInfo = {
      kind: 'query',
      name: 'info',
      key: 'blueos/v1/recorder/query/info',
      request_schema: '',
      response_schema: STD_STRING_SCHEMA,
    }

    const request = vi.fn().mockResolvedValue({
      kind: 'cdr',
      schemaName: STD_STRING_SCHEMA,
      value: { data: 'reply' },
    } satisfies InspectorRequestResult)

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: {
        serviceInfo: vi.fn(),
        request,
        rawQuery: vi.fn(),
      },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    controller.setRequestText('{}')
    await controller.sendRequest(endpoint)
    flush()
    expect(request).toHaveBeenCalledWith(
      endpoint.key,
      'query',
      '',
      STD_STRING_SCHEMA,
      undefined,
    )
    expect(latestState(states).lastRequestResult).toEqual({
      status: 'success',
      result: { kind: 'cdr', schemaName: STD_STRING_SCHEMA, value: { data: 'reply' } },
    })
    controller.stop()
  })

  it('re-decodes when the catalog arrives late', () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    const ready = provider.ready()

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    source.handlers?.onRos2Liveliness(ROS2DDS_CHATTER_TOKEN, true)
    source.handlers?.onSample({
      key: 'chatter',
      payload: stringCdrPayload('late'),
      encoding: 'zenoh/bytes',
      receivedAt: 1,
      kind: 'put',
    })
    flush()
    controller.selectTopic('chatter')
    flush()
    expect(latestState(states).selectedDecoded?.kind).toBe('binary')

    provider.resolveCatalog()
    return ready.then(() => {
      flush()
      expect(latestState(states).catalogLoaded).toBe(true)
      expect(latestState(states).selectedDecoded).toEqual({
        kind: 'cdr',
        schemaName: STD_STRING_SCHEMA,
        value: { data: 'late' },
      })
      controller.stop()
    })
  })

  it('stop cancels the pending frame and unsubscribes the source', () => {
    const source = new FakeSource()
    const callbacks: Array<() => void> = []
    let cancelled = false
    const scheduler: FrameScheduler = {
      schedule(callback: () => void): number {
        callbacks.push(callback)
        return callbacks.length
      },
      cancel(handle: unknown): void {
        cancelled = handle === 1
      },
    }

    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: () => undefined })

    controller.start()
    source.handlers?.onSample({
      key: 'topic/a',
      payload: new Uint8Array([1]),
      encoding: 'zenoh/bytes',
      receivedAt: 1,
      kind: 'put',
    })
    controller.stop()
    expect(source.unsubscribed).toBe(true)
    expect(cancelled).toBe(true)
    callbacks[0]?.()
  })

  it('undeclares subscribers when stop runs during async source setup', async () => {
    const source = new StoppingDuringSetupSource()
    const { scheduler } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: () => undefined })

    controller.start()
    await delay(10)
    controller.stop()
    await delay(50)
    expect(source.dataUndeclare).toHaveBeenCalled()
    expect(source.livelinessUndeclare).not.toHaveBeenCalled()
  })

  it('surfaces source errors in view state', () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    source.handlers?.onError('zenoh offline')
    flush()
    expect(latestState(states).sourceError).toBe('zenoh offline')
    controller.stop()
  })

  it('stores raw query replies without adding registry topics', async () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const sample: SampleRecord = {
      key: 'blueos/v1/example/http/get',
      payload: new TextEncoder().encode('{"ok":true}'),
      encoding: 'application/json',
      receivedAt: 1,
      kind: 'put',
    }

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: {
        serviceInfo: vi.fn(),
        request: vi.fn(),
        rawQuery: vi.fn().mockResolvedValue([sample]),
      },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    flush()
    const topicsBefore = latestState(states).topicGroups.flatMap((group) => group.topics).length
    await controller.rawQuery(sample.key, '{}')
    flush()
    expect(latestState(states).topicGroups.flatMap((group) => group.topics)).toHaveLength(topicsBefore)
    expect(latestState(states).lastRequestResult).toEqual({
      status: 'success',
      result: {
        kind: 'replies',
        replies: [{ key: sample.key, decoded: { kind: 'json', value: { ok: true } } }],
      },
    })
    controller.stop()
  })

  it('skips selectedDecoded while the video view is active', () => {
    const source = new FakeSource()
    const { scheduler, flush } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const decode = vi.fn(cdrCodec.decode)
    const codec = { ...cdrCodec, decode }

    const states: InspectorViewState[] = []
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })

    controller.start()
    source.handlers?.onRos2Liveliness(ROS2DDS_CHATTER_TOKEN, true)
    source.handlers?.onSample({
      key: 'chatter',
      payload: stringCdrPayload('frame'),
      encoding: 'zenoh/bytes',
      receivedAt: 1,
      kind: 'put',
    })
    flush()
    controller.selectTopic('chatter')
    controller.selectView('video')
    flush()
    expect(latestState(states).selectedViewId).toBe('video')
    decode.mockClear()
    source.handlers?.onSample({
      key: 'chatter',
      payload: stringCdrPayload('frame2'),
      encoding: 'zenoh/bytes',
      receivedAt: 2,
      kind: 'put',
    })
    flush()
    expect(decode).not.toHaveBeenCalled()
    expect(latestState(states).selectedDecoded).toBeNull()
    controller.selectView('json')
    flush()
    expect(latestState(states).selectedDecoded).toEqual({
      kind: 'cdr',
      schemaName: STD_STRING_SCHEMA,
      value: { data: 'frame2' },
    })
    controller.stop()
  })

  it('notifies selected-sample subscribers on every frame for the selected key', () => {
    const source = new FakeSource()
    const { scheduler } = createFakeScheduler()
    const provider = deferredCatalogProvider()
    void provider.ready()
    provider.resolveCatalog()

    const frames: SampleRecord[] = []
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: vi.fn(), request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider: provider,
      codec: cdrCodec,
      scheduler,
    }, { onState: () => undefined })

    controller.start()
    controller.selectTopic('video/topic')
    controller.subscribeSelectedSample((sample) => frames.push(sample))

    const first: SampleRecord = {
      key: 'video/topic',
      payload: new Uint8Array([1]),
      encoding: 'zenoh/bytes',
      receivedAt: 1,
      kind: 'put',
    }
    const second: SampleRecord = {
      key: 'video/topic',
      payload: new Uint8Array([2]),
      encoding: 'zenoh/bytes',
      receivedAt: 2,
      kind: 'put',
    }
    source.handlers?.onSample(first)
    source.handlers?.onSample(second)
    expect(frames).toEqual([first, second])
    controller.stop()
  })
})
