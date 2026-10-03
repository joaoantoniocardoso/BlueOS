/* eslint-disable import/no-extraneous-dependencies */
import { CommandAckStatus, JobStatusStatus } from '@blueos-idl/constants'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import {
  answerPermission, cancelJob, jobHistory, newJobId, pauseJob, query, resumeJob, sendCommand,
} from '@/libs/blueos-api/command'
import { updateSettingsCommand } from '@/libs/blueos-api/endpoints'
import { NoReplyError, QueryFailedError, UnexpectedEncodingError } from '@/libs/blueos-api/errors'
import {
  cdrEncoding, commandKey, ENCODING_APPLICATION_CDR, jobHistoryKey,
} from '@/libs/blueos-api/keys'
import { Level, SetLevel } from '@/libs/blueos-api/services/example'
import type { Reply } from '@/libs/blueos-api/transport'
import type { MessageForSchema, SchemaName } from '@/libs/blueos-api/types'

import FakeTransport from './fake-transport'

const JOB_ID = '0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10'

function answer<Schema extends SchemaName>(key: string, schema: Schema, message: MessageForSchema<Schema>): Reply {
  return { kind: 'sample', sample: { key, payload: encodeCdr(schema, message), encoding: cdrEncoding(schema) } }
}

function attachedJobId(attachment: Uint8Array | undefined): string {
  return new TextDecoder().decode(attachment)
}

describe('newJobId', () => {
  it('is a new lowercase version 4 UUID every time', () => {
    const first = newJobId()
    const second = newJobId()

    expect(first).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/)
    expect(second).not.toBe(first)
  })
})

describe('sendCommand', () => {
  it('submits the request as CDR under a new Job id and returns the decoded CommandAck', async () => {
    const transport = new FakeTransport()
    const ack = {
      accepted: true, job_id: '', status: CommandAckStatus.Succeeded, reason: '',
    }

    const sending = sendCommand(transport, SetLevel, { level: 3 })
    const sent = await transport.nextQuery()
    const jobId = attachedJobId(sent.body?.attachment)
    sent.reply(answer(SetLevel.key, 'blueos_msgs/msg/CommandAck', { ...ack, job_id: jobId }))

    expect(await sending).toEqual({ ...ack, job_id: jobId })
    expect(jobId).toMatch(/^[0-9a-f-]{36}$/)
    expect(sent.key).toBe(SetLevel.key)
    expect(sent.body?.payload).toEqual(encodeCdr(SetLevel.goalSchema, { level: 3 }))
    expect(sent.body?.encoding).toBe(cdrEncoding(SetLevel.goalSchema))
  })

  it('resubmits under the Job id it is given, so a retry returns the same Job', async () => {
    const transport = new FakeTransport()

    const sending = sendCommand(transport, SetLevel, { level: 3 }, JOB_ID)
    const sent = await transport.nextQuery()
    sent.reply(answer(SetLevel.key, 'blueos_msgs/msg/CommandAck', {
      accepted: true, job_id: JOB_ID, status: CommandAckStatus.Executing, reason: '',
    }))

    expect(attachedJobId(sent.body?.attachment)).toBe(JOB_ID)
    expect((await sending).status).toBe(CommandAckStatus.Executing)
  })

  it('returns a rejection as a CommandAck with its reason', async () => {
    const transport = new FakeTransport()
    const rejection = {
      accepted: false, job_id: JOB_ID, status: CommandAckStatus.StatusUnknown, reason: 'id reused',
    }

    const sending = sendCommand(transport, SetLevel, { level: 3 }, JOB_ID)
    const sent = await transport.nextQuery()
    sent.reply(answer(SetLevel.key, 'blueos_msgs/msg/CommandAck', rejection))

    expect(await sending).toEqual(rejection)
  })

  it('submits UpdateSettings as a Job and returns its final status', async () => {
    const transport = new FakeTransport()
    const envelope = { document_json: '{"VERSION":1}', fields: [] }

    const sending = sendCommand(transport, updateSettingsCommand('tank'), envelope)
    const sent = await transport.nextQuery()
    const jobId = attachedJobId(sent.body?.attachment)
    sent.reply(answer(sent.key, 'blueos_msgs/msg/CommandAck', {
      accepted: true, job_id: jobId, status: CommandAckStatus.Succeeded, reason: '',
    }))

    expect(await sending).toEqual({
      accepted: true, job_id: jobId, status: CommandAckStatus.Succeeded, reason: '',
    })
    expect(jobId).toMatch(/^[0-9a-f-]{36}$/)
    expect(sent.key).toBe(commandKey('tank', 'UpdateSettings'))
    expect(sent.body?.payload).toEqual(encodeCdr('blueos_msgs/msg/SettingsEnvelope', envelope))
  })

  it('fails with NoReplyError when no Service answers', async () => {
    const transport = new FakeTransport()

    const sending = sendCommand(transport, SetLevel, { level: 3 })
    const sent = await transport.nextQuery()
    sent.reply()

    await expect(sending).rejects.toEqual(new NoReplyError(SetLevel.key))
  })

  it('fails with QueryFailedError when the queryable replies with an error', async () => {
    const transport = new FakeTransport()

    const sending = sendCommand(transport, SetLevel, { level: 3 })
    const sent = await transport.nextQuery()
    sent.reply({ kind: 'error', payload: new TextEncoder().encode('inbox closed'), encoding: 'text/plain' })

    await expect(sending).rejects.toEqual(new QueryFailedError(SetLevel.key, 'inbox closed'))
  })
})

describe('Job controls', () => {
  it.each([
    ['CancelJob', cancelJob, CommandAckStatus.Canceling],
    ['PauseJob', pauseJob, CommandAckStatus.Paused],
    ['ResumeJob', resumeJob, CommandAckStatus.Executing],
  ] as const)('%s sends the Job id with no body and returns the ack', async (name, control, status) => {
    const transport = new FakeTransport()
    const ack = {
      accepted: true, job_id: JOB_ID, status, reason: '',
    }

    const sending = control(transport, 'tank', JOB_ID)
    const sent = await transport.nextQuery()
    sent.reply(answer(sent.key, 'blueos_msgs/msg/CommandAck', ack))

    expect(await sending).toEqual(ack)
    expect(sent.key).toBe(commandKey('tank', name))
    expect(attachedJobId(sent.body?.attachment)).toBe(JOB_ID)
    expect(sent.body?.payload).toEqual(new Uint8Array())
    expect(sent.body?.encoding).toBe(ENCODING_APPLICATION_CDR)
  })

  it('returns the rejection of a control the Job type does not allow', async () => {
    const transport = new FakeTransport()
    const rejection = {
      accepted: false, job_id: JOB_ID, status: CommandAckStatus.Executing, reason: 'Steep does not allow PauseJob',
    }

    const sending = pauseJob(transport, 'tank', JOB_ID)
    const sent = await transport.nextQuery()
    sent.reply(answer(sent.key, 'blueos_msgs/msg/CommandAck', rejection))

    expect(await sending).toEqual(rejection)
  })

  it('AnswerPermission sends the answer as a PermissionAnswer', async () => {
    const transport = new FakeTransport()

    const sending = answerPermission(transport, 'tank', JOB_ID, false)
    const sent = await transport.nextQuery()
    sent.reply(answer(sent.key, 'blueos_msgs/msg/CommandAck', {
      accepted: true, job_id: JOB_ID, status: CommandAckStatus.Canceled, reason: 'permission denied',
    }))

    expect((await sending).reason).toBe('permission denied')
    expect(sent.key).toBe(commandKey('tank', 'AnswerPermission'))
    expect(attachedJobId(sent.body?.attachment)).toBe(JOB_ID)
    expect(sent.body?.payload).toEqual(encodeCdr('blueos_msgs/msg/PermissionAnswer', { granted: false }))
    expect(sent.body?.encoding).toBe(cdrEncoding('blueos_msgs/msg/PermissionAnswer'))
  })
})

describe('query', () => {
  it('sends the request and returns the decoded response', async () => {
    const transport = new FakeTransport()

    const asking = query(transport, Level, {})
    const sent = await transport.nextQuery()
    sent.reply(answer(Level.key, Level.responseSchema, { level: 4, max_level: 10 }))

    expect(await asking).toEqual({ level: 4, max_level: 10 })
    expect(sent.key).toBe(Level.key)
    expect(sent.body?.encoding).toBe(cdrEncoding(Level.requestSchema))
    expect(sent.body?.attachment).toBeUndefined()
  })

  it('refuses a reply that is not the declared response', async () => {
    const transport = new FakeTransport()

    const asking = query(transport, Level, {})
    const sent = await transport.nextQuery()
    sent.reply(answer(Level.key, 'blueos_msgs/msg/CommandAck', {
      accepted: true, job_id: '', status: CommandAckStatus.StatusUnknown, reason: '',
    }))

    await expect(asking).rejects.toBeInstanceOf(UnexpectedEncodingError)
  })
})

describe('jobHistory', () => {
  it('reads the last finished Jobs of a Job type, in the order they ended', async () => {
    const transport = new FakeTransport()
    const history = {
      jobs: [
        {
          job_id: 'job-1', job_type: 'Fill', status: JobStatusStatus.Succeeded, reason: '',
        },
        {
          job_id: 'job-2', job_type: 'Fill', status: JobStatusStatus.Aborted, reason: 'tank full',
        },
      ],
    }

    const reading = jobHistory(transport, 'tank', 'Fill')
    const sent = await transport.nextQuery()
    sent.reply(answer(jobHistoryKey('tank', 'Fill'), 'blueos_msgs/msg/JobList', history))

    expect(await reading).toEqual(history)
    expect(sent.key).toBe(jobHistoryKey('tank', 'Fill'))
    expect(sent.body).toBeUndefined()
  })
})
