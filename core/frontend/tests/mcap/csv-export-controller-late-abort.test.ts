/* eslint-disable import/no-extraneous-dependencies */
import {
  describe, expect, it, vi,
} from 'vitest'

import { McapCsvExportController } from '@/libs/mcap/adapters/mcap-csv-export-controller'
import type { McapVideoRecording } from '@/libs/mcap/adapters/player'

const exported = vi.hoisted(() => ({ finish: null as (() => void) | null }))

vi.mock('@/libs/mcap/logic/csv', () => ({
  exportChannelsAsCsv: () => new Promise<Blob>((resolve) => {
    exported.finish = () => resolve(new Blob(['depth\n1\n']))
  }),
}))

describe('McapCsvExportController when the export is cancelled as it finishes', () => {
  it('saves nothing, since the export no longer wanted its file', async () => {
    const callbacks = {
      onState: vi.fn(), onBusy: vi.fn(), onError: vi.fn(), onSaved: vi.fn(),
    }
    const channels = [{
      channelId: 1, topic: '/depth', schemaName: 'x', messageEncoding: 'cdr', messageCount: 1,
    }]
    const recording = { channels } as unknown as McapVideoRecording
    const controller = new McapCsvExportController(recording, null, 'dive', callbacks)
    controller.mount()
    const saving = controller.saveCsv('dive.csv')

    controller.destroy()
    exported.finish?.()
    await saving

    expect(callbacks.onSaved).not.toHaveBeenCalled()
  })
})
