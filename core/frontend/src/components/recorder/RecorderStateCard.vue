<template>
  <v-card outlined class="mb-4">
    <v-card-title class="text-subtitle-1">
      Recording state
    </v-card-title>
    <v-card-text>
      <v-row dense>
        <v-col cols="12" sm="6" md="4">
          <div class="text-caption grey--text">
            Session
          </div>
          <v-chip
            small
            :color="recordingState?.session_active ? 'error' : 'default'"
            :outlined="!recordingState?.session_active"
          >
            {{ recordingState?.session_active ? 'recording' : 'idle' }}
          </v-chip>
        </v-col>
        <v-col cols="12" sm="6" md="4">
          <div class="text-caption grey--text">
            Vehicle armed
          </div>
          <v-chip
            small
            :color="recordingState?.armed ? 'warning' : 'default'"
            outlined
          >
            {{ recordingState?.armed ? 'armed' : 'disarmed' }}
          </v-chip>
        </v-col>
        <v-col cols="12" sm="6" md="4">
          <div class="text-caption grey--text">
            MCAP size (session)
          </div>
          <div>{{ sessionSizeLabel }}</div>
          <div
            v-if="recordingState?.session_active"
            class="text-caption grey--text"
          >
            Duration is not available from recorder state.
          </div>
        </v-col>
      </v-row>
      <div v-if="recordingState?.current_file" class="mt-3">
        <div class="text-caption grey--text">
          Active file
        </div>
        <code class="text-body-2">{{ recordingState.current_file }}</code>
      </div>
      <div v-if="statusDetail" class="mt-2 text-caption">
        Service status: {{ statusDetail }}
      </div>
      <div class="mt-4">
        <div class="text-subtitle-2 mb-2">
          Video streams recording
        </div>
        <v-alert
          v-if="videoTopics.length === 0"
          type="info"
          dense
          outlined
        >
          No video topics are actively gated into the MCAP. External capture must be running per stream.
        </v-alert>
        <v-chip
          v-for="topic in videoTopics"
          :key="topic"
          class="ma-1"
          small
          color="primary"
        >
          {{ topic }}
        </v-chip>
      </div>
      <div class="mt-4">
        <div class="text-subtitle-2 mb-2">
          Backbone topics
        </div>
        <div class="text-body-2">
          {{ backboneTopicsHint }}
        </div>
      </div>
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
import type { RecordingState } from '@blueos-idl/messages'

import { formatRecordingBytes } from '@/components/recorder/format'

export default {
  name: 'RecorderStateCard',
  props: {
    recordingState: {
      type: Object as () => RecordingState | null,
      default: null,
    },
    statusDetail: {
      type: String,
      default: '',
    },
    policyMavlinkOnlyWhenArmed: {
      type: Boolean,
      default: true,
    },
  },
  computed: {
    sessionSizeLabel(): string {
      const bytes = this.recordingState?.session_bytes_written ?? 0
      return formatRecordingBytes(bytes)
    },
    videoTopics(): string[] {
      return this.recordingState?.recording_video_topics ?? []
    },
    backboneTopicsHint(): string {
      if (!this.recordingState?.session_active) {
        return 'Recording session is inactive; the Zenoh tap is not writing MCAP data.'
      }
      const mavlinkPart = this.policyMavlinkOnlyWhenArmed
        ? 'MAVLink topics while armed (when enabled in policy)'
        : 'MAVLink topics whenever the session is active'
      return 'While the session is active, non-video backbone traffic is recorded subject to policy. '
        + `${mavlinkPart}; video/* only for streams listed above; other keys are recorded when the session is on. `
        + 'A per-topic list is not exposed on the recorder state API.'
    },
  },
}
</script>
