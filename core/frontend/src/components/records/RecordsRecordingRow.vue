<template>
  <v-card class="records-row d-flex flex-column" outlined>
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
    <v-card-text v-if="file.state === 'repairing' && repairLabel" class="py-2">
      <v-progress-linear
        :value="repairPercent"
        color="primary"
        height="8"
        rounded
      />
      <div class="caption mt-1">
        {{ repairLabel }}
      </div>
    </v-card-text>
    <v-spacer />
    <v-card-actions class="pt-0 flex-wrap">
      <v-btn
        v-for="operationName in file.allowed_operations"
        :key="operationName"
        v-tooltip="operationTooltip(operationName)"
        small
        :color="operationColor(operationName)"
        :loading="busyOperation === operationName"
        :disabled="disabled"
        class="mr-1 mb-1"
        @click="runOperation(operationName)"
      >
        <v-icon small left>
          {{ operationIcon(operationName) }}
        </v-icon>
        {{ operationLabel(operationName) }}
      </v-btn>
      <v-btn
        v-if="file.state === 'ready'"
        v-tooltip="`Download ${file.name}`"
        icon
        small
        color="primary"
        :href="downloadUrl"
        :download="file.name"
        :disabled="disabled"
        @click.stop
      >
        <v-icon>mdi-download</v-icon>
      </v-btn>
    </v-card-actions>
  </v-card>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import {
  CANCEL_REPAIR,
  DELETE_RECORDING,
  REPAIR_RECORDING,
  SNAPSHOT_RECORDING,
} from '@/libs/recorder/constants'
import type { LibraryRecording } from '@/libs/recorder/types'
import { prettifySize } from '@/utils/helper_functions'

export default Vue.extend({
  name: 'RecordsRecordingRow',
  props: {
    file: {
      type: Object as PropType<LibraryRecording>,
      required: true,
    },
    downloadUrl: {
      type: String,
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
  },
  computed: {
    stateColor(): string {
      const colors: Record<string, string> = {
        recording: 'warning',
        needs_repair: 'error',
        repairing: 'primary',
        ready: 'success',
      }
      return colors[this.file.state] ?? 'grey'
    },
    stateLabel(): string {
      const labels: Record<string, string> = {
        recording: 'Recording',
        needs_repair: 'Needs repair',
        repairing: 'Repairing',
        ready: 'Ready',
      }
      return labels[this.file.state] ?? this.file.state
    },
    repairPercent(): number {
      if (this.file.repair_total_bytes <= 0) {
        return 0
      }
      return Math.min(100, this.file.repair_bytes_processed / this.file.repair_total_bytes * 100)
    },
    repairLabel(): string | null {
      if (this.file.state !== 'repairing' || this.file.repair_total_bytes <= 0) {
        return null
      }
      const read = prettifySize(this.file.repair_bytes_processed / 1024)
      const total = prettifySize(this.file.repair_total_bytes / 1024)
      return `${read} of ${total}`
    },
  },
  methods: {
    formatSize(bytes: number): string {
      return prettifySize(bytes / 1024)
    },
    formatDate(timestamp: number): string {
      return new Date(timestamp * 1000).toLocaleString()
    },
    operationLabel(operationName: string): string {
      switch (operationName) {
        case REPAIR_RECORDING:
          return 'Repair'
        case CANCEL_REPAIR:
          return 'Cancel repair'
        case DELETE_RECORDING:
          return 'Delete'
        case SNAPSHOT_RECORDING:
          return 'Download snapshot'
        default:
          return operationName
      }
    },
    operationIcon(operationName: string): string {
      switch (operationName) {
        case REPAIR_RECORDING:
          return 'mdi-wrench'
        case CANCEL_REPAIR:
          return 'mdi-stop'
        case DELETE_RECORDING:
          return 'mdi-delete'
        case SNAPSHOT_RECORDING:
          return 'mdi-download'
        default:
          return 'mdi-playlist-check'
      }
    },
    operationColor(operationName: string): string {
      if (operationName === DELETE_RECORDING) {
        return 'error'
      }
      return 'primary'
    },
    operationTooltip(operationName: string): string {
      return `${this.operationLabel(operationName)} ${this.file.name}`
    },
    runOperation(operationName: string): void {
      this.$emit('operation', operationName, this.file)
    },
  },
})
</script>
