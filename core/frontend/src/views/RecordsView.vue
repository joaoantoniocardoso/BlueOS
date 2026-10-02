<template>
  <v-container fluid class="records-view">
    <v-alert
      v-if="!recorderServiceRunning"
      type="error"
      dense
      class="mb-4"
    >
      Recorder service is not running.
    </v-alert>

    <v-alert
      v-else-if="!libraryLoading && recordings.length === 0"
      type="info"
      dense
      class="mb-4"
    >
      No recordings found yet.
    </v-alert>

    <v-alert
      v-if="lastError"
      type="error"
      dense
      class="mb-4"
    >
      {{ lastError }}
    </v-alert>

    <v-row>
      <v-col
        v-for="file in recordings"
        :key="file.path"
        cols="12"
        sm="6"
        md="4"
        lg="3"
      >
        <records-recording-row
          :file="file"
          :download-url="downloadUrl(file.path)"
          :disabled="!recorderServiceRunning"
          :busy-operation="busyPath === file.path ? busyOperation : null"
          @operation="onOperation"
        />
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import Vue from 'vue'

import RecordsRecordingRow from '@/components/records/RecordsRecordingRow.vue'
import type { Transport } from '@/libs/blueos-api/transport'
import zenohTransport from '@/libs/blueos-api/zenoh-transport'
import { createRecorderClient, type RecorderClient } from '@/libs/recorder/client'
import {
  CANCEL_REPAIR,
  DELETE_RECORDING,
  REPAIR_RECORDING,
  SNAPSHOT_RECORDING,
} from '@/libs/recorder/constants'
import type { LibraryRecording, RecordingOperationEvent } from '@/libs/recorder/types'
import { operationFailureMessage } from '@/libs/recorder/view-logic'
import zenoh from '@/libs/zenoh'
import { blueosApiMixin } from '@/mixins/blueosApi'

export default Vue.extend({
  name: 'RecordsView',
  components: { RecordsRecordingRow },
  mixins: [blueosApiMixin],
  data() {
    return {
      transport: null as Transport | null,
      recorder: null as RecorderClient | null,
      recordings: [] as LibraryRecording[],
      libraryLoading: true,
      recorderServiceRunning: false,
      lastError: '' as string,
      busyPath: null as string | null,
      busyOperation: null as string | null,
    }
  },
  async created() {
    const session = await zenoh.getSession()
    this.transport = zenohTransport(session)
    this.recorder = createRecorderClient(this.transport)
    await Promise.all([
      this.blueosTrackSubscription(this.recorder.watchLibrary(
        (files) => {
          this.recordings = files
          this.libraryLoading = false
        },
        (error) => {
          this.lastError = error instanceof Error ? error.message : String(error)
        },
      )),
      this.blueosTrackSubscription(this.recorder.watchServiceRunning((running) => {
        this.recorderServiceRunning = running
        if (!running) {
          this.libraryLoading = false
          this.recordings = []
        }
      })),
      this.blueosTrackSubscription(this.recorder.watchOperations(
        (event) => this.onRecordingOperation(event),
        (error) => {
          this.lastError = error instanceof Error ? error.message : String(error)
        },
      )),
    ])
  },
  methods: {
    downloadUrl(path: string): string {
      return this.recorder?.recordingDownloadUrl(path) ?? ''
    },
    onRecordingOperation(event: RecordingOperationEvent): void {
      const failure = operationFailureMessage(event, event.path)
      if (failure) {
        this.lastError = failure
      }
    },
    async onOperation(operationName: string, file: LibraryRecording): Promise<void> {
      if (!this.recorder) {
        return
      }
      this.lastError = ''
      this.busyPath = file.path
      this.busyOperation = operationName
      try {
        if (operationName === REPAIR_RECORDING) {
          const result = await this.recorder.repairRecording(file.path)
          if (!result.accepted) {
            this.lastError = result.reason
          }
          return
        }
        if (operationName === CANCEL_REPAIR) {
          const result = await this.recorder.cancelRepair(file.path)
          if (!result.accepted) {
            this.lastError = result.reason
          }
          return
        }
        if (operationName === DELETE_RECORDING) {
          const result = await this.recorder.deleteRecording(file.path)
          if (!result.accepted) {
            this.lastError = result.reason
          }
          return
        }
        if (operationName === SNAPSHOT_RECORDING) {
          const outputPath = await this.recorder.snapshotRecording(file.path)
          this.triggerDownload(
            this.recorder.recordingDownloadUrl(outputPath),
            outputPath.split('/').pop() ?? file.name,
          )
        }
      } catch (error) {
        this.lastError = error instanceof Error ? error.message : String(error)
      } finally {
        this.busyPath = null
        this.busyOperation = null
      }
    },
    triggerDownload(url: string, fileName: string): void {
      const link = document.createElement('a')
      link.href = url
      link.download = fileName
      link.click()
    },
  },
})
</script>

<style scoped>
.records-view {
  min-height: 100%;
}
</style>
