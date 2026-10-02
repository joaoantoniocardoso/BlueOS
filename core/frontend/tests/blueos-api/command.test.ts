import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { JOB_ID_NONE, query, sendCommand } from '@/libs/blueos-api/command'
import { updateSettingsCommand } from '@/libs/blueos-api/endpoints'
import { NoReplyError, QueryFailedError, UnexpectedEncodingError } from '@/libs/blueos-api/errors'
import { cdrEncoding, commandKey } from '@/libs/blueos-api/keys'
import { Level, SetLevel } from '@/libs/blueos-api/services/example'
import type { Reply } from '@/libs/blueos-api/transport'
import type { MessageForSchema, SchemaName } from '@/libs/blueos-api/types'

import FakeTransport from './fake-transport'

function answer<Schema extends SchemaName>(key: string, schema: Schema, message: MessageForSchema<Schema>): Reply {
  return { kind: 'sample', sample: { key, payload: encodeCdr(schema, message), encoding: cdrEncoding(schema) } }
}

describe('sendCommand', () => {
  it('sends the request as CDR on the Command key and returns the decoded CommandAck', async () => {
    const transport = new FakeTransport()

    const sending = sendCommand(transport, SetLevel, { level: 3 })
    const sent = await transport.nextQuery()
    sent.reply(answer(SetLevel.key, 'blueos_msgs/msg/CommandAck', { accepted: true, job_id: 1, reason: '' }))

    expect(await sending).toEqual({ accepted: true, job_id: 1, reason: '' })
    expect(sent.key).toBe(SetLevel.key)
    expect(sent.body).toEqual({
      payload: encodeCdr(SetLevel.requestSchema, { level: 3 }),
      encoding: cdrEncoding(SetLevel.requestSchema),
    })
  })

  it('returns a rejection as a CommandAck with its reason', async () => {
    const transport = new FakeTransport()
    const rejection = { accepted: false, job_id: JOB_ID_NONE, reason: 'the level is above the maximum' }

    const sending = sendCommand(transport, SetLevel, { level: 3 })
    const sent = await transport.nextQuery()
    sent.reply(answer(SetLevel.key, 'blueos_msgs/msg/CommandAck', rejection))

    expect(await sending).toEqual(rejection)
  })

  it('sends UpdateSettings like any other Command', async () => {
    const transport = new FakeTransport()
    const envelope = { document_json: '{"VERSION":1}', fields: [] }

    const sending = sendCommand(transport, updateSettingsCommand('tank'), envelope)
    const sent = await transport.nextQuery()
    sent.reply(answer(sent.key, 'blueos_msgs/msg/CommandAck', { accepted: true, job_id: JOB_ID_NONE, reason: '' }))

    expect((await sending).accepted).toBe(true)
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

describe('query', () => {
  it('sends the request and returns the decoded response', async () => {
    const transport = new FakeTransport()

    const asking = query(transport, Level, { padding: 0 })
    const sent = await transport.nextQuery()
    sent.reply(answer(Level.key, Level.responseSchema, { level: 4, max_level: 10 }))

    expect(await asking).toEqual({ level: 4, max_level: 10 })
    expect(sent.key).toBe(Level.key)
    expect(sent.body?.encoding).toBe(cdrEncoding(Level.requestSchema))
  })

  it('refuses a reply that is not the declared response', async () => {
    const transport = new FakeTransport()

    const asking = query(transport, Level, { padding: 0 })
    const sent = await transport.nextQuery()
    sent.reply(answer(Level.key, 'blueos_msgs/msg/CommandAck', { accepted: true, job_id: 0, reason: '' }))

    await expect(asking).rejects.toBeInstanceOf(UnexpectedEncodingError)
  })
})
