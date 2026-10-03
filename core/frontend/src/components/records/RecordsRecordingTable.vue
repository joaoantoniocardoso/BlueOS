<template>
  <v-data-table
    :value="selectedFiles"
    :headers="headers"
    :items="files"
    item-key="path"
    :sort-by="sortKey"
    :sort-desc="sortDescending"
    :custom-sort="keepOrder"
    must-sort
    show-select
    :items-per-page="25"
    :footer-props="{ 'items-per-page-options': [10, 25, 50, -1] }"
    :mobile-breakpoint="0"
    class="records-table"
    @input="$emit('update:selected-files', $event)"
    @update:sort-by="$emit('update:sort-key', $event)"
    @update:sort-desc="$emit('update:sort-descending', $event)"
  >
    <template #item.preview="{ item }">
      <records-recording-preview
        class="table-preview"
        :file="item"
        :download-url="downloadUrl(item.path)"
        :disabled="disabled"
        compact
      />
    </template>
    <template #item.name="{ item }">
      <span class="font-weight-medium">{{ item.name }}</span>
    </template>
    <template #item.state="{ item }">
      <v-chip x-small :color="stateUi(item).color">
        {{ stateUi(item).label }}
      </v-chip>
      <records-repair-progress :file="item" class="my-1" />
    </template>
    <template #item.size_bytes="{ item }">
      {{ formatSize(item.size_bytes) }}
    </template>
    <template #item.created="{ item }">
      {{ formatDate(item.created) }}
    </template>
    <template #item.actions="{ item }">
      <div class="d-flex align-center justify-end">
        <v-btn
          v-if="canPlay(item)"
          v-tooltip="`Play ${item.name}`"
          :aria-label="`Play ${item.name}`"
          icon
          small
          color="primary"
          :disabled="disabled"
          @click="$emit('play', item)"
        >
          <v-icon small>
            mdi-play-circle
          </v-icon>
        </v-btn>
        <span
          v-for="operationName in operationButtons(item)"
          :key="operationName"
          v-tooltip="operationTooltip(item, operationName)"
        >
          <v-btn
            :aria-label="operationTooltip(item, operationName)"
            icon
            small
            :color="operationUi(operationName).color"
            :loading="busyPath === item.path && busyOperation === operationName"
            :disabled="disabled || operationDisabledReason(item, operationName) !== null"
            @click="$emit('operation', operationName, item)"
          >
            <v-icon small>
              {{ operationUi(operationName).icon }}
            </v-icon>
          </v-btn>
        </span>
        <span v-tooltip="downloadTooltip(item)">
          <v-btn
            :aria-label="downloadTooltip(item)"
            icon
            small
            color="primary"
            :loading="downloading(item)"
            :disabled="disabled || !canDownload(item)"
            @click="$emit('download', item)"
          >
            <v-icon small>
              mdi-download
            </v-icon>
          </v-btn>
        </span>
      </div>
    </template>
  </v-data-table>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import RecordsRecordingPreview from '@/components/records/RecordsRecordingPreview.vue'
import RecordsRepairProgress from '@/components/records/RecordsRepairProgress.vue'
import { DOWNLOAD } from '@/libs/recorder/constants'
import type { RecordingSortKey } from '@/libs/recorder/sort'
import type { LibraryRecording } from '@/libs/recorder/types'
import {
  canDownloadRecording,
  canPlayRecording,
  downloadTooltip,
  operationButtons,
  operationDisabledReason,
  RECORDING_OPERATION_UI,
  RECORDING_STATE_UI,
} from '@/libs/recorder/view-logic'
import { prettifySize } from '@/utils/helper_functions'

export default Vue.extend({
  name: 'RecordsRecordingTable',
  components: { RecordsRecordingPreview, RecordsRepairProgress },
  props: {
    files: {
      type: Array as PropType<LibraryRecording[]>,
      required: true,
    },
    downloadUrl: {
      type: Function as PropType<(path: string) => string>,
      required: true,
    },
    disabled: {
      type: Boolean,
      default: false,
    },
    busyPath: {
      type: String as PropType<string | null>,
      default: null,
    },
    busyOperation: {
      type: String as PropType<string | null>,
      default: null,
    },
    selectedFiles: {
      type: Array as PropType<LibraryRecording[]>,
      default: () => [],
    },
    sortKey: {
      type: String as PropType<RecordingSortKey>,
      required: true,
    },
    sortDescending: {
      type: Boolean,
      required: true,
    },
  },
  data() {
    return {
      headers: [
        {
          text: '',
          value: 'preview',
          sortable: false,
          width: 96,
        },
        { text: 'Name', value: 'name' },
        { text: 'State', value: 'state' },
        { text: 'Size', value: 'size_bytes' },
        { text: 'Created', value: 'created' },
        {
          text: '',
          value: 'actions',
          sortable: false,
          align: 'end',
        },
      ],
    }
  },
  methods: {
    canPlay: canPlayRecording,
    downloadTooltip,
    /** The view sorts `files` with the sort the cards share; the headers only change that sort. */
    keepOrder(items: LibraryRecording[]): LibraryRecording[] {
      return items
    },
    operationButtons,
    operationDisabledReason,
    canDownload: canDownloadRecording,
    downloading(file: LibraryRecording): boolean {
      return this.busyPath === file.path && this.busyOperation === DOWNLOAD
    },
    operationTooltip(file: LibraryRecording, operationName: string): string {
      return operationDisabledReason(file, operationName) ?? `${this.operationUi(operationName).label} ${file.name}`
    },
    stateUi(file: LibraryRecording): { label: string, color: string } {
      return RECORDING_STATE_UI[file.state] ?? { label: file.state, color: 'secondary' }
    },
    operationUi(operationName: string): { label: string, icon: string, color: string } {
      return RECORDING_OPERATION_UI[operationName]
        ?? { label: operationName, icon: 'mdi-playlist-check', color: 'primary' }
    },
    formatSize(bytes: number): string {
      return prettifySize(bytes / 1024)
    },
    formatDate(timestamp: number): string {
      return new Date(timestamp * 1000).toLocaleString()
    },
  },
})
</script>

<style scoped>
.table-preview {
  width: 96px;
}
</style>
