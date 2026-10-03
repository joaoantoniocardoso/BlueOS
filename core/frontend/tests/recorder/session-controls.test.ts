/* eslint-disable import/no-extraneous-dependencies */
import { CommandAckStatus } from '@blueos-idl/constants'
import type { RecordingState } from '@blueos-idl/messages'
import { describe, expect, it } from 'vitest'

import { encodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { NAME, recording } from '@/libs/blueos-api/services/recorder'
import { createRecorderClient } from '@/libs/recorder/client'
import { runSessionAction, sessionControls, sessionNotice } from '@/libs/recorder/session-controls'
import type { RecorderCommandResult } from '@/libs/recorder/types'

import FakeTransport from '../blueos-api/fake-transport'

function session(overrides: Partial<RecordingState> = {}): RecordingState {
  return {
    armed: false,
    session_active: false,
    current_file: '',
    session_bytes_written: 0,
    recording_video_topics: [],
    ...overrides,
  }
}

function result(overrides: Partial<RecorderCommandResult> = {}): RecorderCommandResult {
  return {
    accepted: true, reason: '', job_id: 'job', status: CommandAckStatus.Succeeded, ...overrides,
  }
}

describe('sessionControls', () => {
  it('enables Start and disables Stop while no session is active', () => {
    expect(sessionControls(session(), true, null, false)).toEqual({ canStart: true, canStop: false })
  })

  it('enables Stop and disables Start while a session is active, unless rotating', () => {
    const active = session({ session_active: true })

    expect(sessionControls(active, true, null, false)).toEqual({ canStart: false, canStop: true })
    expect(sessionControls(active, true, null, true)).toEqual({ canStart: true, canStop: true })
  })

  it('disables both until the recording State arrives and while the service is down', () => {
    expect(sessionControls(null, true, null, false)).toEqual({ canStart: false, canStop: false })
    expect(sessionControls(session(), false, null, false)).toEqual({ canStart: false, canStop: false })
  })

  it('disables both while an action is pending', () => {
    expect(sessionControls(session({ session_active: true }), true, 'stop', true))
      .toEqual({ canStart: false, canStop: false })
  })
})

describe('sessionNotice', () => {
  it('reports the final status of an accepted instant Job', () => {
    expect(sessionNotice('start', result())).toEqual({ type: 'success', message: 'Recording started.' })
    expect(sessionNotice('stop', result())).toEqual({ type: 'success', message: 'Recording stopped.' })
  })

  it('reports the reason of a rejected Job', () => {
    expect(sessionNotice('start', result({ accepted: false, reason: 'already recording' })))
      .toEqual({ type: 'warning', message: 'already recording' })
  })

  it('names the status of an accepted Job that did not succeed', () => {
    expect(sessionNotice('stop', result({ status: CommandAckStatus.Aborted, reason: 'disk full' })))
      .toEqual({ type: 'warning', message: 'Stop ended Aborted: disk full' })
  })
})

describe('runSessionAction', () => {
  it('submits Stop and returns the notice from the ack', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)

    const notice = runSessionAction(client, 'stop', false)
    const sent = await transport.nextQuery()
    sent.reply({
      kind: 'sample',
      sample: {
        key: sent.key,
        payload: encodeCdr('blueos_msgs/msg/CommandAck', {
          accepted: true, job_id: 'job', status: CommandAckStatus.Succeeded, reason: '',
        }),
        encoding: cdrEncoding('blueos_msgs/msg/CommandAck'),
      },
    })

    expect(sent.key).toContain('Stop')
    expect(await notice).toEqual({ type: 'success', message: 'Recording stopped.' })
  })

  it('sends the rotate option with Start', async () => {
    const goals: boolean[] = []
    const client = {
      startRecording: async (rotateIfActive: boolean) => {
        goals.push(rotateIfActive)
        return result()
      },
      stopRecording: async () => result(),
    }

    await runSessionAction(client, 'start', true)

    expect(goals).toEqual([true])
  })

  it('turns a transport failure into a warning', async () => {
    const client = {
      startRecording: async () => {
        throw new Error('no reply')
      },
      stopRecording: async () => result(),
    }

    expect(await runSessionAction(client, 'start', false)).toEqual({ type: 'warning', message: 'no reply' })
  })
})

describe('watchRecording', () => {
  it('delivers the recording State, also one already published when it opens', async () => {
    const transport = new FakeTransport()
    const client = createRecorderClient(transport)
    const seen: boolean[] = []

    const watching = client.watchRecording((state) => seen.push(state.session_active))
    const query = await transport.nextQuery()
    expect(query.key).toBe(recording.key)
    query.reply({
      kind: 'sample',
      sample: {
        key: recording.key,
        payload: encodeCdr(recording.messageSchema, session({ session_active: true })),
        encoding: cdrEncoding(recording.messageSchema),
      },
    })
    await watching
    transport.publish({
      key: `blueos/v1/${NAME}/state/recording`,
      payload: encodeCdr(recording.messageSchema, session()),
      encoding: cdrEncoding(recording.messageSchema),
    })

    expect(seen).toEqual([true, false])
  })
})
