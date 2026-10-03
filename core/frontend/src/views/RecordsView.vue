<template>
  <v-container fluid class="records-view">
    <records-session-controls
      :recorder="recorder"
      :recording="recording"
      :service-running="recorderServiceRunning"
    />

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
        label="Date"
        clearable
        dense
        outlined
        hide-details
        class="records-filter mr-3 mb-2"
      />
      <v-checkbox
        v-if="visibleRecordings.length > 0"
        :input-value="allVisibleSelected"
        :indeterminate="someVisibleSelected && !allVisibleSelected"
        dense
        hide-details
        class="mt-0 pt-0 mr-3 mb-2"
        label="Select all"
        @change="toggleSelectAllVisible"
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

    <v-sheet
      v-if="selectedFiles.length > 0"
      rounded
      class="d-flex align-center flex-wrap mb-4 pa-3"
    >
      <div class="mr-4 mb-2 subtitle-2">
        {{ selectedFiles.length }} selected
      </div>
      <v-spacer />
      <v-btn
        v-tooltip="canRepairSelected
          ? 'Rewrite selected recordings so they can be read'
          : 'Nothing selected needs repair'"
        small
        outlined
        color="primary"
        class="mr-2 mb-2"
        :disabled="!canRepairSelected || bulkBusy"
        :loading="bulkRepairing"
        @click="askRepair(selectedFiles)"
      >
        <v-icon small left>
          mdi-wrench
        </v-icon>
        Repair
      </v-btn>
      <v-btn
        v-tooltip="canDeleteSelected ? 'Delete the selected recordings' : 'Nothing selected can be deleted'"
        small
        outlined
        color="error"
        class="mr-2 mb-2"
        :disabled="!canDeleteSelected || bulkBusy"
        :loading="bulkDeleting"
        @click="askDelete(selectedFiles)"
      >
        <v-icon small left>
          mdi-delete
        </v-icon>
        Delete
      </v-btn>
      <v-btn
        small
        text
        class="mb-2"
        @click="clearSelection"
      >
        Clear
      </v-btn>
    </v-sheet>

    <records-recording-table
      v-if="layout === 'list'"
      :files="visibleRecordings"
      :selected-files.sync="selectedTableFiles"
      :download-url="downloadUrl"
      :disabled="!recorderServiceRunning"
      :busy-path="busyPath"
      :busy-operation="busyOperation"
      @operation="onOperation"
      @play="openPlayer"
    />

    <v-data-iterator
      v-else-if="visibleRecordings.length > 0"
      :items="visibleRecordings"
      item-key="path"
      sort-by="created"
      sort-desc
      :items-per-page="24"
      :footer-props="{ 'items-per-page-options': [12, 24, 48, 96] }"
    >
      <template #default="{ items }">
        <v-row>
          <v-col
            v-for="file in items"
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
              selectable
              :selected="selectedPaths.includes(file.path)"
              @toggle-select="toggleSelected(file)"
              @operation="onOperation"
              @play="openPlayer"
            />
          </v-col>
        </v-row>
      </template>
    </v-data-iterator>

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

    <v-dialog v-model="deleteDialog" max-width="480">
      <v-card>
        <v-card-title class="text-h6">
          Delete recordings
        </v-card-title>
        <v-card-text>
          {{ deleteDialogMessage }}
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn text @click="deleteDialog = false">
            Cancel
          </v-btn>
          <v-btn color="error" text :loading="bulkDeleting" @click="confirmBulkDelete">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="repairDialog" max-width="480">
      <v-card>
        <v-card-title class="text-h6">
          Repair recordings
        </v-card-title>
        <v-card-text>
          {{ repairDialogMessage }}
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn text @click="repairDialog = false">
            Cancel
          </v-btn>
          <v-btn color="primary" text :loading="bulkRepairing" @click="confirmBulkRepair">
            Repair
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script lang="ts">
import type { JobStatus, RecordingState as RecordingSessionState } from '@blueos-idl/messages'
import Vue from 'vue'

import McapVideoPlayer from '@/components/records/McapVideoPlayer.vue'
import RecordsRecordingRow from '@/components/records/RecordsRecordingRow.vue'
import RecordsRecordingTable from '@/components/records/RecordsRecordingTable.vue'
import RecordsSessionControls from '@/components/records/RecordsSessionControls.vue'
import type { Transport } from '@/libs/blueos-api/transport'
import zenohTransport from '@/libs/blueos-api/zenoh-transport'
import type { RecordingIndexSource } from '@/libs/mcap/logic/recording-index'
import {
  type BulkAction,
  bulkActionTargets,
  bulkJobEnded,
  runBulkAction,
} from '@/libs/recorder/bulk-actions'
import { createRecorderClient, type RecorderClient } from '@/libs/recorder/client'
import {
  CANCEL_JOB,
  DELETE_RECORDING,
  REPAIR_RECORDING,
  SNAPSHOT_RECORDING,
} from '@/libs/recorder/constants'
import { dateFilterOptions, filterRecordings } from '@/libs/recorder/filter'
import {
  allVisibleSelected,
  pruneSelection,
  selectedVisibleRecordings,
  setVisibleSelection,
  someVisibleSelected,
  togglePathSelection,
} from '@/libs/recorder/selection'
import type {
  LibraryRecording, RecorderCommandResult, RecordingJobResult, RecordingState,
} from '@/libs/recorder/types'
import {
  jobFailureMessage, RECORDING_STATE_UI, recordingByPath, type RepairProgress, withRepairJobs,
} from '@/libs/recorder/view-logic'
import zenoh from '@/libs/zenoh'
import { blueosApiMixin } from '@/mixins/blueosApi'

export default Vue.extend({
  name: 'RecordsView',
  components: {
    RecordsRecordingRow, RecordsRecordingTable, RecordsSessionControls, McapVideoPlayer,
  },
  mixins: [blueosApiMixin],
  data() {
    return {
      transport: null as Transport | null,
      recorder: null as RecorderClient | null,
      recordings: [] as LibraryRecording[],
      recording: null as RecordingSessionState | null,
      repairProgress: {} as RepairProgress,
      jobs: [] as JobStatus[],
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
      selectedPaths: [] as string[],
      deleteDialog: false,
      repairDialog: false,
      deleteTargets: [] as LibraryRecording[],
      repairTargets: [] as LibraryRecording[],
      bulkOperation: null as string | null,
      bulkAction: { failures: [], pending: [] } as BulkAction,
    }
  },
  computed: {
    liveRecordings(): LibraryRecording[] {
      return withRepairJobs(this.recordings, this.repairProgress, this.jobs)
    },
    visibleRecordings(): LibraryRecording[] {
      return filterRecordings(this.liveRecordings, {
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
      return recordingByPath(this.liveRecordings, this.activeRecordingPath)
    },
    selectedFiles(): LibraryRecording[] {
      return selectedVisibleRecordings(this.selectedPaths, this.visibleRecordings)
    },
    selectedTableFiles: {
      get(): LibraryRecording[] {
        return this.selectedFiles
      },
      set(items: LibraryRecording[]): void {
        this.selectedPaths = setVisibleSelection(this.selectedPaths, this.visibleRecordings, items)
      },
    },
    allVisibleSelected(): boolean {
      return allVisibleSelected(this.selectedPaths, this.visibleRecordings)
    },
    someVisibleSelected(): boolean {
      return someVisibleSelected(this.selectedPaths, this.visibleRecordings)
    },
    canDeleteSelected(): boolean {
      return bulkActionTargets(this.selectedFiles, DELETE_RECORDING).length > 0
    },
    canRepairSelected(): boolean {
      return bulkActionTargets(this.selectedFiles, REPAIR_RECORDING).length > 0
    },
    bulkBusy(): boolean {
      return this.bulkOperation !== null
    },
    bulkDeleting(): boolean {
      return this.bulkOperation === DELETE_RECORDING
    },
    bulkRepairing(): boolean {
      return this.bulkOperation === REPAIR_RECORDING
    },
    deleteDialogMessage(): string {
      const targets = bulkActionTargets(this.deleteTargets, DELETE_RECORDING)
      if (targets.length === 1) {
        return `Delete ${targets[0].name}? This cannot be undone.`
      }
      return `Delete ${targets.length} recordings? This cannot be undone.`
    },
    repairDialogMessage(): string {
      const targets = bulkActionTargets(this.repairTargets, REPAIR_RECORDING)
      if (targets.length === 1) {
        return `Repair ${targets[0].name}? This rewrites the file on the vehicle.`
      }
      return `Repair ${targets.length} recordings? This rewrites each file on the vehicle.`
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
          this.selectedPaths = pruneSelection(this.selectedPaths, files.map((file) => file.path))
          this.libraryLoading = false
        },
        (error) => this.showError(error),
      )),
      this.blueosTrackSubscription(this.recorder.watchServiceRunning((running) => {
        this.recorderServiceRunning = running
        if (!running) {
          this.libraryLoading = false
          this.recordings = []
        }
      })),
      this.blueosTrackSubscription(this.recorder.watchRecording(
        (state) => {
          this.recording = state
        },
        (error) => this.showError(error),
      )),
      this.blueosTrackSubscription(this.recorder.watchRepairProgress(
        (progress) => {
          this.repairProgress = progress
        },
        (error) => this.showError(error),
      )),
      this.blueosTrackSubscription(this.recorder.watchJobs(
        (jobs) => {
          this.jobs = jobs
        },
        (error) => this.showError(error),
      )),
      this.blueosTrackSubscription(this.recorder.watchOperations(
        (entry) => this.onRecordingOperation(entry),
        (error) => this.showError(error),
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
    showError(error: unknown): void {
      this.lastError = error instanceof Error ? error.message : String(error)
    },
    onRecordingOperation(entry: RecordingJobResult): void {
      if (bulkJobEnded(this.bulkAction, entry)) {
        this.reportBulkFailures()
        return
      }
      const failure = jobFailureMessage(entry)
      if (failure) {
        this.lastError = failure
      }
    },
    reportBulkFailures(): void {
      if (this.bulkAction.failures.length === 0) {
        return
      }
      this.lastError = this.bulkAction.failures.join('\n')
    },
    toggleSelected(file: LibraryRecording): void {
      this.selectedPaths = togglePathSelection(this.selectedPaths, file.path)
    },
    toggleSelectAllVisible(selected: boolean): void {
      this.selectedPaths = setVisibleSelection(
        this.selectedPaths,
        this.visibleRecordings,
        selected ? this.visibleRecordings : [],
      )
    },
    clearSelection(): void {
      this.selectedPaths = []
    },
    askDelete(files: LibraryRecording[]): void {
      this.deleteTargets = files
      this.deleteDialog = true
    },
    askRepair(files: LibraryRecording[]): void {
      this.repairTargets = files
      this.repairDialog = true
    },
    async confirmBulkDelete(): Promise<void> {
      const targets = this.deleteTargets
      this.deleteDialog = false
      this.deleteTargets = []
      await this.runBulk(DELETE_RECORDING, targets, (recorder, path) => recorder.deleteRecording(path))
    },
    async confirmBulkRepair(): Promise<void> {
      const targets = this.repairTargets
      this.repairDialog = false
      this.repairTargets = []
      await this.runBulk(REPAIR_RECORDING, targets, (recorder, path) => recorder.repairRecording(path))
    },
    async runBulk(
      operationName: string,
      targets: LibraryRecording[],
      submit: (recorder: RecorderClient, path: string) => Promise<RecorderCommandResult>,
    ): Promise<void> {
      const { recorder } = this
      const pending = bulkActionTargets(targets, operationName).map((file) => file.path)
      if (!recorder || pending.length === 0) {
        return
      }
      this.lastError = ''
      this.bulkAction = { failures: [], pending }
      this.bulkOperation = operationName
      try {
        await runBulkAction(this.bulkAction, operationName, (path) => submit(recorder, path))
        this.reportBulkFailures()
      } finally {
        this.bulkOperation = null
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
        this.showError(error)
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
