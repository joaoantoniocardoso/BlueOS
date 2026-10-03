/* eslint-disable import/no-extraneous-dependencies */
import { CommandAckStatus } from '@blueos-idl/constants'
import type { RecordingState } from '@blueos-idl/messages'

import type { RecorderClient } from './client'
import type { RecorderCommandResult } from './types'

export type SessionAction = 'start' | 'stop'

export interface SessionControls {
  canStart: boolean
  canStop: boolean
}

export interface SessionNotice {
  type: 'success' | 'warning'
  message: string
}

const ACTION_LABEL: Record<SessionAction, { done: string, name: string }> = {
  start: { done: 'Recording started.', name: 'Start' },
  stop: { done: 'Recording stopped.', name: 'Stop' },
}

/**
 * What the record button and the Stop button offer. Both wait for the recording State, so a Stop is offered only while
 * a session is active, and a Start only while none is, or when it rotates the file; `pending` blocks a second
 * submission until the ack arrives.
 */
export function sessionControls(
  recording: RecordingState | null,
  serviceRunning: boolean,
  pending: SessionAction | null,
  rotateIfActive: boolean,
): SessionControls {
  if (recording === null || !serviceRunning || pending !== null) {
    return { canStart: false, canStop: false }
  }
  return {
    canStart: !recording.session_active || rotateIfActive,
    canStop: recording.session_active,
  }
}

/** Shows the final status Start and Stop carry in their ack (they are instant Job types), or why they were refused. */
export function sessionNotice(action: SessionAction, result: RecorderCommandResult): SessionNotice {
  if (!result.accepted) {
    return { type: 'warning', message: result.reason || `${ACTION_LABEL[action].name} was rejected.` }
  }
  if (result.status === CommandAckStatus.Succeeded) {
    return { type: 'success', message: ACTION_LABEL[action].done }
  }
  const status = Object.entries(CommandAckStatus).find(([, value]) => value === result.status)?.[0] ?? 'Unknown'
  const reason = result.reason ? `: ${result.reason}` : ''
  return { type: 'warning', message: `${ACTION_LABEL[action].name} ended ${status}${reason}` }
}

/** Submits Start or Stop as a Job and describes its ack; a failure to reach the Service is a warning notice. */
export async function runSessionAction(
  client: Pick<RecorderClient, 'startRecording' | 'stopRecording'>,
  action: SessionAction,
  rotateIfActive: boolean,
): Promise<SessionNotice> {
  try {
    const result = action === 'start'
      ? await client.startRecording(rotateIfActive)
      : await client.stopRecording()
    return sessionNotice(action, result)
  } catch (error) {
    return { type: 'warning', message: error instanceof Error ? error.message : String(error) }
  }
}
