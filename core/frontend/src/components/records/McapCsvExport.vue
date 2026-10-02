<template>
  <div class="csv-export">
    <div class="d-flex align-center mb-2">
      <span class="caption grey--text text--darken-1">
        {{ selection_label }}
      </span>
      <v-spacer />
      <span v-if="range_label" class="caption grey--text text--darken-1">
        {{ range_label }}
      </span>
    </div>

    <div class="d-flex align-center mb-2 flex-wrap">
      <v-text-field
        v-model="search"
        v-tooltip="'Filter the channel list'"
        dense
        hide-details
        outlined
        clearable
        label="Search channels"
        prepend-inner-icon="mdi-magnify"
        class="channel-search mr-2 mb-2"
        :disabled="Boolean(export_progress)"
      />
      <v-btn
        v-tooltip="'Include every channel in the export'"
        small
        text
        class="mr-1"
        :disabled="Boolean(export_progress)"
        @click="selectAll"
      >
        Select all
      </v-btn>
      <v-btn
        v-tooltip="'Clear the channel selection'"
        small
        text
        :disabled="Boolean(export_progress)"
        @click="selectNone"
      >
        Clear
      </v-btn>
      <v-btn
        v-tooltip="'Select every channel except video and raw byte streams'"
        small
        text
        :disabled="Boolean(export_progress)"
        @click="selectDefaults"
      >
        Default
      </v-btn>
    </div>

    <v-data-table
      v-model="selected"
      :headers="headers"
      :items="channels"
      :search="search"
      item-key="channelId"
      show-select
      dense
      :items-per-page="8"
      class="channel-table elevation-0"
      :loading="channels.length === 0"
      :footer-props="{ 'items-per-page-options': [8, 16, -1] }"
    >
      <template #item.schemaName="{ item }">
        <span class="text-truncate d-inline-block channel-schema">
          {{ item.schemaName || '-' }}
        </span>
      </template>
      <template #item.messageCount="{ item }">
        {{ item.messageCount.toLocaleString() }}
      </template>
    </v-data-table>

    <div class="d-flex align-center mt-3">
      <template v-if="export_progress">
        <div class="export-progress flex-grow-1 mr-3">
          <div class="d-flex align-center caption mb-1">
            <span>{{ export_status }}</span>
            <v-spacer />
            <span>{{ export_percentage }}%</span>
          </div>
          <v-progress-linear
            :value="export_percentage"
            height="6"
            rounded
            color="primary"
          />
          <div class="caption warning--text mt-1">
            Keep this page open. Leaving now will stop this export.
          </div>
        </div>
        <v-btn
          small
          text
          class="mr-2"
          @click="cancelExport"
        >
          Cancel
        </v-btn>
      </template>
      <v-spacer v-else />
      <v-btn
        small
        color="primary"
        :loading="Boolean(export_progress)"
        :disabled="Boolean(export_progress) || selected.length === 0"
        @click="saveCsv"
      >
        <v-icon small left>
          mdi-download
        </v-icon>
        Save as CSV
      </v-btn>
    </div>
  </div>
</template>

<script lang="ts">
import { saveAs } from 'file-saver'
import Vue, { PropType } from 'vue'

import {
  csvExportPercentage,
  csvExportStatusText,
  csvFileName,
  csvRangeLabel,
  csvSelectionLabel,
  filterChannelsBySearch,
  McapCsvExportController,
  McapVideoRecording,
  Mp4ExportRange,
} from '@/libs/mcap'
import { prettifySize } from '@/utils/helper_functions'

export default Vue.extend({
  name: 'McapCsvExport',
  props: {
    recording: { type: Object as PropType<McapVideoRecording>, required: true },
    clip: { type: Object as PropType<Mp4ExportRange | null>, default: null },
    name: { type: String, required: true },
  },
  data() {
    return {
      controller: null as McapCsvExportController | null,
      selected: [] as ReturnType<McapCsvExportController['getState']>['selected'],
      search: '',
      export_progress: null as ReturnType<McapCsvExportController['getState']>['exportProgress'],
    }
  },
  computed: {
    channels() {
      const all = this.recording.channels
      return filterChannelsBySearch(all, this.search)
    },
    headers(): { text: string, value: string }[] {
      return [
        { text: 'Topic', value: 'topic' },
        { text: 'Schema', value: 'schemaName' },
        { text: 'Encoding', value: 'messageEncoding' },
        { text: 'Messages', value: 'messageCount' },
      ]
    },
    selection_label(): string {
      return csvSelectionLabel(this.selected.length, this.recording.channels.length)
    },
    range_label(): string {
      return csvRangeLabel(this.clip)
    },
    export_percentage(): number {
      return csvExportPercentage(this.export_progress)
    },
    export_status(): string {
      return csvExportStatusText(this.export_progress, (kilobytes) => prettifySize(kilobytes))
    },
  },
  watch: {
    search(value: string) {
      this.controller?.setSearch(value)
    },
    selected(value: ReturnType<McapCsvExportController['getState']>['selected']) {
      this.controller?.setSelected(value)
    },
  },
  mounted() {
    this.controller = new McapCsvExportController(this.recording, this.clip, this.name, {
      onState: (state) => {
        this.selected = state.selected
        this.search = state.search
        this.export_progress = state.exportProgress
      },
      onBusy: (busy) => this.$emit('busy', busy),
      onError: (message) => this.$emit('error', message),
      onSaved: (blob, fileName) => saveAs(blob, fileName),
    })
    this.controller.mount()
  },
  beforeDestroy() {
    this.controller?.destroy()
  },
  methods: {
    selectAll(): void {
      this.controller?.selectAll()
    },
    selectNone(): void {
      this.controller?.selectNone()
    },
    selectDefaults(): void {
      this.controller?.selectDefaults()
    },
    async saveCsv(): Promise<void> {
      await this.controller?.saveCsv(csvFileName(this.name, this.clip))
    },
    cancelExport(): void {
      this.controller?.cancelExport()
    },
  },
})

</script>

<style scoped>
.csv-export {
  min-width: 0;
}

.channel-table {
  background: transparent;
}

.channel-search {
  max-width: 280px;
  min-width: 160px;
}

.channel-schema {
  max-width: min(220px, 40vw);
}

.export-progress {
  min-width: 0;
}
</style>
