<template>
  <v-sheet rounded class="d-flex align-center flex-wrap mb-4 px-2 py-2">
    <v-btn
      v-tooltip="startTooltip"
      color="error"
      class="mr-2"
      :disabled="!controls.canStart"
      :loading="pending === 'start'"
      @click="run('start')"
    >
      <v-icon left>
        mdi-record-circle
      </v-icon>
      Record
    </v-btn>
    <v-btn
      v-tooltip="'Stop the active recording'"
      color="primary"
      class="mr-4"
      :disabled="!controls.canStop"
      :loading="pending === 'stop'"
      @click="run('stop')"
    >
      <v-icon left>
        mdi-stop-circle
      </v-icon>
      Stop
    </v-btn>
    <v-switch
      v-model="rotateIfActive"
      v-tooltip="'Start a new file when a recording is already active'"
      label="Rotate file if a session is already active"
      dense
      hide-details
      class="mt-0"
      :disabled="!serviceRunning"
    />
    <v-spacer />
    <v-chip
      v-if="recording"
      small
      :color="recording.session_active ? 'error' : undefined"
    >
      {{ recording.session_active ? 'Recording' : 'Idle' }}
    </v-chip>
    <v-alert
      v-if="notice"
      :type="notice.type"
      dense
      class="mt-2 mb-0 flex-grow-1"
      style="flex-basis: 100%"
    >
      {{ notice.message }}
    </v-alert>
  </v-sheet>
</template>

<script lang="ts">
import type { RecordingState } from '@blueos-idl/messages'
import Vue, { PropType } from 'vue'

import type { RecorderClient } from '@/libs/recorder/client'
import {
  runSessionAction, type SessionAction, sessionControls,
  type SessionNotice,
} from '@/libs/recorder/session-controls'

export default Vue.extend({
  name: 'RecordsSessionControls',
  props: {
    recorder: {
      type: Object as PropType<RecorderClient | null>,
      default: null,
    },
    recording: {
      type: Object as PropType<RecordingState | null>,
      default: null,
    },
    serviceRunning: {
      type: Boolean,
      required: true,
    },
  },
  data() {
    return {
      rotateIfActive: false,
      pending: null as SessionAction | null,
      notice: null as SessionNotice | null,
    }
  },
  computed: {
    controls() {
      const reachable = this.serviceRunning && this.recorder !== null
      return sessionControls(this.recording, reachable, this.pending, this.rotateIfActive)
    },
    startTooltip(): string {
      return this.recording?.session_active && !this.rotateIfActive
        ? 'A recording is already active'
        : 'Start recording'
    },
  },
  methods: {
    async run(action: SessionAction): Promise<void> {
      if (!this.recorder) {
        return
      }
      this.pending = action
      this.notice = await runSessionAction(this.recorder, action, this.rotateIfActive)
      this.pending = null
    },
  },
})
</script>
