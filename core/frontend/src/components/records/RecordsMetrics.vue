<template>
  <v-sheet v-if="metrics" rounded class="d-flex align-center flex-wrap mb-4 px-2 py-2">
    <v-simple-table dense class="records-metrics-lanes transparent">
      <thead>
        <tr>
          <th>Lane</th>
          <th class="text-right">
            Bytes written
          </th>
          <th class="text-right">
            Samples
          </th>
          <th class="text-right">
            Dropped
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="lane in metrics.lanes" :key="lane.lane">
          <td>{{ lane.lane }}</td>
          <td class="text-right">
            {{ prettifySize(lane.bytesWritten / 1024) }}
          </td>
          <td class="text-right">
            {{ lane.samplesWritten }}
          </td>
          <td class="text-right" :class="{ 'error--text': lane.samplesDropped > 0 }">
            {{ lane.samplesDropped }}
          </td>
        </tr>
      </tbody>
    </v-simple-table>
    <v-spacer />
    <div class="mr-4">
      <v-chip
        v-tooltip="'Mean time the Recorder takes to handle one message from its Inbox'"
        small
        class="mr-2"
      >
        Inbox step {{ stepLabel }}
      </v-chip>
      <v-chip
        v-tooltip="'Messages waiting in the Recorder Inbox'"
        small
      >
        Inbox depth {{ metrics.inboxDepth ?? 'N/A' }}
      </v-chip>
    </div>
  </v-sheet>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import type { RecorderMetrics } from '@/libs/recorder/view-logic'
import { prettifySize } from '@/utils/helper_functions'

export default Vue.extend({
  name: 'RecordsMetrics',
  props: {
    metrics: {
      type: Object as PropType<RecorderMetrics | null>,
      default: null,
    },
  },
  computed: {
    stepLabel(): string {
      const seconds = this.metrics?.inboxStepSeconds
      return seconds === null || seconds === undefined ? 'N/A' : `${(seconds * 1000).toFixed(3)} ms`
    },
  },
  methods: {
    prettifySize,
  },
})
</script>

<style scoped>
.records-metrics-lanes {
  min-width: 320px;
}
</style>
