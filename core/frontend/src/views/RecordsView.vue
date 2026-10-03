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

    <v-alert
      v-if="recordings.length > 0 && visibleRecordings.length === 0"
      type="info"
      dense
      class="mb-4"
    >
      No recordings match the search and filters.
    </v-alert>

    <v-sheet v-if="recordings.length > 0" rounded class="d-flex align-center flex-wrap mb-4 px-2 pt-2">
      <v-text-field
        v-model="search"
        label="Search by name"
        prepend-inner-icon="mdi-magnify"
        clearable
        dense
        outlined
        hide-details
        class="records-search mr-3 mb-2"
      />
      <v-select
        v-model="stateFilter"
        :items="stateOptions"
        label="State"
        clearable
        dense
        outlined
        hide-details
        class="records-filter mr-3 mb-2"
      />
      <v-select
        v-model="dateFilter"
        :items="dateOptions"
        label="Date (UTC)"
        clearable
        dense
        outlined
        hide-details
        class="records-filter mr-3 mb-2"
      />
      <v-spacer />
      <v-btn
        v-tooltip="'Cards'"
        icon
        small
        class="mb-2"
        :color="layout === 'cards' ? 'primary' : undefined"
        @click="layout = 'cards'"
      >
        <v-icon small>
          mdi-view-grid-outline
        </v-icon>
      </v-btn>
      <v-btn
        v-tooltip="'List'"
        icon
        small
        class="mb-2"
        :color="layout === 'list' ? 'primary' : undefined"
        @click="layout = 'list'"
      >
        <v-icon small>
          mdi-view-list-outline
        </v-icon>
      </v-btn>
    </v-sheet>

    <records-recording-table
      v-if="layout === 'list'"
      :files="visibleRecordings"
      :download-url="downloadUrl"
      :disabled="!recorderServiceRunning"
      :busy-path="busyPath"
      :busy-operation="busyOperation"
      @operation="onOperation"
      @play="openPlayer"
    />

    <v-row v-else>
      <v-col
        v-for="file in visibleRecordings"
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
          @play="openPlayer"
        />
      </v-col>
    </v-row>

    <v-dialog
      v-model="playerOpen"
      max-width="1080"
      scrollable
      @click:outside="closePlayer"
    >
      <v-card v-if="activeRecording">
        <v-card-title class="py-2">
          <span class="text-truncate">{{ activeRecording.name }}</span>
          <v-spacer />
          <v-btn icon @click="closePlayer">
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-card-title>
        <v-divider />
        <v-card-text class="pa-2">
          <mcap-video-player
            v-if="playerOpen"
            :url="downloadUrl(activeRecording.path)"
            :index-source="indexSource(activeRecording.path)"
            :ongoing="activeRecording.state === 'recording'"
            :written-size-bytes="activeRecording.size_bytes"
            @busy="playerBusy = $event"
          />
        </v-card-text>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script lang="ts">
import Vue from 'vue'

import McapVideoPlayer from '@/components/records/McapVideoPlayer.vue'
import RecordsRecordingRow from '@/components/records/RecordsRecordingRow.vue'
import RecordsRecordingTable from '@/components/records/RecordsRecordingTable.vue'
import type { Transport } from '@/libs/blueos-api/transport'
import zenohTransport from '@/libs/blueos-api/zenoh-transport'
import type { RecordingIndexSource } from '@/libs/mcap/logic/recording-index'
import { createRecorderClient, type RecorderClient } from '@/libs/recorder/client'
import {
  CANCEL_JOB,
  DELETE_RECORDING,
  REPAIR_RECORDING,
  SNAPSHOT_RECORDING,
} from '@/libs/recorder/constants'
import { dateFilterOptions, filterRecordings } from '@/libs/recorder/filter'
import type { LibraryRecording, RecordingOperationEvent, RecordingState } from '@/libs/recorder/types'
import { operationFailureMessage, RECORDING_STATE_UI, recordingByPath } from '@/libs/recorder/view-logic'
import zenoh from '@/libs/zenoh'
import { blueosApiMixin } from '@/mixins/blueosApi'

export default Vue.extend({
  name: 'RecordsView',
  components: { RecordsRecordingRow, RecordsRecordingTable, McapVideoPlayer },
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
      layout: 'cards' as 'cards' | 'list',
      search: null as string | null,
      stateFilter: null as RecordingState | null,
      dateFilter: null as string | null,
      playerOpen: false,
      playerBusy: false,
      activeRecordingPath: null as string | null,
    }
  },
  computed: {
    visibleRecordings(): LibraryRecording[] {
      return filterRecordings(this.recordings, {
        search: this.search ?? '',
        state: this.stateFilter,
        date: this.dateFilter,
      })
    },
    stateOptions(): { text: string, value: string }[] {
      return Object.entries(RECORDING_STATE_UI).map(([value, { label }]) => ({ text: label, value }))
    },
    dateOptions(): { text: string, value: string }[] {
      return dateFilterOptions(this.recordings)
    },
    activeRecording(): LibraryRecording | null {
      return recordingByPath(this.recordings, this.activeRecordingPath)
    },
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
    indexSource(path: string): RecordingIndexSource | undefined {
      return this.recorder?.recordingIndexSource(path)
    },
    openPlayer(file: LibraryRecording): void {
      this.activeRecordingPath = file.path
      this.playerOpen = true
    },
    closePlayer(): void {
      if (this.playerBusy) {
        return
      }
      this.playerOpen = false
      this.activeRecordingPath = null
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
        if (operationName === CANCEL_JOB) {
          const result = await this.recorder.cancelRepair(file.repair_job_id)
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

.records-search {
  max-width: 320px;
}

.records-filter {
  max-width: 200px;
}
</style>
