/* eslint-disable import/no-extraneous-dependencies */
import type { EndpointInfo } from '@blueos-idl/messages'
import { SCHEMAS } from '@blueos-idl/schemas'
import { describe, expect, it } from 'vitest'

import { encodeCdrWithSchema } from '@/libs/blueos-api/cdr'
import { cdrCodec } from '@/libs/zenoh-inspector/adapters/cdr-codec'
import { decodePayload } from '@/libs/zenoh-inspector/logic/decode'
import { unwrapJobPart } from '@/libs/zenoh-inspector/logic/job-parts'
import type {
  DecodedPayload, SampleRecord, SchemaProvider, TopicInfo,
} from '@/libs/zenoh-inspector/logic/types'

const FEEDBACK_LIST = 'blueos_msgs/msg/JobFeedbackList'
const JOB_RESULT = 'blueos_msgs/msg/JobResult'
const SNAPSHOT_FEEDBACK = 'blueos_recorder_msgs/action/SnapshotRecording_Feedback'
const SNAPSHOT_RESULT = 'blueos_recorder_msgs/action/SnapshotRecording_Result'
const FEEDBACK_KEY = 'blueos/v1/recorder/jobs/SnapshotRecording/feedback'
const RESULT_KEY = 'blueos/v1/recorder/jobs/SnapshotRecording/result'

const provider: SchemaProvider = {
  schemaText: (schemaName) => SCHEMAS[schemaName as keyof typeof SCHEMAS],
}

// The `schema` of an info reply for a Job output key: the wrapper's schema, then the carried part as a `MSG:` section.
function carrying(wrapper: string, partName: string, partSchema: string): string {
  return `${SCHEMAS[wrapper as keyof typeof SCHEMAS]}\n${'='.repeat(80)}\nMSG: ${partName}\n${
    SCHEMAS[partSchema as keyof typeof SCHEMAS]}`
}

function endpoint(name: string, key: string, kind: string, interfaceType: string, schema: string): EndpointInfo {
  return {
    kind, name, key, interface_type: interfaceType, schema,
  }
}

const ENDPOINTS: EndpointInfo[] = [
  endpoint(
    'jobs/SnapshotRecording/feedback',
    FEEDBACK_KEY,
    'state',
    FEEDBACK_LIST,
    carrying(FEEDBACK_LIST, 'blueos_recorder_msgs/SnapshotRecording_Feedback', SNAPSHOT_FEEDBACK),
  ),
  endpoint(
    'jobs/SnapshotRecording/result',
    RESULT_KEY,
    'event',
    JOB_RESULT,
    carrying(JOB_RESULT, 'blueos_recorder_msgs/SnapshotRecording_Result', SNAPSHOT_RESULT),
  ),
]

function part(schemaName: string, message: Record<string, unknown>): Uint8Array {
  return encodeCdrWithSchema(schemaName, SCHEMAS[schemaName as keyof typeof SCHEMAS], message)
}

function decodedSample(key: string, schemaName: string, message: Record<string, unknown>): DecodedPayload {
  const sample: SampleRecord = {
    key,
    payload: encodeCdrWithSchema(schemaName, SCHEMAS[schemaName as keyof typeof SCHEMAS], message),
    encoding: `application/cdr;${schemaName}`,
    receivedAt: 1,
    kind: 'put',
  }
  const topic: TopicInfo = {
    key, source: 'blueos', sampleCount: 1, lastSample: sample,
  }
  return decodePayload(topic, undefined, provider, cdrCodec)
}

function job(jobId: string, status: number): Record<string, unknown> {
  return {
    job_id: jobId, job_type: 'SnapshotRecording', status, reason: '',
  }
}

describe('unwrapJobPart', () => {
  it('shows the Feedback of each active Job with the fields of its Job type', () => {
    const wire = decodedSample(FEEDBACK_KEY, FEEDBACK_LIST, {
      jobs: [
        { job_id: 'job-a', feedback: part(SNAPSHOT_FEEDBACK, { output_path: '/data/a.snapshot.mcap' }) },
        { job_id: 'job-b', feedback: part(SNAPSHOT_FEEDBACK, { output_path: '/data/b.snapshot.mcap' }) },
      ],
    })

    expect(unwrapJobPart(wire, FEEDBACK_KEY, ENDPOINTS, provider, cdrCodec)).toEqual({
      kind: 'cdr',
      schemaName: FEEDBACK_LIST,
      value: {
        jobs: [
          { job_id: 'job-a', feedback: { output_path: '/data/a.snapshot.mcap' } },
          { job_id: 'job-b', feedback: { output_path: '/data/b.snapshot.mcap' } },
        ],
      },
    })
  })

  it('shows the Job result of an ended Job with the fields of its Job type', () => {
    const wire = decodedSample(RESULT_KEY, JOB_RESULT, {
      job: job('job-a', 4),
      result: part(SNAPSHOT_RESULT, { path: '/data/a.mcap', output_path: '/data/a.snapshot.mcap' }),
    })

    expect(unwrapJobPart(wire, RESULT_KEY, ENDPOINTS, provider, cdrCodec)).toEqual({
      kind: 'cdr',
      schemaName: JOB_RESULT,
      value: {
        job: job('job-a', 4),
        result: { path: '/data/a.mcap', output_path: '/data/a.snapshot.mcap' },
      },
    })
  })

  it('keeps the bytes of a Job result the Job type declares none for', () => {
    const wire = decodedSample(RESULT_KEY, JOB_RESULT, { job: job('job-a', 5), result: new Uint8Array() })

    expect(unwrapJobPart(wire, RESULT_KEY, ENDPOINTS, provider, cdrCodec)).toEqual(wire)
  })

  it('shows a Feedback with no fields as an empty message', () => {
    const wire = decodedSample(FEEDBACK_KEY, FEEDBACK_LIST, {
      jobs: [{ job_id: 'job-a', feedback: new Uint8Array([0, 1, 0, 0, 0]) }],
    })
    const empty = ENDPOINTS.map((entry) => ({
      ...entry,
      schema: `${SCHEMAS[FEEDBACK_LIST]}\n${'='.repeat(80)}\nMSG: blueos_recorder_msgs/StartRecording_Feedback\n`,
    }))

    expect(unwrapJobPart(wire, FEEDBACK_KEY, empty, provider, cdrCodec)).toEqual({
      kind: 'cdr',
      schemaName: FEEDBACK_LIST,
      value: { jobs: [{ job_id: 'job-a', feedback: {} }] },
    })
  })

  it('leaves the payload as the wrapper when the info reply carries no part for the key', () => {
    const wire = decodedSample(FEEDBACK_KEY, FEEDBACK_LIST, {
      jobs: [{ job_id: 'job-a', feedback: part(SNAPSHOT_FEEDBACK, { output_path: '/data/a.snapshot.mcap' }) }],
    })
    const withoutPart = ENDPOINTS.map((entry) => ({ ...entry, schema: SCHEMAS[FEEDBACK_LIST] }))

    expect(unwrapJobPart(wire, FEEDBACK_KEY, withoutPart, provider, cdrCodec)).toEqual(wire)
    expect(unwrapJobPart(wire, FEEDBACK_KEY, [], provider, cdrCodec)).toEqual(wire)
  })

  it('does not touch a payload that is not a Job output wrapper', () => {
    const wire = decodedSample('blueos/v1/recorder/jobs', 'blueos_msgs/msg/JobList', { jobs: [job('job-a', 4)] })

    expect(unwrapJobPart(wire, 'blueos/v1/recorder/jobs', ENDPOINTS, provider, cdrCodec)).toEqual(wire)
  })
})
