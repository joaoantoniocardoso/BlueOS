/* eslint-disable import/no-extraneous-dependencies */
import { JobStatusStatus, ServiceStatusStatus } from '@blueos-idl/constants'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { jobsState, settingsState, statusState } from '@/libs/blueos-api/endpoints'
import { QueryFailedError, UnexpectedEncodingError } from '@/libs/blueos-api/errors'
import {
  cdrEncoding, jobsKey, settingsKey, statusStateKey,
} from '@/libs/blueos-api/keys'
import { tank } from '@/libs/blueos-api/services/tank'
import type { Sample } from '@/libs/blueos-api/transport'
import type { MessageForSchema, SchemaName } from '@/libs/blueos-api/types'
import { watchState } from '@/libs/blueos-api/watch'

import FakeTransport from './fake-transport'

interface Observed<Message> {
  values: { message: Message, key: string }[]
  errors: unknown[]
  onValue: (message: Message, key: string) => void
  onError: (error: unknown) => void
}

function observe<Message>(): Observed<Message> {
  const values: { message: Message, key: string }[] = []
  const errors: unknown[] = []
  return {
    values,
    errors,
    onValue: (message, key) => values.push({ message, key }),
    onError: (error) => errors.push(error),
  }
}

function sample<Schema extends SchemaName>(key: string, schema: Schema, message: MessageForSchema<Schema>): Sample {
  return { key, payload: encodeCdr(schema, message), encoding: cdrEncoding(schema) }
}

function level(value: number): Sample {
  return sample(tank.key, tank.messageSchema, { level: value, max_level: 10 })
}

function levels(observed: Observed<MessageForSchema<typeof tank.messageSchema>>): number[] {
  return observed.values.map(({ message }) => message.level)
}

describe('watchState', () => {
  it('keeps an update published after subscribing and before querying, and ignores the older reply', async () => {
    const transport = new FakeTransport()
    transport.afterSubscribe = () => transport.publish(level(2))
    const observed = observe<MessageForSchema<typeof tank.messageSchema>>()

    const watching = watchState(transport, tank, observed)
    const query = await transport.nextQuery()
    query.reply({ kind: 'sample', sample: level(1) })
    await watching

    expect(levels(observed)).toEqual([2])
  })

  it('never loses an update published while the query is pending', async () => {
    const transport = new FakeTransport()
    const observed = observe<MessageForSchema<typeof tank.messageSchema>>()

    const watching = watchState(transport, tank, observed)
    const query = await transport.nextQuery()
    transport.publish(level(2))
    query.reply({ kind: 'sample', sample: level(1) })
    await watching

    expect(transport.subscribers.map(({ key }) => key)).toEqual([tank.key])
    expect(query.key).toBe(tank.key)
    expect(query.body).toBeUndefined()
    expect(levels(observed)).toEqual([2])
  })

  it('delivers the reply when no update came first, then every update', async () => {
    const transport = new FakeTransport()
    const observed = observe<MessageForSchema<typeof tank.messageSchema>>()

    const watching = watchState(transport, tank, observed)
    const query = await transport.nextQuery()
    query.reply({ kind: 'sample', sample: level(1) })
    await watching
    transport.publish(level(2))
    transport.publish(level(3))

    expect(levels(observed)).toEqual([1, 2, 3])
    expect(observed.values[0].key).toBe(tank.key)
  })

  it('waits for the first update when the State has no value yet', async () => {
    const transport = new FakeTransport()
    const observed = observe<MessageForSchema<typeof tank.messageSchema>>()

    const watching = watchState(transport, tank, observed)
    const query = await transport.nextQuery()
    query.reply()
    await watching
    transport.publish(level(5))

    expect(levels(observed)).toEqual([5])
  })

  it('tells the keys of a wildcard apart, ignoring only the reply of a key that was updated', async () => {
    const transport = new FakeTransport()
    const observed = observe<MessageForSchema<'blueos_msgs/msg/ServiceStatus'>>()
    function status(service: string, value: number): Sample {
      return sample(statusStateKey(service), 'blueos_msgs/msg/ServiceStatus', { status: value, detail: '' })
    }

    const watching = watchState(transport, statusState('*'), observed)
    const query = await transport.nextQuery()
    transport.publish(status('tank', ServiceStatusStatus.Degraded))
    query.reply(
      { kind: 'sample', sample: status('tank', ServiceStatusStatus.Ready) },
      { kind: 'sample', sample: status('recorder', ServiceStatusStatus.Ready) },
    )
    await watching

    expect(query.key).toBe('blueos/v1/*/state/status')
    expect(observed.values.map(({ message, key }) => [key, message.status])).toEqual([
      [statusStateKey('tank'), ServiceStatusStatus.Degraded],
      [statusStateKey('recorder'), ServiceStatusStatus.Ready],
    ])
  })

  it('watches the settings and the jobs of a Service with the same helper', async () => {
    const transport = new FakeTransport()
    const settings = observe<MessageForSchema<'blueos_msgs/msg/SettingsEnvelope'>>()
    const jobs = observe<MessageForSchema<'blueos_msgs/msg/JobList'>>()
    const envelope = { document_json: '{"VERSION":1}', fields: [{ path: 'auto_start', restart_required: true }] }
    const jobList = {
      jobs: [{
        job_id: 1, parent_job_id: 0, status: JobStatusStatus.Running, name: 'repair',
      }],
    }

    const watchingSettings = watchState(transport, settingsState('tank'), settings)
    const settingsQuery = await transport.nextQuery()
    settingsQuery.reply({
      kind: 'sample', sample: sample(settingsKey('tank'), 'blueos_msgs/msg/SettingsEnvelope', envelope),
    })
    const watchingJobs = watchState(transport, jobsState('tank'), jobs)
    const jobsQuery = await transport.nextQuery()
    jobsQuery.reply({ kind: 'sample', sample: sample(jobsKey('tank'), 'blueos_msgs/msg/JobList', jobList) })
    await Promise.all([watchingSettings, watchingJobs])

    expect(settings.values).toEqual([{ message: envelope, key: settingsKey('tank') }])
    expect(jobs.values).toEqual([{ message: jobList, key: jobsKey('tank') }])
  })

  it('reports a sample it cannot decode and keeps watching', async () => {
    const transport = new FakeTransport()
    const observed = observe<MessageForSchema<typeof tank.messageSchema>>()

    const watching = watchState(transport, tank, observed)
    const query = await transport.nextQuery()
    query.reply()
    await watching
    transport.publish({ ...level(1), encoding: 'application/json' })
    transport.publish(level(2))

    expect(observed.errors).toHaveLength(1)
    expect(observed.errors[0]).toBeInstanceOf(UnexpectedEncodingError)
    expect(levels(observed)).toEqual([2])
  })

  it('reports an error reply and keeps watching', async () => {
    const transport = new FakeTransport()
    const observed = observe<MessageForSchema<typeof tank.messageSchema>>()

    const watching = watchState(transport, tank, observed)
    const query = await transport.nextQuery()
    query.reply({ kind: 'error', payload: new TextEncoder().encode('busy'), encoding: 'text/plain' })
    await watching
    transport.publish(level(2))

    expect(observed.errors).toEqual([new QueryFailedError(tank.key, 'busy')])
    expect(levels(observed)).toEqual([2])
  })

  it('closes its subscriber when the query fails', async () => {
    const transport = new FakeTransport()
    const observed = observe<MessageForSchema<typeof tank.messageSchema>>()

    const watching = watchState(transport, tank, observed)
    const query = await transport.nextQuery()
    query.fail(new Error('router unreachable'))

    await expect(watching).rejects.toThrow('router unreachable')
    expect(transport.subscribers.map(({ open }) => open)).toEqual([false])
  })

  it('delivers nothing once closed', async () => {
    const transport = new FakeTransport()
    const observed = observe<MessageForSchema<typeof tank.messageSchema>>()

    const watching = watchState(transport, tank, observed)
    const query = await transport.nextQuery()
    query.reply()
    const subscription = await watching
    await subscription.close()
    transport.publish(level(2))

    expect(levels(observed)).toEqual([])
  })
})
