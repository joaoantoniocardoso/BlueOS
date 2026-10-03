/* eslint-disable import/no-extraneous-dependencies */
import {
  describe, expect, it, vi,
} from 'vitest'

import { McapCsvExportController } from '@/libs/mcap/adapters/mcap-csv-export-controller'
import type { McapVideoRecording } from '@/libs/mcap/adapters/player'
import type { McapRecordingChannel } from '@/libs/mcap/logic/channels'

function channel(channelId: number, topic: string): McapRecordingChannel {
  return {
    channelId, topic, schemaName: 'blueos_msgs/msg/Sample', messageEncoding: 'cdr', messageCount: 10,
  }
}

function mountedController(): { controller: McapCsvExportController, onState: ReturnType<typeof vi.fn> } {
  const channels = [channel(1, '/depth'), channel(2, '/heading')]
  const onState = vi.fn()
  const controller = new McapCsvExportController({ channels } as unknown as McapVideoRecording, null, 'dive', {
    onState, onBusy: vi.fn(), onError: vi.fn(), onSaved: vi.fn(),
  })
  controller.mount()
  onState.mockClear()
  return { controller, onState }
}

describe('McapCsvExportController', () => {
  it('does not report a selection or a search that did not change, so an echo cannot loop', () => {
    const { controller, onState } = mountedController()

    controller.setSelected([channel(1, '/depth'), channel(2, '/heading')])
    controller.setSearch('')

    expect(onState).not.toHaveBeenCalled()
  })

  it('reports a changed selection once', () => {
    const { controller, onState } = mountedController()

    controller.setSelected([channel(2, '/heading')])

    expect(onState).toHaveBeenCalledTimes(1)
    expect(onState.mock.calls[0][0].selected.map((entry: McapRecordingChannel) => entry.channelId)).toEqual([2])
  })
})
