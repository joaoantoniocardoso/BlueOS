/* eslint-disable import/no-extraneous-dependencies */
import type { ServiceInfo } from '@blueos-idl/messages'
import { SCHEMAS } from '@blueos-idl/schemas'
import {
  describe, expect, it, vi,
} from 'vitest'

import { encodeCdrWithSchema } from '@/libs/blueos-api/cdr'
import { cdrCodec } from '@/libs/zenoh-inspector/adapters/cdr-codec'
import {
  InspectorController,
  type InspectorSource,
  type InspectorSourceHandlers,
  type InspectorViewState,
} from '@/libs/zenoh-inspector/inspector-controller'
import type { FrameScheduler } from '@/libs/zenoh-inspector/logic/types'

const FEEDBACK_LIST = 'blueos_msgs/msg/JobFeedbackList'
const SNAPSHOT_FEEDBACK = 'blueos_recorder_msgs/action/SnapshotRecording_Feedback'
const FEEDBACK_KEY = 'blueos/v1/recorder/jobs/SnapshotRecording/feedback'

const schemaProvider = {
  schemaText: (schemaName: string) => SCHEMAS[schemaName as keyof typeof SCHEMAS],
  ready: async () => Promise.resolve(),
}

const serviceInfo: ServiceInfo = {
  name: 'recorder',
  version: '1',
  build: 'test',
  capabilities: [],
  endpoints: [{
    kind: 'state',
    name: 'jobs/SnapshotRecording/feedback',
    key: FEEDBACK_KEY,
    interface_type: FEEDBACK_LIST,
    schema: `${SCHEMAS[FEEDBACK_LIST]}\n${'='.repeat(80)}\nMSG: blueos_recorder_msgs/SnapshotRecording_Feedback\n${
      SCHEMAS[SNAPSHOT_FEEDBACK]}`,
  }],
}

describe('InspectorController Job outputs', () => {
  it('shows the Feedback fields of a Job type once the info of its Service arrives', async () => {
    let handlers: InspectorSourceHandlers | null = null
    const source: InspectorSource = {
      start(sourceHandlers) {
        handlers = sourceHandlers
        return () => undefined
      },
    }
    const callbacks: Array<() => void> = []
    const scheduler: FrameScheduler = {
      schedule: (callback) => callbacks.push(callback),
      cancel: () => undefined,
    }
    function flush(): void {
      while (callbacks.length > 0) {
        callbacks.shift()?.()
      }
    }
    const states: InspectorViewState[] = []
    const serviceInfoQuery = vi.fn().mockResolvedValue(serviceInfo)
    const controller = new InspectorController({
      source,
      apiClient: { serviceInfo: serviceInfoQuery, request: vi.fn(), rawQuery: vi.fn() },
      schemaProvider,
      codec: cdrCodec,
      scheduler,
    }, { onState: (state) => states.push({ ...state }) })
    const feedback = encodeCdrWithSchema(SNAPSHOT_FEEDBACK, SCHEMAS[SNAPSHOT_FEEDBACK], {
      output_path: '/data/a.snapshot.mcap',
    })

    controller.start();
    (handlers as InspectorSourceHandlers | null)?.onSample({
      key: FEEDBACK_KEY,
      payload: encodeCdrWithSchema(FEEDBACK_LIST, SCHEMAS[FEEDBACK_LIST], {
        jobs: [{ job_id: 'job-a', feedback: Array.from(feedback) }],
      }),
      encoding: `application/cdr;${FEEDBACK_LIST}`,
      receivedAt: 1,
      kind: 'put',
    })
    flush()
    controller.selectTopic(FEEDBACK_KEY)
    await Promise.resolve()
    flush()
    controller.stop()

    expect(serviceInfoQuery).toHaveBeenCalledWith('recorder')
    expect(states[states.length - 1].selectedDecoded).toEqual({
      kind: 'cdr',
      schemaName: FEEDBACK_LIST,
      value: { jobs: [{ job_id: 'job-a', feedback: { output_path: '/data/a.snapshot.mcap' } }] },
    })
  })
})
