<template>
  <v-container fluid class="records-view">
    <v-overlay
      v-if="!isSafe"
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
      v-if="!loading && recorderServiceRunning && recordings.length === 0"
      type="info"
      dense
      class="mb-4"
    >
      No recordings found yet.
    </v-alert>

    <v-alert
      v-if="pageBusy"
      type="warning"
      dense
      prominent
      class="mb-4"
    >
      <div class="d-flex align-center flex-wrap">
        <v-icon class="mr-2">
          mdi-open-in-app
        </v-icon>
        <div>
          Keep this page open until the current download or export finishes.
          Those steps run in this browser and will stop if you leave.
          Repair on the vehicle continues, but you would lose progress shown here.
        </div>
      </div>
    </v-alert>

    <v-sheet rounded class="d-flex align-center flex-wrap mb-4 px-2 pt-2 toolbar">
      <v-select
        v-if="dateFilterOptions.length > 1"
        v-model="selectedDate"
        :items="dateFilterOptions"
        label="Filter by date (UTC)"
        dense
        outlined
        hide-details
        class="date-filter mr-3 mb-2"
      />
      <v-checkbox
        v-if="filteredRecordings.length > 0"
        :input-value="allFilteredSelected"
        :indeterminate="someFilteredSelected && !allFilteredSelected"
        dense
        hide-details
        class="mt-0 pt-0 mr-3 mb-2"
        label="Select all"
        @change="toggleSelectAllFiltered"
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
        v-tooltip="'Table'"
        icon
        small
        class="mb-2"
        :color="layout === 'table' ? 'primary' : undefined"
        @click="layout = 'table'"
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
      <div class="mr-4 mb-2">
        <div class="subtitle-2">
          {{ selectedFiles.length }} selected
        </div>
        <div class="caption grey--text text--darken-1">
          {{ selectionStats }}
        </div>
      </div>
      <v-spacer />
      <v-btn
        v-tooltip="canDownloadSelected ? 'Download the selected recordings' : 'Nothing selected can be downloaded'"
        small
        outlined
        color="primary"
        class="mr-2 mb-2"
        :disabled="!canDownloadSelected || pageBusy"
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
        :disabled="!canRepairSelected || pageBusy"
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
        :disabled="!canDeleteSelected || pageBusy"
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

    <v-row v-if="layout === 'cards'">
      <v-col
        v-for="file in filteredRecordings"
        :key="file.path"
        cols="12"
        sm="6"
        md="4"
        lg="3"
      >
        <div class="record-card-wrap">
          <v-checkbox
            :input-value="isSelected(file)"
            dense
            hide-details
            class="card-select"
            @click.stop
            @change="toggleSelected(file)"
          />
          <v-card outlined class="record-card d-flex flex-column">
            <div class="preview-wrapper">
              <div
                class="record-preview grey darken-3 d-flex flex-column align-center justify-center"
                :class="{ 'preview-clickable': canPlay(file) }"
                :role="canPlay(file) ? 'button' : undefined"
                :tabindex="canPlay(file) ? 0 : undefined"
                @click="canPlay(file) && openPlayer(file)"
                @keydown.enter="canPlay(file) && openPlayer(file)"
              >
                <img
                  v-if="thumbnailUrl(file)"
                  :src="thumbnailUrl(file)"
                  class="preview-image"
                  alt=""
                >
                <div class="preview-overlay d-flex flex-column align-center justify-center">
                  <v-btn
                    v-if="canPlay(file)"
                    icon
                    large
                    color="primary"
                    class="play-btn"
                  >
                    <v-icon large>
                      mdi-play-circle
                    </v-icon>
                  </v-btn>
                  <v-progress-circular
                    v-else-if="file.state === 'repairing'"
                    :indeterminate="!repairProgress[file.path]"
                    :value="repairProgress[file.path]?.percent ?? 0"
                    color="primary"
                    size="48"
                  >
                    <span v-if="repairProgress[file.path]" class="caption">
                      {{ Math.round(repairProgress[file.path].percent) }}%
                    </span>
                  </v-progress-circular>
                  <div class="mt-2 caption text-center preview-caption">
                    <div v-if="durationLabel(file)">
                      {{ durationLabel(file) }}
                    </div>
                    <div v-if="file.state === 'recording'">
                      recording…
                    </div>
                    <div v-else-if="endedLabel(file)">
                      ended {{ endedLabel(file) }}
                    </div>
                    <template v-if="file.state === 'needs_repair'">
                      <div>{{ repairFailure(file) ?? 'Recording index is missing' }}</div>
                      <v-btn
                        v-tooltip="'Rewrite this recording on the vehicle so that it can be read'"
                        x-small
                        text
                        color="primary"
                        class="mt-1"
                        :disabled="!canRepair(file)"
                        @click.stop="askRepair([file])"
                      >
                        <v-icon x-small left>
                          mdi-wrench
                        </v-icon>
                        Repair
                      </v-btn>
                    </template>
                    <div v-else-if="file.state === 'repairing'">
                      <div>Repairing recording index…</div>
                      <div v-if="repairProgress[file.path]">
                        {{ repairProgress[file.path].label }}
                      </div>
                      <v-btn
                        v-tooltip="'Stop the rewrite and leave the recording as it is'"
                        x-small
                        text
                        color="primary"
                        class="mt-1"
                        @click.stop="cancelRepair(file)"
                      >
                        <v-icon x-small left>
                          mdi-stop
                        </v-icon>
                        Stop
                      </v-btn>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <v-card-title class="py-2">
              <div class="text-truncate">
                {{ recordingTitle(file) }}
              </div>
            </v-card-title>
            <v-card-subtitle class="py-0">
              <v-chip
                x-small
                class="mr-2"
                :color="stateChipColor(file.state)"
              >
                {{ stateChipLabel(file) }}
              </v-chip>
              <span class="mr-2">{{ formatSize(file.size_bytes) }}</span>
              <span class="caption">{{ formatDate(file.created) }}</span>
            </v-card-subtitle>
            <v-card-subtitle v-if="tracksLabel(file)" class="py-0 caption">
              {{ tracksLabel(file) }}
            </v-card-subtitle>
            <v-spacer />
            <v-card-actions class="pt-0">
              <span v-tooltip="deleteTooltip(file)">
                <v-btn
                  icon
                  small
                  color="error"
                  :disabled="!canDelete(file)"
                  @click="askDelete([file])"
                >
                  <v-icon>mdi-delete</v-icon>
                </v-btn>
              </span>
              <v-btn
                v-if="file.state === 'needs_repair'"
                v-tooltip="'Rewrite this recording on the vehicle so that it can be read'"
                icon
                small
                color="primary"
                :disabled="!canRepair(file)"
                @click="repair(file)"
              >
                <v-icon>mdi-wrench</v-icon>
              </v-btn>
              <v-spacer />
              <span v-tooltip="downloadTooltip(file)">
                <v-btn
                  icon
                  small
                  color="primary"
                  :disabled="!canDownload(file)"
                  @click="downloadRecording(file)"
                >
                  <v-icon>mdi-download</v-icon>
                </v-btn>
              </span>
            </v-card-actions>
          </v-card>
        </div>
      </v-col>
    </v-row>

    <v-card
      v-else
      elevation="1"
    >
      <v-card-text>
        <v-data-table
          v-model="selectedTableItems"
          :headers="tableHeaders"
          :items="tableItems"
          :items-per-page="10"
          :footer-props="{ 'items-per-page-options': [10, 25, 50, -1] }"
          :mobile-breakpoint="0"
          item-key="path"
          show-select
          :sort-by.sync="tableSortBy"
          :sort-desc.sync="tableSortDesc"
          class="records-table"
          @click:row="openPlayerFromRow"
        >
          <template #item.preview="{ item }">
            <div
              class="table-preview grey darken-3"
              :class="{ 'preview-clickable': canPlay(item.file) }"
            >
              <img
                v-if="thumbnailUrl(item.file)"
                :src="thumbnailUrl(item.file)"
                class="table-preview-image"
                alt=""
              >
            </div>
          </template>
          <template #item.name="{ item }">
            <div class="font-weight-medium">
              {{ recordingTitle(item.file) }}
            </div>
          </template>
          <template #item.state="{ item }">
            <v-chip
              x-small
              :color="stateChipColor(item.file.state)"
            >
              {{ stateChipLabel(item.file) }}
            </v-chip>
          </template>
          <template #item.size_bytes="{ item }">
            {{ formatSize(item.file.size_bytes) }}
          </template>
          <template #item.created="{ item }">
            {{ formatDate(item.file.created) }}
          </template>
          <template #item.duration="{ item }">
            {{ durationLabel(item.file) ?? '—' }}
          </template>
          <template #item.tracks="{ item }">
            {{ tracksLabel(item.file) ?? '—' }}
          </template>
          <template #item.actions="{ item }">
            <div class="d-flex align-center justify-end" @click.stop>
              <span v-tooltip="deleteTooltip(item.file)">
                <v-btn
                  icon
                  small
                  color="error"
                  :disabled="!canDelete(item.file)"
                  @click="askDelete([item.file])"
                >
                  <v-icon small>
                    mdi-delete
                  </v-icon>
                </v-btn>
              </span>
              <v-btn
                v-if="item.file.state === 'needs_repair'"
                v-tooltip="'Rewrite this recording on the vehicle so that it can be read'"
                icon
                small
                color="primary"
                :disabled="!canRepair(item.file)"
                @click="askRepair([item.file])"
              >
                <v-icon small>
                  mdi-wrench
                </v-icon>
              </v-btn>
              <v-btn
                v-if="item.file.state === 'repairing'"
                v-tooltip="'Stop the rewrite and leave the recording as it is'"
                icon
                small
                color="primary"
                @click="cancelRepair(item.file)"
              >
                <v-icon small>
                  mdi-stop
                </v-icon>
              </v-btn>
              <span v-tooltip="downloadTooltip(item.file)">
                <v-btn
                  icon
                  small
                  color="primary"
                  :disabled="!canDownload(item.file)"
                  @click="downloadRecording(item.file)"
                >
                  <v-icon small>
                    mdi-download
                  </v-icon>
                </v-btn>
              </span>
            </div>
          </template>
        </v-data-table>
      </v-card-text>
    </v-card>

    <v-dialog
      v-model="playerOpen"
      :fullscreen="$vuetify.breakpoint.smAndDown"
      :max-width="$vuetify.breakpoint.smAndDown ? undefined : 1080"
      scrollable
      :persistent="playerBusy"
      @click:outside="closePlayer"
    >
      <v-card class="player-card">
        <v-card-title class="headline d-flex align-center flex-wrap">
          <div class="text-truncate mr-2">
            {{ activeRecord ? recordingTitle(activeRecord) : '' }}
          </div>
          <span
            v-if="activeRecordMeta"
            class="caption grey--text text--darken-1 font-weight-regular mr-2"
          >
            {{ activeRecordMeta }}
          </span>
          <v-spacer />
          <v-btn
            v-if="activeRecord && canDownload(activeRecord)"
            v-tooltip="'Download the whole recording file. This is not cut to the export time range.'"
            small
            outlined
            color="primary"
            class="mr-2"
            :disabled="!canDownload(activeRecord)"
            @click="downloadRecording(activeRecord)"
          >
            <v-icon small left>
              mdi-download
            </v-icon>
            Download full MCAP
          </v-btn>
          <v-btn
            v-tooltip="'Close'"
            icon
            small
            class="ml-2"
            color="primary"
            :disabled="playerBusy"
            @click.stop="closePlayer"
          >
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-card-title>
        <v-card-text>
          <mcap-video-player
            v-if="activeRecord"
            :key="activeRecord.path"
            :url="recordingStreamUrl(activeRecord)"
            :index-source="recordingIndexSource(activeRecord)"
            :ongoing="activeRecord.state === 'recording'"
            :written-size-bytes="activeWrittenSizeBytes()"
            @busy="onPlayerBusy"
            @summary="onPlayerSummary"
          />
        </v-card-text>
      </v-card>
    </v-dialog>

    <WarningDialog
      v-model="deleteDialog"
      :message="deleteMessage"
      confirm-label="Delete"
      confirm-color="error"
      @confirm="confirmDelete"
    />
    <WarningDialog
      v-model="repairDialog"
      :message="repairMessage"
      confirm-label="Repair"
      @confirm="confirmRepair"
    />
    <WarningDialog
      v-model="leaveDialog"
      :message="leaveMessage"
      confirm-label="Leave anyway"
      confirm-color="error"
      cancel-label="Stay on this page"
      :persistent="true"
      :close-on-outside="false"
      :close-on-esc="false"
      @confirm="confirmLeave"
      @input="onLeaveDialogInput"
    />
  </v-container>
</template>

<script lang="ts">
import Vue from 'vue'
import { NavigationGuardNext, Route } from 'vue-router'

import WarningDialog from '@/components/common/WarningDialog.vue'
import McapVideoPlayer from '@/components/records/McapVideoPlayer.vue'
import { blueosApiMixin } from '@/libs/blueos-api/vue2'
import { McapVideoSummary } from '@/libs/mcap'
import Notifier from '@/libs/notifier'
import {
  activeRecordMetaLabel,
  browsingAllowedWhileArmed,
  canDeleteRecording,
  canDownloadRecording,
  canPlayRecording,
  canRepairRecording,
  createRecorderClient,
  dateFilterOptions,
  deleteConfirmationMessage,
  deleteTooltip,
  downloadTooltip,
  filterRecordingsByDate,
  formatDateFromUnixSeconds,
  formatDuration,
  LibraryRecording,
  RECORDER_SERVICE,
  RecorderClient,
  recordingDurationSeconds,
  RecordingState,
  recordingTitle,
  RECORDS_ALL_DATES,
  RECORDS_LAYOUT_STORAGE_KEY,
  RECORDS_LEAVE_MESSAGE,
  RecordsPageController,
  RecordsPageState,
  repairEstimateMessage,
  repairProgressFromFile,
  selectionStatsLabel,
  stateChipColor as stateChipColorFor,
  stateChipLabel as buildStateChipLabel,
  tracksSummaryLabel,
} from '@/libs/recorder'
import type { Service } from '@/types/common'
import { prettifySize } from '@/utils/helper_functions'

const records_service: Service = {
  name: 'Records',
  description: 'Recording library',
  company: 'Blue Robotics',
  version: '1.0.0',
}
const notifier = new Notifier(records_service)

function storedLayout(): 'cards' | 'table' {
  try {
    return window.localStorage.getItem(RECORDS_LAYOUT_STORAGE_KEY) === 'table' ? 'table' : 'cards'
  } catch {
    return 'cards'
  }
}

interface RecordingTableItem {
  path: string
  file: LibraryRecording
  name: string
  state: RecordingState
  size_bytes: number
  created: number
  duration: number
}

export default Vue.extend({
  name: 'RecordsView',
  components: { McapVideoPlayer, WarningDialog },
  mixins: [blueosApiMixin],
  beforeRouteLeave(_to: Route, _from: Route, next: NavigationGuardNext): void {
    if (!this.pageBusy) {
      next()
      return
    }
    this.pendingLeave = next
    this.leaveDialog = true
  },
  data() {
    const layout = storedLayout()
    const recorder = createRecorderClient() as RecorderClient
    const page: RecordsPageState = {
      vehicleArmed: false,
      recordings: [],
      libraryLoading: true,
      recorderServiceRunning: true,
      playerOpen: false,
      playerBusy: false,
      activeRecord: null,
      selectedDate: RECORDS_ALL_DATES,
      layout,
      summaries: {},
      thumbnails: {},
      selectedPaths: [],
      deleteDialog: false,
      deleteTargets: [],
      repairDialog: false,
      repairTargets: [],
      bulkDownloading: false,
      bulkRepairing: false,
    }
    const pageController = new RecordsPageController(recorder, {
      onState: (state) => { this.page = { ...state } },
      onNotifyError: (type, message) => notifier.pushError(type, message, true),
      onTriggerDownload: (url, fileName) => {
        const link = document.createElement('a')
        link.href = url
        link.download = fileName
        link.click()
      },
    }, layout)
    return {
      recorder,
      pageController,
      page,
      pendingLeave: null as NavigationGuardNext | null,
      leaveDialog: false,
      leaveMessage: RECORDS_LEAVE_MESSAGE,
      tableSortBy: 'created',
      tableSortDesc: true,
    }
  },
  computed: {
    isSafe(): boolean {
      return browsingAllowedWhileArmed(this.page.vehicleArmed)
    },
    recordings(): LibraryRecording[] { return this.page.recordings },
    loading(): boolean { return this.page.libraryLoading },
    recorderServiceRunning(): boolean { return this.page.recorderServiceRunning },
    playerOpen(): boolean { return this.page.playerOpen },
    playerBusy(): boolean { return this.page.playerBusy },
    activeRecord(): LibraryRecording | null { return this.page.activeRecord },
    selectedDate: {
      get(): string { return this.page.selectedDate },
      set(value: string) { this.pageController.setSelectedDate(value) },
    },
    layout: {
      get(): 'cards' | 'table' { return this.page.layout },
      set(value: 'cards' | 'table') { this.pageController.setLayout(value) },
    },
    summaries(): Record<string, McapVideoSummary> { return this.page.summaries },
    thumbnails(): Record<string, string> { return this.page.thumbnails },
    selectedPaths: {
      get(): string[] { return this.page.selectedPaths },
      set(value: string[]) { this.pageController.setSelectedPaths(value) },
    },
    deleteDialog: {
      get(): boolean { return this.page.deleteDialog },
      set(value: boolean) {
        if (!value) this.pageController.clearDeleteDialog()
      },
    },
    repairDialog: {
      get(): boolean { return this.page.repairDialog },
      set(value: boolean) {
        if (!value) this.pageController.clearRepairDialog()
      },
    },
    deleteTargets(): LibraryRecording[] { return this.page.deleteTargets },
    repairTargets(): LibraryRecording[] { return this.page.repairTargets },
    bulkDownloading(): boolean { return this.page.bulkDownloading },
    bulkRepairing(): boolean { return this.page.bulkRepairing },
    repairProgress(): Record<string, { percent: number, label: string }> {
      const progress: Record<string, { percent: number, label: string }> = {}
      for (const file of this.recordings) {
        const entry = repairProgressFromFile(file, (bytes) => this.formatSize(bytes))
        if (entry) progress[file.path] = entry
      }
      return progress
    },
    dateFilterOptions() { return dateFilterOptions(this.recordings) },
    filteredRecordings() { return filterRecordingsByDate(this.recordings, this.selectedDate) },
    tableItems(): RecordingTableItem[] {
      return this.filteredRecordings.map((file) => this.tableRow(file))
    },
    selectedTableItems: {
      get(): RecordingTableItem[] {
        const selected = new Set(this.selectedPaths)
        return this.tableItems.filter((item) => selected.has(item.path))
      },
      set(items: RecordingTableItem[]) { this.onTableSelect(items) },
    },
    selectedFiles(): LibraryRecording[] {
      const selected = new Set(this.selectedPaths)
      return this.filteredRecordings.filter((file) => selected.has(file.path))
    },
    allFilteredSelected(): boolean {
      return this.filteredRecordings.length > 0
        && this.filteredRecordings.every((file) => this.selectedPaths.includes(file.path))
    },
    someFilteredSelected(): boolean {
      return this.filteredRecordings.some((file) => this.selectedPaths.includes(file.path))
    },
    selectionStats(): string {
      const files = this.selectedFiles
      const size = files.reduce((total, file) => total + file.size_bytes, 0)
      const duration = files.reduce((total, file) => total + (this.durationSeconds(file) ?? 0), 0)
      return selectionStatsLabel(files, duration, size, (bytes) => this.formatSize(bytes))
    },
    canDownloadSelected(): boolean { return this.selectedFiles.some((file) => this.canDownload(file)) },
    canRepairSelected(): boolean { return this.selectedFiles.some((file) => this.canRepair(file)) },
    canDeleteSelected(): boolean { return this.selectedFiles.some((file) => this.canDelete(file)) },
    pageBusy(): boolean { return this.playerBusy || this.bulkDownloading || this.bulkRepairing },
    deleteMessage(): string { return deleteConfirmationMessage(this.deleteTargets) },
    repairMessage(): string {
      return repairEstimateMessage(this.repairTargets, (bytes) => this.formatSize(bytes))
    },
    activeRecordMeta(): string | null {
      const file = this.activeRecord
      if (!file) return null
      return activeRecordMetaLabel(
        file,
        this.summaries[file.path],
        this.durationSeconds(file),
        (bytes) => this.formatSize(bytes),
      )
    },
    tableHeaders() {
      return [
        {
          text: '', value: 'preview', sortable: false, width: '88px',
        },
        { text: 'Name', value: 'name' },
        { text: 'State', value: 'state' },
        { text: 'Size', value: 'size_bytes' },
        { text: 'Created', value: 'created' },
        { text: 'Duration', value: 'duration' },
        { text: 'Tracks', value: 'tracks', sortable: false },
        {
          text: '', value: 'actions', sortable: false, align: 'end',
        },
      ]
    },
  },
  watch: {
    isSafe(safe: boolean) {
      if (safe) {
        return
      }
      this.pageController.pauseNetworkActivity()
    },
    pageBusy(busy: boolean) { this.syncLeaveGuard(busy) },
    layout(value: 'cards' | 'table') {
      try {
        window.localStorage.setItem(RECORDS_LAYOUT_STORAGE_KEY, value)
      } catch {
        // Private mode or quota: the current session still keeps the choice.
      }
    },
  },
  mounted() {
    this.blueosTrackUnsubscribe(this.blueosWatchServiceAlive(RECORDER_SERVICE, (alive) => {
      this.pageController.setRecorderServiceRunning(alive)
    }))
    this.blueosTrackUnsubscribe(this.recorder.watchLibrary((files) => {
      this.pageController.setLibrary(files)
    }))
    this.blueosTrackUnsubscribe(this.recorder.watchVehicleArmed((armed) => {
      this.pageController.setVehicleArmed(armed)
    }))
    this.blueosTrackUnsubscribe(this.recorder.watchOperations((event) => {
      this.pageController.handleOperation(event)
    }))
    this.syncLeaveGuard(this.pageBusy)
  },
  beforeDestroy() {
    this.syncLeaveGuard(false)
    this.pageController.destroy()
  },
  methods: {
    activeWrittenSizeBytes(): number | undefined {
      return this.pageController.activeWrittenSizeBytes()
    },
    recordingStreamUrl(file: LibraryRecording): string {
      return this.pageController.recordingStreamUrl(file)
    },
    recordingIndexSource(file: LibraryRecording) {
      return this.pageController.recordingIndexSource(file)
    },
    syncLeaveGuard(busy: boolean): void {
      if (busy) {
        window.addEventListener('beforeunload', this.onBeforeUnload)
        return
      }
      window.removeEventListener('beforeunload', this.onBeforeUnload)
    },
    onBeforeUnload(event: BeforeUnloadEvent): void {
      if (!this.pageBusy) return
      event.preventDefault()
      event.returnValue = this.leaveMessage
    },
    onLeaveDialogInput(open: boolean): void {
      if (open || !this.pendingLeave) return
      this.pendingLeave(false)
      this.pendingLeave = null
    },
    confirmLeave(): void {
      const next = this.pendingLeave
      this.pendingLeave = null
      this.leaveDialog = false
      this.syncLeaveGuard(false)
      next?.()
    },
    isSelected(file: LibraryRecording): boolean {
      return this.selectedPaths.includes(file.path)
    },
    toggleSelected(file: LibraryRecording): void {
      if (this.isSelected(file)) {
        this.selectedPaths = this.selectedPaths.filter((path) => path !== file.path)
        return
      }
      this.selectedPaths = [...this.selectedPaths, file.path]
    },
    toggleSelectAllFiltered(selected: boolean): void {
      this.selectedPaths = selected ? this.filteredRecordings.map((file) => file.path) : []
    },
    clearSelection(): void { this.selectedPaths = [] },
    onTableSelect(items: RecordingTableItem[]): void {
      const visible = new Set(this.tableItems.map((item) => item.path))
      const hiddenSelected = this.selectedPaths.filter((path) => !visible.has(path))
      this.selectedPaths = [...hiddenSelected, ...items.map((item) => item.path)]
    },
    tableRow(file: LibraryRecording): RecordingTableItem {
      return {
        path: file.path,
        file,
        name: recordingTitle(file),
        state: file.state,
        size_bytes: file.size_bytes,
        created: file.created,
        duration: this.durationSeconds(file) ?? -1,
      }
    },
    onPlayerBusy(busy: boolean): void { this.pageController.setPlayerBusy(busy) },
    onPlayerSummary(summary: McapVideoSummary): void {
      const path = this.activeRecord?.path
      if (path) this.pageController.onPlayerSummary(summary, path)
    },
    thumbnailUrl(file: LibraryRecording): string | null {
      return this.thumbnails[file.path] ?? null
    },
    repairFailure(file: LibraryRecording): string | null {
      return file.repair_error || null
    },
    canPlay(file: LibraryRecording): boolean { return canPlayRecording(file) },
    openPlayerFromRow(item: RecordingTableItem): void {
      if (this.canPlay(item.file)) this.openPlayer(item.file)
    },
    canDelete(file: LibraryRecording): boolean {
      return canDeleteRecording(file, this.isSafe)
    },
    canRepair(file: LibraryRecording): boolean {
      return canRepairRecording(file, this.isSafe)
    },
    canDownload(file: LibraryRecording): boolean {
      return canDownloadRecording(file, this.isSafe)
    },
    deleteTooltip(file: LibraryRecording): string { return deleteTooltip(file) },
    downloadTooltip(file: LibraryRecording): string { return downloadTooltip(file) },
    stateChipColor(state: RecordingState): string { return stateChipColorFor(state) },
    stateChipLabel(file: LibraryRecording): string {
      return buildStateChipLabel(file, this.repairProgress[file.path] ?? null)
    },
    durationSeconds(file: LibraryRecording): number | null {
      return recordingDurationSeconds(file, this.summaries, Date.now() / 1000)
    },
    durationLabel(file: LibraryRecording): string | null {
      const duration = this.durationSeconds(file)
      return duration === null ? null : formatDuration(duration)
    },
    endedLabel(file: LibraryRecording): string | null {
      const summary = this.summaries[file.path]
      return summary ? formatDateFromUnixSeconds(summary.ended) : null
    },
    tracksLabel(file: LibraryRecording): string | null {
      return tracksSummaryLabel(this.summaries[file.path])
    },
    askRepair(files: LibraryRecording[]): void {
      this.pageController.askRepair(files.filter((file) => this.canRepair(file)))
    },
    async confirmRepair(): Promise<void> {
      const targets = this.repairTargets
      this.pageController.clearRepairDialog()
      await this.pageController.confirmRepair(targets)
    },
    async cancelRepair(file: LibraryRecording): Promise<void> {
      await this.pageController.cancelRepair(file)
    },
    async downloadRecording(file: LibraryRecording): Promise<void> {
      await this.pageController.downloadRecording(file, this.canDownload(file))
    },
    async downloadSelected(): Promise<void> {
      const files = this.selectedFiles.filter((file) => this.canDownload(file))
      await this.pageController.downloadSelected(files)
    },
    askDelete(files: LibraryRecording[]): void {
      this.pageController.askDelete(files.filter((file) => this.canDelete(file)))
    },
    async confirmDelete(): Promise<void> {
      const targets = this.deleteTargets
      this.pageController.clearDeleteDialog()
      await this.pageController.confirmDelete(targets)
    },
    openPlayer(file: LibraryRecording): void {
      if (!this.canPlay(file) || !this.isSafe) return
      this.pageController.openPlayer(file)
    },
    closePlayer(): void {
      if (this.playerBusy) return
      this.pageController.closePlayer()
    },
    recordingTitle(file: LibraryRecording): string { return recordingTitle(file) },
    formatSize(bytes: number): string { return prettifySize(bytes / 1024) },
    formatDate(timestamp: number): string { return formatDateFromUnixSeconds(timestamp) },
  },
})

</script>

<style scoped>
.records-view {
  position: relative;
  min-height: 100%;
}

.record-card {
  height: 100%;
  overflow: hidden;
}

.record-card-wrap {
  position: relative;
  height: 100%;
}

.card-select {
  position: absolute;
  top: 4px;
  left: 8px;
  z-index: 2;
}

.preview-wrapper {
  position: relative;
}

.record-preview {
  position: relative;
  overflow: hidden;
  height: 180px;
}

.preview-image {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.preview-overlay {
  position: relative;
  z-index: 1;
  width: 100%;
  height: 100%;
  padding: 8px;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.55), rgba(0, 0, 0, 0.15));
}

.preview-caption {
  color: rgba(255, 255, 255, 0.9);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.7);
}

.play-btn {
  background-color: rgba(255, 255, 255, 0.85) !important;
  pointer-events: all;
}

.preview-clickable {
  cursor: pointer;
}

.player-card {
  position: relative;
  max-height: 100%;
}

.date-filter {
  max-width: 280px;
  min-width: 200px;
}

.records-table ::v-deep tbody tr {
  cursor: pointer;
}

.table-preview {
  position: relative;
  width: 72px;
  height: 40px;
  overflow: hidden;
  border-radius: 4px;
}

.table-preview-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.toolbar {
  gap: 4px;
}

.min-width-0 {
  min-width: 0;
}

.mr-2 {
  margin-right: 8px;
}
</style>
