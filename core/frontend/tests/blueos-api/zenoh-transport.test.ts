import {
  CongestionControl,
  Encoding,
  FifoChannel,
  type GetOptions,
  KeyExpr,
  Priority,
  Reply,
  ReplyError,
  Sample,
  SampleKind,
  type Session,
  type SubscriberOptions,
  ZBytes,
} from '@eclipse-zenoh/zenoh-ts'
import { describe, expect, it } from 'vitest'

import zenohTransport from '@/libs/blueos-api/zenoh-transport'

function zenohSample(key: string, kind: SampleKind, payload: number[]): Sample {
  return new Sample(
    new KeyExpr(key),
    new ZBytes(new Uint8Array(payload)),
    kind,
    Encoding.fromString('application/cdr;blueos_msgs/msg/CommandAck'),
    undefined,
    undefined,
    Priority.DATA,
    CongestionControl.DROP,
    false,
  )
}

describe('zenohTransport', () => {
  it('passes on every put to a subscriber and closes it by undeclaring', async () => {
    let handler: ((sample: Sample) => void) | undefined
    let undeclared = false
    const session = {
      declareSubscriber: async (key: string, options: SubscriberOptions) => {
        expect(key).toBe('blueos/v1/tank/state/tank')
        handler = options.handler as (sample: Sample) => void
        return {
          undeclare: async () => {
            undeclared = true
          },
        }
      },
    } as unknown as Session
    const received: unknown[] = []

    const subscription = await zenohTransport(session).subscribe('blueos/v1/tank/state/tank', (sample) => {
      received.push(sample)
    })
    handler?.(zenohSample('blueos/v1/tank/state/tank', SampleKind.PUT, [1, 2]))
    handler?.(zenohSample('blueos/v1/tank/state/tank', SampleKind.DELETE, []))
    await subscription.close()

    expect(received).toEqual([{
      key: 'blueos/v1/tank/state/tank',
      payload: new Uint8Array([1, 2]),
      encoding: 'application/cdr;blueos_msgs/msg/CommandAck',
    }])
    expect(undeclared).toBe(true)
  })

  it('sends the body and returns every reply once the query completes', async () => {
    let sent: GetOptions | undefined
    const session = {
      get: async (key: string, options: GetOptions) => {
        expect(key).toBe('blueos/v1/tank/command/Drain')
        sent = options
        const replies = new FifoChannel<Reply>(8)
        replies.send(new Reply(zenohSample('blueos/v1/tank/command/Drain', SampleKind.PUT, [3])))
        replies.send(new Reply(new ReplyError(new ZBytes('busy'), Encoding.fromString('text/plain'))))
        replies.close()
        return replies
      },
    } as unknown as Session

    const replies = await zenohTransport(session).get('blueos/v1/tank/command/Drain', {
      payload: new Uint8Array([0]),
      encoding: 'application/cdr;blueos_example_msgs/msg/EmptyRequest',
    })

    expect(replies).toEqual([
      {
        kind: 'sample',
        sample: {
          key: 'blueos/v1/tank/command/Drain',
          payload: new Uint8Array([3]),
          encoding: 'application/cdr;blueos_msgs/msg/CommandAck',
        },
      },
      { kind: 'error', payload: new TextEncoder().encode('busy'), encoding: 'text/plain' },
    ])
    expect((sent?.payload as ZBytes).toBytes()).toEqual(new Uint8Array([0]))
    expect(sent?.encoding?.toString()).toBe('application/cdr;blueos_example_msgs/msg/EmptyRequest')
  })
})
