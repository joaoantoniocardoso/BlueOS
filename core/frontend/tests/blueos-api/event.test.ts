/* eslint-disable import/no-extraneous-dependencies */
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { operation } from '@/libs/blueos-api/services/recorder'
import { watchEvent } from '@/libs/blueos-api/watch-event'

import FakeTransport from './fake-transport'

describe('watchEvent', () => {
  it('decodes each published event sample', async () => {
    const transport = new FakeTransport()
    const events: unknown[] = []
    const watching = watchEvent(transport, operation, {
      onValue: (message) => events.push(message),
      onError: () => undefined,
    })
    await watching

    transport.publish({
      key: operation.key,
      payload: encodeCdr(operation.messageSchema, {
        operation: 1,
        path: 'live.mcap',
        output_path: 'live.snapshot-2024-01-02T03-04-05Z.mcap',
        succeeded: true,
        cancelled: false,
        error: '',
      }),
      encoding: cdrEncoding(operation.messageSchema),
    })

    expect(events).toHaveLength(1)
    expect(events[0]).toMatchObject({
      path: 'live.mcap',
      output_path: 'live.snapshot-2024-01-02T03-04-05Z.mcap',
      succeeded: true,
    })
  })
})
