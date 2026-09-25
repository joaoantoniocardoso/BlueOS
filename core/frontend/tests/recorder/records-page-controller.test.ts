import {
  afterEach, beforeEach, describe, expect, it, vi,
} from 'vitest'

import type { LibraryRecording, RecorderClient } from '@/libs/recorder'
import { RecordsPageController, SNAPSHOT_WAIT_TIMEOUT_MS } from '@/libs/recorder'

function file(overrides: Partial<LibraryRecording> = {}): LibraryRecording {
  return {
    path: 'live.mcap',
    name: 'live.mcap',
    size_bytes: 1000,
    created: 1_700_000_000,
    state: 'recording',
    repair_bytes_processed: 0,
    repair_total_bytes: 0,
    repair_bytes_per_second: 0,
    repair_error: '',
    ...overrides,
  }
}

function stubRecorder(): RecorderClient {
  return {
    watchLibrary: () => () => undefined,
    watchVehicleArmed: () => () => undefined,
    watchOperations: () => () => undefined,
    repairRecording: async () => ({ accepted: true, reason: '' }),
    cancelRepair: async () => ({ accepted: true, reason: '' }),
    deleteRecording: async () => ({ accepted: true, reason: '' }),
    snapshotRecording: async () => ({ accepted: true, reason: '' }),
    indexSource: () => ({
      readIndexPage: async () => ({ chunks: [], next_offset: 0 }),
    }),
    recordingUrl: (path) => `/userdata/recorder/${path}`,
  }
}

describe('RecordsPageController', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('clears library loading when the recorder service is not running', () => {
    const controller = new RecordsPageController(stubRecorder(), {
      onState: () => undefined,
      onNotifyError: () => undefined,
      onTriggerDownload: () => undefined,
    }, 'cards')

    expect(controller.getState().libraryLoading).toBe(true)
    controller.setRecorderServiceRunning(false)
    const state = controller.getState()
    expect(state.libraryLoading).toBe(false)
    expect(state.recorderServiceRunning).toBe(false)
  })

  it('resolves snapshot wait from library state after reconnect', async () => {
    const onNotifyError = vi.fn()
    const controller = new RecordsPageController(stubRecorder(), {
      onState: () => undefined,
      onNotifyError,
      onTriggerDownload: () => undefined,
    }, 'cards')

    const wait = (controller as unknown as {
      waitForSnapshot: (path: string) => Promise<string>
    }).waitForSnapshot('live.mcap')

    controller.setLibrary([
      file(),
      file({
        path: 'live.snapshot-2024-01-02T03-04-05Z.mcap',
        name: 'live.snapshot-2024-01-02T03-04-05Z.mcap',
        state: 'ready',
      }),
    ])

    await expect(wait).resolves.toBe('live.snapshot-2024-01-02T03-04-05Z.mcap')
    expect(onNotifyError).not.toHaveBeenCalled()
  })

  it('rejects snapshot wait on timeout', async () => {
    const controller = new RecordsPageController(stubRecorder(), {
      onState: () => undefined,
      onNotifyError: () => undefined,
      onTriggerDownload: () => undefined,
    }, 'cards')

    const wait = (controller as unknown as {
      waitForSnapshot: (path: string) => Promise<string>
    }).waitForSnapshot('live.mcap')

    vi.advanceTimersByTime(SNAPSHOT_WAIT_TIMEOUT_MS + 1)
    await expect(wait).rejects.toThrow(/Timed out waiting for the snapshot file/)
  })
})
