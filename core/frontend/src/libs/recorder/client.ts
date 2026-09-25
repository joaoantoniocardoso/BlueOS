import { sendCommand, type Unsubscribe, watchState } from '@/libs/blueos-api'
import type { RecordingIndexSource } from '@/libs/mcap'

import {
  CANCEL_REPAIR_COMMAND_SCHEMA,
  DEFAULT_RECORDING_HTTP_PREFIX,
  DELETE_RECORDING_COMMAND_SCHEMA,
  RECORDER_SERVICE,
  RECORDING_LIBRARY_SCHEMA,
  RECORDING_STATE_SCHEMA,
  REPAIR_RECORDING_COMMAND_SCHEMA,
  SNAPSHOT_RECORDING_COMMAND_SCHEMA,
} from './constants'
import { createRecordingIndexSource } from './index-source'
import { mapRecordingFile } from './map'
import type {
  LibraryRecording,
  RecorderCommandResult,
  RecordingOperationEvent,
} from './types'
import { recordingUrl } from './url'
import { sortRecordingsNewestFirst } from './view-logic'
import { watchRecordingOperations } from './watch-operation'

export interface RecorderClient {
  watchLibrary(onLibrary: (files: LibraryRecording[]) => void): Unsubscribe
  watchVehicleArmed(onArmed: (armed: boolean) => void): Unsubscribe
  watchOperations(onOperation: (event: RecordingOperationEvent) => void): Unsubscribe
  repairRecording(path: string): Promise<RecorderCommandResult>
  cancelRepair(path: string): Promise<RecorderCommandResult>
  deleteRecording(path: string): Promise<RecorderCommandResult>
  snapshotRecording(path: string): Promise<RecorderCommandResult>
  indexSource(path: string): RecordingIndexSource
  recordingUrl(path: string): string
}

export interface RecorderClientOptions {
  recordingHttpPrefix?: string
}

function commandResult(ack: { accepted: boolean, reason: string }): RecorderCommandResult {
  return { accepted: ack.accepted, reason: ack.reason }
}

export function createRecorderClient(options: RecorderClientOptions = {}): RecorderClient {
  const httpPrefix = options.recordingHttpPrefix ?? DEFAULT_RECORDING_HTTP_PREFIX

  return {
    watchLibrary(onLibrary) {
      return watchState(RECORDER_SERVICE, 'library', RECORDING_LIBRARY_SCHEMA, (library) => {
        onLibrary(sortRecordingsNewestFirst(library.files.map(mapRecordingFile)))
      })
    },

    watchVehicleArmed(onArmed) {
      return watchState(RECORDER_SERVICE, 'recording', RECORDING_STATE_SCHEMA, (state) => {
        onArmed(state.armed)
      })
    },

    watchOperations(onOperation) {
      return watchRecordingOperations(onOperation)
    },

    async repairRecording(path) {
      const ack = await sendCommand(
        RECORDER_SERVICE,
        'RepairRecording',
        REPAIR_RECORDING_COMMAND_SCHEMA,
        { path },
      )
      return commandResult(ack)
    },

    async cancelRepair(path) {
      const ack = await sendCommand(
        RECORDER_SERVICE,
        'CancelRepair',
        CANCEL_REPAIR_COMMAND_SCHEMA,
        { path },
      )
      return commandResult(ack)
    },

    async deleteRecording(path) {
      const ack = await sendCommand(
        RECORDER_SERVICE,
        'DeleteRecording',
        DELETE_RECORDING_COMMAND_SCHEMA,
        { path },
      )
      return commandResult(ack)
    },

    async snapshotRecording(path) {
      const ack = await sendCommand(
        RECORDER_SERVICE,
        'SnapshotRecording',
        SNAPSHOT_RECORDING_COMMAND_SCHEMA,
        { path },
      )
      return commandResult(ack)
    },

    indexSource(path) {
      return createRecordingIndexSource(path)
    },

    recordingUrl(path) {
      return recordingUrl(path, httpPrefix)
    },
  }
}
