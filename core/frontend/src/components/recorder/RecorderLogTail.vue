<template>
  <v-card outlined>
    <v-card-title class="text-subtitle-1 d-flex align-center">
      Service log tail
      <v-spacer />
      <v-btn icon small :disabled="entries.length === 0" @click="clear">
        <v-icon>mdi-delete-outline</v-icon>
      </v-btn>
    </v-card-title>
    <v-card-subtitle>
      Live tracing output from the recorder service (D-13). Domain events are not published on a separate
      events stream.
    </v-card-subtitle>
    <v-card-text>
      <v-alert
        v-if="entries.length === 0"
        type="info"
        dense
        outlined
      >
        No log lines yet. Start the recorder service or trigger recording to see output here.
      </v-alert>
      <v-simple-table v-else dense class="recorder-log-table">
        <thead>
          <tr>
            <th class="text-left">
              Level
            </th>
            <th class="text-left">
              Message
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(entry, index) in visibleEntries" :key="`${index}-${entry.message}`">
            <td class="text-no-wrap">
              <v-chip x-small :color="levelColor(entry.level)">
                {{ levelLabel(entry.level) }}
              </v-chip>
            </td>
            <td class="text-body-2">
              {{ entry.message }}
            </td>
          </tr>
        </tbody>
      </v-simple-table>
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
import type { Log } from '@blueos-idl/messages'

import { logLevelLabel } from '@/components/recorder/format'

const MAX_ENTRIES = 80

export default {
  name: 'RecorderLogTail',
  props: {
    entries: {
      type: Array as () => Log[],
      required: true,
    },
  },
  computed: {
    visibleEntries(): Log[] {
      return this.entries.slice(-MAX_ENTRIES)
    },
  },
  methods: {
    levelLabel(level: number): string {
      return logLevelLabel(level)
    },
    levelColor(level: number): string {
      if (level >= 50) {
        return 'error'
      }
      if (level >= 40) {
        return 'warning'
      }
      return 'primary'
    },
    clear() {
      this.$emit('clear')
    },
  },
}
</script>

<style scoped>
.recorder-log-table tbody tr td {
  vertical-align: top;
}
</style>
