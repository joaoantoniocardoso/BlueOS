/* eslint-disable import/no-extraneous-dependencies */
import {
  describe, expect, it, vi,
} from 'vitest'

import { McapCsvExportController } from '@/libs/mcap/adapters/mcap-csv-export-controller'
import type { McapVideoRecording } from '@/libs/mcap/adapters/player'
import type { McapRecordingChannel } from '@/libs/mcap/logic/channels'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'

import { buildJsonTelemetryMcap } from './build-mcap'
import MemoryByteSource from './memory-byte-source'

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

describe('McapCsvExportController when the player is destroyed', () => {
  it('stops a running export and saves nothing', async () => {
    const bytes = await buildJsonTelemetryMcap(200_000, 1024 * 1024)
    const reader = await McapIndexedReader.open(new MemoryByteSource(bytes))
    const { startTime, endTime } = reader.summary
    const callbacks = {
      onState: vi.fn(), onBusy: vi.fn(), onError: vi.fn(), onSaved: vi.fn(),
    }
    const controller = new McapCsvExportController({
      reader,
      startTime,
      durationSeconds: Number(endTime - startTime) / 1e9,
      channels: [...reader.summary.channels.keys()].map((channelId) => channel(channelId, '/depth')),
    } as unknown as McapVideoRecording, null, 'dive', callbacks)
    controller.mount()

    const saving = controller.saveCsv('dive.csv')
    setTimeout(() => controller.destroy(), 0)
    await saving

    expect(callbacks.onSaved).not.toHaveBeenCalled()
    expect(callbacks.onError).not.toHaveBeenCalled()
    expect(controller.getState().exportProgress).toBeNull()
  }, 60_000)
})
