/* eslint-disable import/no-extraneous-dependencies */
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { jobResultEvent } from '@/libs/blueos-api/endpoints'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { watchEvent } from '@/libs/blueos-api/watch-event'

import FakeTransport from './fake-transport'

describe('watchEvent', () => {
  it('decodes each published event sample', async () => {
    const transport = new FakeTransport()
    const events: unknown[] = []
    const results = jobResultEvent('recorder', 'SnapshotRecording')
    const watching = watchEvent(transport, results, {
      onValue: (message) => events.push(message),
      onError: () => undefined,
    })
    await watching

    transport.publish({
      key: results.key,
      payload: encodeCdr(results.messageSchema, {
        job: {
          job_id: '0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10',
          job_type: 'SnapshotRecording',
          status: 4,
          reason: '',
        },
        result: [],
      }),
      encoding: cdrEncoding(results.messageSchema),
    })

    expect(events).toHaveLength(1)
    expect(events[0]).toMatchObject({
      job: { job_type: 'SnapshotRecording', status: 4 },
    })
  })
})
