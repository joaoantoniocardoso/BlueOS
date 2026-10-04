<template>
  <v-card class="records-row d-flex flex-column" outlined>
    <v-checkbox
      v-if="selectable"
      :input-value="selected"
      dense
      hide-details
      class="card-select ma-2 mb-0"
      @click.stop
      @change="$emit('toggle-select')"
    />
    <records-recording-preview
      :file="file"
      :byte-source="byteSource"
      :disabled="disabled"
      @play="$emit('play', file)"
    />
    <v-card-title class="py-2 text-truncate">
      {{ file.name }}
    </v-card-title>
    <v-card-subtitle class="py-0">
      <v-chip
        x-small
        :color="stateColor"
        class="mr-2"
      >
        {{ stateLabel }}
      </v-chip>
      <span class="mr-2">{{ formatSize(file.size_bytes) }}</span>
      <span class="caption">{{ formatDate(file.created) }}</span>
    </v-card-subtitle>
    <v-card-subtitle class="py-0 caption">
      <div>{{ durationLabel(file) }} &middot; {{ tracksLabel(file) }}</div>
      <div v-if="endedText">
        Ended {{ endedText }}
      </div>
      <div v-if="caption">
        {{ caption }}
      </div>
    </v-card-subtitle>
    <records-repair-progress :file="file" class="px-4 py-2" />
    <v-spacer />
    <v-card-actions class="pt-0 flex-wrap">
      <span
        v-for="operationName in operationButtons(file)"
        :key="operationName"
        v-tooltip="operationTooltip(operationName)"
        class="mr-1 mb-1"
      >
        <v-btn
          small
          :color="operationColor(operationName)"
          :loading="busyOperation === operationName"
          :disabled="disabled || operationDisabledReason(file, operationName) !== null"
          @click="runOperation(operationName)"
        >
          <v-icon small left>
            {{ operationIcon(operationName) }}
          </v-icon>
          {{ operationLabel(operationName) }}
        </v-btn>
      </span>
      <span v-tooltip="downloadTooltip">
        <v-btn
          :aria-label="downloadTooltip"
          icon
          small
          color="primary"
          :loading="downloading"
          :disabled="disabled || !canDownload"
          @click.stop="$emit('download', file)"
        >
          <v-icon>mdi-download</v-icon>
        </v-btn>
      </span>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import RecordsRecordingPreview from '@/components/records/RecordsRecordingPreview.vue'
import RecordsRepairProgress from '@/components/records/RecordsRepairProgress.vue'
import type { ByteSource } from '@/libs/mcap/logic/byte-source'
import { DOWNLOAD } from '@/libs/recorder/constants'
import type { LibraryRecording } from '@/libs/recorder/types'
import {
  canDownloadRecording,
  downloadTooltip,
  durationLabel,
  operationButtons,
  operationDisabledReason,
  RECORDING_OPERATION_UI,
  RECORDING_STATE_UI,
  recordingCaption,
  recordingEndSeconds,
  tracksLabel,
} from '@/libs/recorder/view-logic'
import { prettifySize } from '@/utils/helper_functions'

export default Vue.extend({
  name: 'RecordsRecordingRow',
  components: { RecordsRecordingPreview, RecordsRepairProgress },
  props: {
    file: {
      type: Object as PropType<LibraryRecording>,
      required: true,
    },
    byteSource: {
      type: Function as PropType<(path: string) => ByteSource | undefined>,
      required: true,
    },
    disabled: {
      type: Boolean,
      default: false,
    },
    busyOperation: {
      type: String as PropType<string | null>,
      default: null,
    },
    selectable: {
      type: Boolean,
      default: false,
    },
    selected: {
      type: Boolean,
      default: false,
    },
  },
  computed: {
    canDownload(): boolean {
      return canDownloadRecording(this.file)
    },
    downloading(): boolean {
      return this.busyOperation === DOWNLOAD
    },
    downloadTooltip(): string {
      return downloadTooltip(this.file)
    },
    stateColor(): string {
      return RECORDING_STATE_UI[this.file.state]?.color ?? 'grey'
    },
    stateLabel(): string {
      return RECORDING_STATE_UI[this.file.state]?.label ?? this.file.state
    },
    caption(): string | null {
      return recordingCaption(this.file)
    },
    endedText(): string | null {
      const endSeconds = recordingEndSeconds(this.file)
      return endSeconds === null ? null : this.formatDate(endSeconds)
    },
  },
  methods: {
    durationLabel,
    operationButtons,
    operationDisabledReason,
    tracksLabel,
    formatSize(bytes: number): string {
      return prettifySize(bytes / 1024)
    },
    formatDate(timestamp: number): string {
      return new Date(timestamp * 1000).toLocaleString()
    },
    operationLabel(operationName: string): string {
      return RECORDING_OPERATION_UI[operationName]?.label ?? operationName
    },
    operationIcon(operationName: string): string {
      return RECORDING_OPERATION_UI[operationName]?.icon ?? 'mdi-playlist-check'
    },
    operationColor(operationName: string): string {
      return RECORDING_OPERATION_UI[operationName]?.color ?? 'primary'
    },
    operationTooltip(operationName: string): string {
      return operationDisabledReason(this.file, operationName)
        ?? `${this.operationLabel(operationName)} ${this.file.name}`
    },
    runOperation(operationName: string): void {
      this.$emit('operation', operationName, this.file)
    },
  },
})
</script>
