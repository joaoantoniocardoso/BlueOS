<template>
  <v-container fluid class="records-view">
    <v-overlay
      v-if="armed"
      absolute
      :opacity="0.9"
      z-index="10"
    >
      <div class="d-flex flex-column align-center text-center pa-4">
        <v-icon large color="warning" class="mb-3">
          mdi-alert-outline
        </v-icon>
        <p class="mb-0">
          Recording browsing is paused while the vehicle is armed,
          so the link stays free for vehicle control.
        </p>
      </div>
    </v-overlay>

    <records-session-controls
      class="records-session"
      :recorder="recorder"
      :recording="recording"
      :service-running="recorderServiceRunning"
    />

    <v-alert
      v-if="recordings.length > 0"
      type="warning"
      dense
      class="mb-4"
    >
      Playing or downloading a recording pulls it over the vehicle link and can take most of the
      available bandwidth, which may disturb vehicle operations. Close the player when you are done with it.
    </v-alert>

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
      v-if="leaveGuarded"
      type="warning"
      dense
      prominent
      icon="mdi-open-in-app"
      class="mb-4"
    >
      Keep this page open until the current download or export finishes.
      Those steps run in this browser and will stop if you leave.
      Repair on the vehicle continues, but you would lose progress shown here.
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
      <v-select
        v-model="sort.key"
        :items="sortOptions"
        label="Sort by"
        dense
        outlined
        hide-details
        class="records-filter mb-2"
      />
      <v-btn
        v-tooltip="sort.descending ? 'Descending: highest first' : 'Ascending: lowest first'"
        :aria-label="sort.descending ? 'Sort ascending' : 'Sort descending'"
        icon
        small
        class="mr-3 mb-2"
        @click="sort.descending = !sort.descending"
      >
        <v-icon small>
          {{ sort.descending ? 'mdi-sort-descending' : 'mdi-sort-ascending' }}
        </v-icon>
      </v-btn>
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
        aria-label="Cards"
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
        aria-label="List"
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
        v-tooltip="canDownloadSelected
          ? 'Download the selected recordings. The browser may ask to allow several downloads.'
          : 'Nothing selected can be downloaded'"
        small
        outlined
        color="primary"
        class="mr-2 mb-2"
        :disabled="!canDownloadSelected || bulkBusy || actionsDisabled"
        :loading="bulkDownloading"
        @click="downloadSelected"
      >
        <v-icon small left>
          mdi-download
        </v-icon>
        Download
      </v-btn>
      <v-btn
        v-tooltip="canRepairSelected
          ? 'Rewrite selected recordings so they can be read'
          : 'Nothing selected needs repair'"
        small
        outlined
        color="primary"
        class="mr-2 mb-2"
        :disabled="!canRepairSelected || bulkBusy || actionsDisabled"
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
        :disabled="!canDeleteSelected || bulkBusy || actionsDisabled"
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
      :sort-key.sync="sort.key"
      :sort-descending.sync="sort.descending"
      :download-url="downloadUrl"
      :disabled="actionsDisabled"
      :busy-path="busyPath"
      :busy-operation="busyOperation"
      @operation="onOperation"
      @download="downloadRecording"
      @play="openPlayer"
    />

    <v-data-iterator
      v-else-if="visibleRecordings.length > 0"
      :items="visibleRecordings"
      item-key="path"
      disable-sort
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
              :disabled="actionsDisabled"
              :busy-operation="busyPath === file.path ? busyOperation : null"
              selectable
              :selected="selectedPaths.includes(file.path)"
              @toggle-select="toggleSelected(file)"
              @operation="onOperation"
              @download="downloadRecording"
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
          <v-btn
            v-if="canDownload(activeRecording)"
            v-tooltip="'Download the whole recording file. This is not cut to the export time range.'"
            small
            outlined
            color="primary"
            class="mr-2"
            :loading="downloading(activeRecording)"
            @click="downloadRecording(activeRecording)"
          >
            <v-icon small left>
              mdi-download
            </v-icon>
            Download full MCAP
          </v-btn>
          <v-btn
            v-tooltip="'Close'"
            aria-label="Close"
            icon
            @click="closePlayer"
          >
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

    <warning-dialog
      v-model="leaveDialog"
      :message="leaveMessage"
      confirm-label="Leave anyway"
      confirm-color="error"
      cancel-label="Stay on this page"
      persistent
      :close-on-outside="false"
      :close-on-esc="false"
      @confirm="confirmLeave"
      @input="onLeaveDialogInput"
    />
  </v-container>
</template>

<script lang="ts">
import type { JobStatus, RecordingState as RecordingSessionState } from '@blueos-idl/messages'
import Vue from 'vue'
import type { NavigationGuardNext, Route } from 'vue-router'

import WarningDialog from '@/components/common/WarningDialog.vue'
import McapVideoPlayer from '@/components/records/McapVideoPlayer.vue'
import RecordsRecordingRow from '@/components/records/RecordsRecordingRow.vue'
import RecordsRecordingTable from '@/components/records/RecordsRecordingTable.vue'
import RecordsSessionControls from '@/components/records/RecordsSessionControls.vue'
import type { Transport } from '@/libs/blueos-api/transport'
import zenohTransport from '@/libs/blueos-api/zenoh-transport'
import type { RecordingIndexSource } from '@/libs/mcap/logic/recording-index'
import message_manager, { MessageLevel } from '@/libs/message-manager'
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
  DOWNLOAD,
  RECORDS_LEAVE_MESSAGE,
  REPAIR_RECORDING,
} from '@/libs/recorder/constants'
import { dateFilterOptions, filterRecordings } from '@/libs/recorder/filter'
import {
  browserStorage,
  type RecordsLayout,
  type RecordsSort,
  storedLayout,
  storedSort,
  storeLayout,
  storeSort,
} from '@/libs/recorder/preferences'
import {
  allVisibleSelected,
  pruneSelection,
  selectedVisibleRecordings,
  setVisibleSelection,
  someVisibleSelected,
  togglePathSelection,
} from '@/libs/recorder/selection'
import { RECORDING_SORT_OPTIONS, sortRecordings } from '@/libs/recorder/sort'
import type {
  LibraryRecording, RecorderCommandResult, RecordingJobResult, RecordingState,
} from '@/libs/recorder/types'
import {
  canDownloadRecording,
  deleteConfirmationMessage,
  jobCanceledMessage,
  jobFailureMessage,
  RECORDING_STATE_UI,
  recordingByPath,
  repairEstimateMessage,
  type RepairProgress,
  withLiveDuration,
  withRepairJobs,
} from '@/libs/recorder/view-logic'
import zenoh from '@/libs/zenoh'
import { blueosApiMixin } from '@/mixins/blueosApi'

export default Vue.extend({
  name: 'RecordsView',
  components: {
    RecordsRecordingRow, RecordsRecordingTable, RecordsSessionControls, McapVideoPlayer, WarningDialog,
  },
  mixins: [blueosApiMixin],
  beforeRouteLeave(_to: Route, _from: Route, next: NavigationGuardNext): void {
    if (!this.leaveGuarded) {
      next()
      return
    }
    this.pendingLeave = next
    this.leaveDialog = true
  },
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
      layout: storedLayout(browserStorage),
      sort: storedSort(browserStorage),
      sortOptions: RECORDING_SORT_OPTIONS,
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
      leaveDialog: false,
      pendingLeave: null as NavigationGuardNext | null,
      nowSeconds: Date.now() / 1000,
      clockId: 0,
    }
  },
  computed: {
    armed(): boolean {
      return this.recording?.armed ?? false
    },
    actionsDisabled(): boolean {
      return !this.recorderServiceRunning || this.armed
    },
    leaveGuarded(): boolean {
      return this.playerBusy || this.busyOperation === DOWNLOAD || this.bulkDownloading || this.bulkRepairing
    },
    leaveMessage(): string {
      return RECORDS_LEAVE_MESSAGE
    },
    liveRecordings(): LibraryRecording[] {
      return withLiveDuration(withRepairJobs(this.recordings, this.repairProgress, this.jobs), this.nowSeconds)
    },
    visibleRecordings(): LibraryRecording[] {
      const filtered = filterRecordings(this.liveRecordings, {
        search: this.search ?? '',
        state: this.stateFilter,
        date: this.dateFilter,
      })
      return sortRecordings(filtered, this.sort.key, this.sort.descending)
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
    canDownloadSelected(): boolean {
      return this.selectedFiles.some((file) => this.canDownload(file))
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
    bulkDownloading(): boolean {
      return this.bulkOperation === DOWNLOAD
    },
    bulkDeleting(): boolean {
      return this.bulkOperation === DELETE_RECORDING
    },
    bulkRepairing(): boolean {
      return this.bulkOperation === REPAIR_RECORDING
    },
    deleteDialogMessage(): string {
      return deleteConfirmationMessage(bulkActionTargets(this.deleteTargets, DELETE_RECORDING))
    },
    repairDialogMessage(): string {
      return repairEstimateMessage(bulkActionTargets(this.repairTargets, REPAIR_RECORDING))
    },
  },
  watch: {
    armed(armed: boolean): void {
      if (armed) {
        this.playerOpen = false
        this.playerBusy = false
        this.activeRecordingPath = null
      }
    },
    layout(layout: RecordsLayout): void {
      storeLayout(browserStorage, layout)
    },
    sort: {
      deep: true,
      handler(sort: RecordsSort): void {
        storeSort(browserStorage, sort)
      },
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
  mounted() {
    window.addEventListener('beforeunload', this.onBeforeUnload)
    this.clockId = window.setInterval(() => {
      if (this.recordings.some((file) => file.state === 'recording')) {
        this.nowSeconds = Date.now() / 1000
      }
    }, 1000)
  },
  beforeDestroy() {
    window.removeEventListener('beforeunload', this.onBeforeUnload)
    window.clearInterval(this.clockId)
  },
  methods: {
    onBeforeUnload(event: BeforeUnloadEvent): void {
      if (!this.leaveGuarded) {
        return
      }
      event.preventDefault()
      event.returnValue = this.leaveMessage
    },
    onLeaveDialogInput(open: boolean): void {
      if (open || !this.pendingLeave) {
        return
      }
      this.pendingLeave(false)
      this.pendingLeave = null
    },
    confirmLeave(): void {
      const next = this.pendingLeave
      this.pendingLeave = null
      this.leaveDialog = false
      next?.()
    },
    downloadUrl(path: string): string {
      return this.recorder?.recordingDownloadUrl(path) ?? ''
    },
    canDownload: canDownloadRecording,
    downloading(file: LibraryRecording): boolean {
      return this.busyPath === file.path && this.busyOperation === DOWNLOAD
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
      const canceled = jobCanceledMessage(entry)
      if (canceled) {
        message_manager.emitMessage(MessageLevel.Info, canceled)
      }
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
      if (operationName === REPAIR_RECORDING) {
        this.askRepair([file])
        return
      }
      if (operationName === DELETE_RECORDING) {
        this.askDelete([file])
        return
      }
      this.lastError = ''
      this.busyPath = file.path
      this.busyOperation = operationName
      try {
        if (operationName === CANCEL_JOB) {
          const result = await this.recorder.cancelRepair(file.repair_job_id)
          if (!result.accepted) {
            this.lastError = result.reason
          }
        }
      } catch (error) {
        this.showError(error)
      } finally {
        this.busyPath = null
        this.busyOperation = null
      }
    },
    async downloadRecording(file: LibraryRecording): Promise<void> {
      if (!this.recorder) {
        return
      }
      this.busyPath = file.path
      this.busyOperation = DOWNLOAD
      try {
        const path = await this.recorder.recordingDownloadPath(file)
        this.triggerDownload(this.recorder.recordingDownloadUrl(path), path.split('/').pop() ?? file.name)
      } catch (error) {
        this.showError(error)
      } finally {
        this.busyPath = null
        this.busyOperation = null
      }
    },
    async downloadSelected(): Promise<void> {
      this.lastError = ''
      this.bulkOperation = DOWNLOAD
      try {
        for (const file of this.selectedFiles.filter((selected) => this.canDownload(selected))) {
          // eslint-disable-next-line no-await-in-loop
          await this.downloadRecording(file)
        }
      } finally {
        this.bulkOperation = null
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
  position: relative;
  min-height: 100%;
}

.records-session {
  position: relative;
  z-index: 11;
}

.records-search {
  max-width: 320px;
}

.records-filter {
  max-width: 200px;
}
</style>
