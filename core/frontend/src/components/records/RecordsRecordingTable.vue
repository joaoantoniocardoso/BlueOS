<template>
  <v-data-table
    :value="selectedFiles"
    :headers="headers"
    :items="files"
    item-key="path"
    sort-by="created"
    sort-desc
    show-select
    :items-per-page="25"
    :footer-props="{ 'items-per-page-options': [10, 25, 50, -1] }"
    :mobile-breakpoint="0"
    class="records-table"
    @input="$emit('update:selected-files', $event)"
  >
    <template #item.name="{ item }">
      <span class="font-weight-medium">{{ item.name }}</span>
    </template>
    <template #item.state="{ item }">
      <v-chip x-small :color="stateUi(item).color">
        {{ stateUi(item).label }}
      </v-chip>
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
        <v-btn
          v-for="operationName in item.allowed_operations"
          :key="operationName"
          v-tooltip="`${operationUi(operationName).label} ${item.name}`"
          icon
          small
          :color="operationUi(operationName).color"
          :loading="busyPath === item.path && busyOperation === operationName"
          :disabled="disabled"
          @click="$emit('operation', operationName, item)"
        >
          <v-icon small>
            {{ operationUi(operationName).icon }}
          </v-icon>
        </v-btn>
        <v-btn
          v-if="item.state === 'ready'"
          v-tooltip="`Download ${item.name}`"
          icon
          small
          color="primary"
          :href="downloadUrl(item.path)"
          :download="item.name"
          :disabled="disabled"
        >
          <v-icon small>
            mdi-download
          </v-icon>
        </v-btn>
      </div>
    </template>
  </v-data-table>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import type { LibraryRecording } from '@/libs/recorder/types'
import { canPlayRecording, RECORDING_OPERATION_UI, RECORDING_STATE_UI } from '@/libs/recorder/view-logic'
import { prettifySize } from '@/utils/helper_functions'

export default Vue.extend({
  name: 'RecordsRecordingTable',
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
  },
  data() {
    return {
      headers: [
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
    stateUi(file: LibraryRecording): { label: string, color: string } {
      return RECORDING_STATE_UI[file.state] ?? { label: file.state, color: 'grey' }
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
