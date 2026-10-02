import { describe, expect, it } from 'vitest'

import { serviceLivelinessKey } from '@/libs/blueos-api/keys'
import { watchServiceAlive } from '@/libs/blueos-api/liveliness'

import FakeTransport from './fake-transport'

describe('watchServiceAlive', () => {
  it('reports alive when liveliness is put and false when deleted', async () => {
    const transport = new FakeTransport()
    const alive: boolean[] = []
    const watching = watchServiceAlive(transport, 'recorder', {
      onAlive: (value) => alive.push(value),
    })
    await watching

    const key = serviceLivelinessKey('recorder')
    transport.publishLiveliness(key, true)
    transport.publishLiveliness(key, false)

    expect(alive).toEqual([true, false])
  })
})
