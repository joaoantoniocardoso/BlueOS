/* eslint-disable import/no-extraneous-dependencies */
import { CommandAckStatus } from '@blueos-idl/constants'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { sendCommand } from '@/libs/blueos-api/command'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { pump, SetLevel } from '@/libs/blueos-api/services/example'
import { watchState } from '@/libs/blueos-api/watch'

import FakeTransport from '../blueos-api/fake-transport'

const idlePump = {
  level: 0,
  max_level: 100,
  self_test_phase: 0,
  self_test_active: false,
}

/**
 * Exercises the same generated client and fake transport as `ExampleMinimalPanel.vue`.
 * Manual (human): mount the Vue panel against a live `blueos example` Service in the browser.
 */
describe('example-minimal frontend', () => {
  it('sends SetLevel and receives pump State updates', async () => {
    const transport = new FakeTransport()
    const levels: number[] = []
    const watching = watchState(transport, pump, {
      onValue: (message) => levels.push(message.level),
      onError: () => undefined,
    })

    const stateQuery = await transport.nextQuery()
    stateQuery.reply({
      kind: 'sample',
      sample: {
        key: pump.key,
        payload: encodeCdr(pump.messageSchema, idlePump),
        encoding: cdrEncoding(pump.messageSchema),
      },
    })
    await watching

    const sending = sendCommand(transport, SetLevel, { level: 42 })
    const commandQuery = await transport.nextQuery()
    expect(commandQuery.key).toBe(SetLevel.key)
    commandQuery.reply({
      kind: 'sample',
      sample: {
        key: SetLevel.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true,
          job_id: new TextDecoder().decode(commandQuery.body?.attachment),
          status: CommandAckStatus.Succeeded,
          reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })
    expect(await sending).toMatchObject({ accepted: true })

    transport.publish({
      key: pump.key,
      payload: encodeCdr(pump.messageSchema, { ...idlePump, level: 42 }),
      encoding: cdrEncoding(pump.messageSchema),
    })

    expect(levels).toEqual([0, 42])
  })
})
