import type { CommandAck, JobStatus, RecordingState as RecordingSessionState } from '@blueos-idl/messages'

import { cancelJob, sendCommand } from '@/libs/blueos-api/command'
import { jobsState, metricsState } from '@/libs/blueos-api/endpoints'
import { watchJobFeedback, watchJobResults } from '@/libs/blueos-api/job'
import { watchServiceAlive } from '@/libs/blueos-api/liveliness'
import {
  DeleteRecording,
  library,
  NAME,
  recording,
  RepairRecording,
  SnapshotRecording,
  Start,
  Stop,
} from '@/libs/blueos-api/services/recorder'
import type { Subscription, Transport } from '@/libs/blueos-api/transport'
import { watchState } from '@/libs/blueos-api/watch'
import type { ByteSource } from '@/libs/mcap/logic/byte-source'
import type { RecordingIndexSource } from '@/libs/mcap/logic/recording-index'

import { ZenohByteSource } from './byte-source'
import { DEFAULT_RECORDING_HTTP_PREFIX, SNAPSHOT_WAIT_TIMEOUT_MS } from './constants'
import { createCachedRecordingIndexSource } from './index-source'
import { mapRecordingFile } from './map'
import type {
  LibraryRecording,
  RecorderCommandResult,
  RecordingJobResult,
} from './types'
import { recordingDownloadUrl } from './url'
import {
  readySnapshotDownloadPath,
  type RecorderMetrics,
  recorderMetrics,
  recordingDownload,
  type RepairProgress,
  snapshotDownloadPath,
  snapshotPathsForSource,
} from './view-logic'

export { SNAPSHOT_WAIT_TIMEOUT_MS } from './constants'

/** The recorder refused the snapshot, for instance because the recording stopped meanwhile. */
class SnapshotRefusedError extends Error {}

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
    onOperation: (entry: RecordingJobResult) => void,
    onError?: (error: unknown) => void,
  ): Promise<Subscription>
  watchRecording(
    onRecording: (state: RecordingSessionState) => void,
    onError?: (error: unknown) => void,
  ): Promise<Subscription>
  watchRepairProgress(
    onProgress: (progress: RepairProgress) => void,
    onError?: (error: unknown) => void,
  ): Promise<Subscription>
  watchJobs(
    onJobs: (jobs: JobStatus[]) => void,
    onError?: (error: unknown) => void,
  ): Promise<Subscription>
  watchMetrics(
    onMetrics: (metrics: RecorderMetrics) => void,
    onError?: (error: unknown) => void,
  ): Promise<Subscription>
  startRecording(rotateIfActive: boolean): Promise<RecorderCommandResult>
  stopRecording(): Promise<RecorderCommandResult>
  repairRecording(path: string): Promise<RecorderCommandResult>
  cancelRepair(repairJobId: string): Promise<RecorderCommandResult>
  deleteRecording(path: string): Promise<RecorderCommandResult>
  snapshotRecording(path: string): Promise<string>
  /** The path to download for `file`: its own, or for the file being written, a snapshot taken now. */
  recordingDownloadPath(file: LibraryRecording): Promise<string>
  recordingDownloadUrl(relativePath: string): string
  recordingIndexSource(path: string): RecordingIndexSource
  /** The bytes of the recording at `path`, read with the `bytes` Query. */
  recordingByteSource(path: string): ByteSource
}

export interface RecorderClientOptions {
  recordingHttpPrefix?: string
}

function commandResult(commandAck: CommandAck): RecorderCommandResult {
  return {
    accepted: commandAck.accepted, reason: commandAck.reason, job_id: commandAck.job_id, status: commandAck.status,
  }
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

  function resolveSnapshotFromOperation(entry: RecordingJobResult): void {
    const waiter = snapshotWaiters[entry.result.path]
    if (!waiter || entry.job.job_type !== SnapshotRecording.name) {
      return
    }
    delete snapshotWaiters[entry.result.path]
    clearTimeout(waiter.timeoutId)
    const outputPath = snapshotDownloadPath(entry)
    if (outputPath) {
      waiter.resolve(outputPath)
      return
    }
    waiter.reject(new Error(entry.job.reason || 'Snapshot failed'))
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
          const contents = new Map(message.contents.map((entry) => [entry.path, entry]))
          librarySnapshot = message.files.map((file) => mapRecordingFile(file, contents.get(file.path)))
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
      const subscriptions = await Promise.all(
        [DeleteRecording, RepairRecording, SnapshotRecording].map((operation) => watchJobResults(
          transport,
          NAME,
          operation.name,
          operation.resultSchema,
          {
            onValue: (entry: RecordingJobResult) => {
              onOperation(entry)
              resolveSnapshotFromOperation(entry)
            },
            onError: (error) => onError?.(error),
          },
        )),
      )
      return {
        close: async () => {
          await Promise.all(subscriptions.map((subscription) => subscription.close()))
        },
      }
    },

    watchRecording(onRecording, onError) {
      return watchState(transport, recording, {
        onValue: (state) => onRecording(state),
        onError: (error) => onError?.(error),
      })
    },

    watchRepairProgress(onProgress, onError) {
      return watchJobFeedback(transport, NAME, RepairRecording.name, RepairRecording.feedbackSchema, {
        onValue: (entries) => onProgress(Object.fromEntries(entries.map(({ jobId, feedback }) => [jobId, feedback]))),
        onError: (error) => onError?.(error),
      })
    },

    watchJobs(onJobs, onError) {
      return watchState(transport, jobsState(NAME), {
        onValue: (list) => onJobs(list.jobs),
        onError: (error) => onError?.(error),
      })
    },

    watchMetrics(onMetrics, onError) {
      return watchState(transport, metricsState(NAME), {
        onValue: (message) => onMetrics(recorderMetrics(message)),
        onError: (error) => onError?.(error),
      })
    },

    async startRecording(rotateIfActive) {
      return commandResult(await sendCommand(transport, Start, { rotate_if_active: rotateIfActive }))
    },

    async stopRecording() {
      return commandResult(await sendCommand(transport, Stop, {}))
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
          removeSnapshotWaiter(path, new SnapshotRefusedError(commandAck.reason || 'Snapshot command rejected'))
        }
      } catch (error) {
        removeSnapshotWaiter(
          path,
          error instanceof Error ? error : new Error(String(error)),
        )
      }
      return snapshotPromise
    },

    async recordingDownloadPath(file) {
      if (recordingDownload(file) !== 'snapshot') {
        return file.path
      }
      try {
        return await this.snapshotRecording(file.path)
      } catch (error) {
        if (error instanceof SnapshotRefusedError) {
          return file.path
        }
        throw error
      }
    },

    recordingDownloadUrl(relativePath) {
      return recordingDownloadUrl(relativePath, httpPrefix)
    },

    recordingIndexSource(path) {
      return createCachedRecordingIndexSource(transport, path, () => {
        const file = librarySnapshot.find((libraryFile) => libraryFile.path === path)
        return file?.size_bytes ?? 0
      })
    },
    recordingByteSource(path) {
      return new ZenohByteSource(transport, path)
    },
  }
}
