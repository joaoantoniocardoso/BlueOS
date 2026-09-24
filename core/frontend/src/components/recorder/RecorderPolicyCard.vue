<template>
  <v-card outlined class="mb-4">
    <v-card-title class="text-subtitle-1">
      Recording policy
    </v-card-title>
    <v-card-text>
      <v-alert
        v-if="restartFields.length > 0"
        type="warning"
        dense
        class="mb-3"
      >
        Restart required after changing: {{ restartFields.join(', ') }}
      </v-alert>
      <v-switch
        v-model="draft.record_mavlink_only_when_armed"
        dense
        hide-details
        class="mb-2"
        label="Record MAVLink only while armed"
        :disabled="!serviceAlive"
      />
      <v-switch
        v-model="draft.auto_start_recording"
        dense
        hide-details
        class="mb-4"
        label="Auto-start recording when the service starts"
        :disabled="!serviceAlive"
      />
      <v-alert type="info" dense outlined class="mb-3">
        Topic filters beyond MAVLink gating and per-stream video capture are not available in the
        recorder policy API.
      </v-alert>
      <v-btn
        color="primary"
        class="mr-2"
        :disabled="!serviceAlive || saving"
        :loading="saving"
        @click="applyPersisted"
      >
        Save settings
      </v-btn>
      <v-btn
        color="secondary"
        outlined
        :disabled="!serviceAlive || saving"
        :loading="saving"
        @click="applyLivePolicy"
      >
        Apply policy (SetPolicy)
      </v-btn>
      <v-alert
        v-if="policyAckMessage"
        :type="policyAckAccepted ? 'success' : 'warning'"
        dense
        class="mt-3 mb-0"
      >
        {{ policyAckMessage }}
      </v-alert>
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
import type { RecordingPolicy } from '@blueos-idl/messages'

function defaultPolicy(): RecordingPolicy {
  return {
    record_mavlink_only_when_armed: true,
    auto_start_recording: true,
  }
}

export default {
  name: 'RecorderPolicyCard',
  props: {
    serviceAlive: {
      type: Boolean,
      required: true,
    },
    policy: {
      type: Object as () => RecordingPolicy,
      default: defaultPolicy,
    },
    restartFields: {
      type: Array as () => string[],
      default: () => [],
    },
    saving: {
      type: Boolean,
      default: false,
    },
    policyAckAccepted: {
      type: Boolean,
      default: true,
    },
    policyAckMessage: {
      type: String,
      default: '',
    },
  },
  data() {
    return {
      draft: defaultPolicy(),
    }
  },
  watch: {
    policy: {
      immediate: true,
      handler(policy: RecordingPolicy) {
        this.draft = { ...policy }
      },
    },
  },
  methods: {
    applyPersisted() {
      this.$emit('save-settings', { ...this.draft })
    },
    applyLivePolicy() {
      this.$emit('set-policy', { ...this.draft })
    },
  },
}
</script>
