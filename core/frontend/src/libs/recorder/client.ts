import { cancelJob, sendCommand } from '@/libs/blueos-api/command'
import { watchJobResults } from '@/libs/blueos-api/job'
import { watchServiceAlive } from '@/libs/blueos-api/liveliness'
import {
  DeleteRecording,
  NAME,
  RepairRecording,
  SnapshotRecording,
  library,
} from '@/libs/blueos-api/services/recorder'
import type { Subscription, Transport } from '@/libs/blueos-api/transport'
import { watchState } from '@/libs/blueos-api/watch'

import type { RecordingIndexSource } from '@/libs/mcap/logic/recording-index'

import { DEFAULT_RECORDING_HTTP_PREFIX, SNAPSHOT_WAIT_TIMEOUT_MS } from './constants'
import { createCachedRecordingIndexSource } from './index-source'
import { mapRecordingFile, mapRecordingOperation } from './map'
import type {
  LibraryRecording,
  RecorderCommandResult,
  RecordingOperationEvent,
} from './types'
import { recordingDownloadUrl } from './url'
import {
  isSnapshotOperationForPath,
  readySnapshotDownloadPath,
  snapshotDownloadPath,
  snapshotPathsForSource,
  sortRecordingsNewestFirst,
} from './view-logic'

export { SNAPSHOT_WAIT_TIMEOUT_MS } from './constants'

interface SnapshotWaiter {
  resolve: (outputPath: string) => void
  reject: (error: Error) => void
  timeoutId: ReturnType<typeof setTimeout>
  ignoredSnapshotPaths: Set<string>
}

export interface RecorderClient {
  watchLibrary(
    onLibrary: (files: LibraryRecording[]) => void,
    onError?: (error: unknown) => void,
  ): Promise<Subscription>
  watchServiceRunning(onRunning: (running: boolean) => void): Promise<Subscription>
  watchOperations(
    onOperation: (event: RecordingOperationEvent) => void,
    onError?: (error: unknown) => void,
  ): Promise<Subscription>
  repairRecording(path: string): Promise<RecorderCommandResult>
  cancelRepair(repairJobId: string): Promise<RecorderCommandResult>
  deleteRecording(path: string): Promise<RecorderCommandResult>
  snapshotRecording(path: string): Promise<string>
  recordingDownloadUrl(relativePath: string): string
  recordingIndexSource(path: string): RecordingIndexSource
}

export interface RecorderClientOptions {
  recordingHttpPrefix?: string
}

function commandResult(commandAck: { accepted: boolean, reason: string }): RecorderCommandResult {
  return { accepted: commandAck.accepted, reason: commandAck.reason }
}

export function createRecorderClient(
  transport: Transport,
  options: RecorderClientOptions = {},
): RecorderClient {
  const httpPrefix = options.recordingHttpPrefix ?? DEFAULT_RECORDING_HTTP_PREFIX
  let librarySnapshot: LibraryRecording[] = []
  const snapshotWaiters: Record<string, SnapshotWaiter> = {}

  function removeSnapshotWaiter(sourcePath: string, rejectWith?: Error): void {
    const waiter = snapshotWaiters[sourcePath]
    if (!waiter) {
      return
    }
    delete snapshotWaiters[sourcePath]
    clearTimeout(waiter.timeoutId)
    if (rejectWith) {
      waiter.reject(rejectWith)
    }
  }

  function resolveSnapshotWaiters(): void {
    for (const [path, waiter] of Object.entries(snapshotWaiters)) {
      const outputPath = readySnapshotDownloadPath(path, librarySnapshot, waiter.ignoredSnapshotPaths)
      if (!outputPath) {
        continue
      }
      delete snapshotWaiters[path]
      clearTimeout(waiter.timeoutId)
      waiter.resolve(outputPath)
    }
  }

  function resolveSnapshotFromOperation(event: RecordingOperationEvent): void {
    const waiter = snapshotWaiters[event.path]
    if (!waiter || !isSnapshotOperationForPath(event, event.path)) {
      return
    }
    delete snapshotWaiters[event.path]
    clearTimeout(waiter.timeoutId)
    const outputPath = snapshotDownloadPath(event)
    if (outputPath) {
      waiter.resolve(outputPath)
      return
    }
    waiter.reject(new Error(event.error || 'Snapshot failed'))
  }

  function beginSnapshotWait(sourcePath: string): Promise<string> {
    const ignoredSnapshotPaths = new Set(snapshotPathsForSource(sourcePath, librarySnapshot))
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        removeSnapshotWaiter(sourcePath, new Error('Timed out waiting for the snapshot file'))
      }, SNAPSHOT_WAIT_TIMEOUT_MS)
      snapshotWaiters[sourcePath] = {
        resolve, reject, timeoutId, ignoredSnapshotPaths,
      }
    })
  }

  return {
    watchLibrary(onLibrary, onError) {
      return watchState(transport, library, {
        onValue: (message) => {
          librarySnapshot = sortRecordingsNewestFirst(message.files.map(mapRecordingFile))
          onLibrary(librarySnapshot)
          resolveSnapshotWaiters()
        },
        onError: (error) => onError?.(error),
      })
    },

    watchServiceRunning(onRunning) {
      return watchServiceAlive(transport, 'recorder', { onAlive: onRunning })
    },

    async watchOperations(onOperation, onError) {
      const report = (event: RecordingOperationEvent): void => {
        onOperation(event)
        resolveSnapshotFromOperation(event)
      }
      const subscriptions = await Promise.all([
        watchJobResults(transport, NAME, RepairRecording.name, RepairRecording.resultSchema, {
          onValue: (entry) => report(mapRecordingOperation('repair', entry)),
          onError: (error) => onError?.(error),
        }),
        watchJobResults(transport, NAME, SnapshotRecording.name, SnapshotRecording.resultSchema, {
          onValue: (entry) => report(mapRecordingOperation('snapshot', entry)),
          onError: (error) => onError?.(error),
        }),
      ])
      return {
        close: async () => {
          await Promise.all(subscriptions.map((subscription) => subscription.close()))
        },
      }
    },

    async repairRecording(path) {
      const commandAck = await sendCommand(transport, RepairRecording, { path })
      return commandResult(commandAck)
    },

    async cancelRepair(repairJobId) {
      const commandAck = await cancelJob(transport, NAME, repairJobId)
      return commandResult(commandAck)
    },

    async deleteRecording(path) {
      const commandAck = await sendCommand(transport, DeleteRecording, { path })
      return commandResult(commandAck)
    },

    async snapshotRecording(path) {
      const snapshotPromise = beginSnapshotWait(path)
      try {
        const commandAck = await sendCommand(transport, SnapshotRecording, { path })
        if (!commandAck.accepted) {
          removeSnapshotWaiter(path, new Error(commandAck.reason || 'Snapshot command rejected'))
        }
      } catch (error) {
        removeSnapshotWaiter(
          path,
          error instanceof Error ? error : new Error(String(error)),
        )
      }
      return snapshotPromise
    },

    recordingDownloadUrl(relativePath) {
      return recordingDownloadUrl(relativePath, httpPrefix)
    },

    recordingIndexSource(path) {
      return createCachedRecordingIndexSource(transport, path, () => {
        const file = librarySnapshot.find((recording) => recording.path === path)
        return file?.size_bytes ?? 0
      })
    },
  }
}
