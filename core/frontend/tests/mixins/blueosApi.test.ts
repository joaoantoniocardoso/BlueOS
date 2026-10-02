/* eslint-disable import/no-extraneous-dependencies */
import { describe, expect, it } from 'vitest'
import Vue from 'vue'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { pump } from '@/libs/blueos-api/services/example'
import { blueosApiMixin } from '@/mixins/blueosApi'

import FakeTransport from '../blueos-api/fake-transport'

describe('blueosApiMixin', () => {
  it('re-renders when a bound State changes', async () => {
    const transport = new FakeTransport()
    let renderCount = 0
    const Component = Vue.extend({
      mixins: [blueosApiMixin],
      data() {
        return { level: null as number | null }
      },
      async created() {
        await this.blueosWatchState(transport, pump, (message) => {
          this.level = message.level
        })
      },
      render(createElement) {
        renderCount += 1
        return createElement('span', String(this.level))
      },
    })
    const viewModel = new Component().$mount()

    const query = await transport.nextQuery()
    query.reply({
      kind: 'sample',
      sample: {
        key: pump.key,
        payload: encodeCdr(pump.messageSchema, {
          level: 1, max_level: 10, self_test_phase: 0, self_test_active: false,
        }),
        encoding: cdrEncoding(pump.messageSchema),
      },
    })
    await Vue.nextTick()

    expect(viewModel.level).toBe(1)
    const rendersAfterQuery = renderCount

    transport.publish({
      key: pump.key,
      payload: encodeCdr(pump.messageSchema, {
        level: 5, max_level: 10, self_test_phase: 0, self_test_active: false,
      }),
      encoding: cdrEncoding(pump.messageSchema),
    })
    await Vue.nextTick()

    expect(viewModel.level).toBe(5)
    expect(renderCount).toBeGreaterThan(rendersAfterQuery)
    viewModel.$destroy()
  })

  it('closes a subscription that resolves after the component was destroyed', async () => {
    const transport = new FakeTransport()
    const Component = Vue.extend({
      mixins: [blueosApiMixin],
      created() {
        void this.blueosWatchState(transport, pump, () => undefined)
      },
      render(createElement) {
        return createElement('span')
      },
    })
    const viewModel = new Component().$mount()

    expect(transport.subscribers).toHaveLength(1)
    expect(transport.subscribers[0].open).toBe(true)

    viewModel.$destroy()

    const query = await transport.nextQuery()
    query.reply({
      kind: 'sample',
      sample: {
        key: pump.key,
        payload: encodeCdr(pump.messageSchema, {
          level: 1, max_level: 10, self_test_phase: 0, self_test_active: false,
        }),
        encoding: cdrEncoding(pump.messageSchema),
      },
    })
    await Vue.nextTick()

    expect(transport.subscribers[0].open).toBe(false)
  })
})
