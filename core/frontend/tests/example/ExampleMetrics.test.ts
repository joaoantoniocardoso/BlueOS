/* eslint-disable import/no-extraneous-dependencies */
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { metricsState } from '@/libs/blueos-api/endpoints'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { NAME } from '@/libs/blueos-api/services/example'
import type { Sample } from '@/libs/blueos-api/transport'
import { watchState } from '@/libs/blueos-api/watch'

import FakeTransport from '../blueos-api/fake-transport'

/**
 * Q35: how does the frontend read the `metrics` of a Service?
 *
 * Every Service publishes `metrics` (D-35), so there is no generated constant for it: `metricsState(service)` builds
 * the State endpoint and `watchState` reads it, as for any State. The Service answers a late client with its current
 * value, so a page opened mid-run shows numbers at once, without waiting for the next publication.
 */
describe('reading the metrics of a Service', () => {
  it('shows the current counters at once, then every change', async () => {
    const transport = new FakeTransport()
    const metrics = metricsState(NAME)
    function sample(value: number): Sample {
      return {
        key: metrics.key,
        payload: encodeCdr(metrics.messageSchema, {
          counters: [{ name: 'commands_handled', labels: [], value }],
          gauges: [],
          histograms: [],
        }),
        encoding: cdrEncoding(metrics.messageSchema),
      }
    }
    const counts: number[] = []

    const watching = watchState(transport, metrics, {
      onValue: (message) => counts.push(message.counters[0].value),
      onError: () => undefined,
    })
    // The late-client query is answered with the current value.
    const query = await transport.nextQuery()
    query.reply({ kind: 'sample', sample: sample(7) })
    await watching
    expect(counts).toEqual([7])

    // From then on the Service publishes each change.
    transport.publish(sample(8))
    expect(counts).toEqual([7, 8])
  })
})
