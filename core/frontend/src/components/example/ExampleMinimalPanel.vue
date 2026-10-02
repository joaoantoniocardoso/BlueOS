<template>
  <v-card outlined>
    <v-card-title class="text-h6">
      Example minimal (developer)
    </v-card-title>
    <v-card-text>
      <div class="d-flex align-center flex-wrap">
        <v-slider
          v-model="targetLevel"
          class="flex-grow-1 mr-4"
          :max="100"
          :min="0"
          label="Level"
          hide-details
        />
        <v-btn
          color="primary"
          :disabled="sending"
          @click="applyLevel"
        >
          Set level
        </v-btn>
      </div>
      <div class="mt-4">
        Pump state:
        <span v-if="pumpLevel === null">waiting...</span>
        <span v-else>{{ pumpLevel }}%</span>
      </div>
      <div
        v-if="lastError"
        class="mt-2 error--text"
      >
        {{ lastError }}
      </div>
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
import Vue from 'vue'

import { sendCommand } from '@/libs/blueos-api/command'
import { pump, SetLevel } from '@/libs/blueos-api/services/example'
import type { Transport } from '@/libs/blueos-api/transport'
import { blueosApiMixin } from '@/mixins/blueosApi'

export default Vue.extend({
  name: 'ExampleMinimalPanel',
  mixins: [blueosApiMixin],
  props: {
    transport: {
      type: Object as () => Transport,
      required: true,
    },
  },
  data() {
    return {
      targetLevel: 0,
      pumpLevel: null as number | null,
      sending: false,
      lastError: '' as string,
    }
  },
  async created() {
    await this.blueosWatchState(
      this.transport,
      pump,
      (message) => {
        this.pumpLevel = message.level
      },
      (error) => {
        this.lastError = error instanceof Error ? error.message : String(error)
      },
    )
  },
  methods: {
    async applyLevel() {
      this.sending = true
      this.lastError = ''
      try {
        const ack = await sendCommand(this.transport, SetLevel, { level: this.targetLevel })
        if (!ack.accepted) {
          this.lastError = ack.reason || 'command rejected'
        }
      } catch (error) {
        this.lastError = error instanceof Error ? error.message : String(error)
      } finally {
        this.sending = false
      }
    },
  },
})
</script>
