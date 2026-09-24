<template>
  <v-card outlined class="mb-4">
    <v-card-title class="text-subtitle-1">
      Session controls
    </v-card-title>
    <v-card-text>
      <v-switch
        v-model="rotateIfActiveLocal"
        dense
        hide-details
        class="mt-0 mb-2"
        label="Rotate file if a session is already active (StartRecording)"
        :disabled="!serviceAlive"
      />
      <v-btn
        class="mr-2 mb-2"
        color="error"
        :disabled="!serviceAlive || commandPending"
        :loading="commandPending && pendingAction === 'start'"
        @click="$emit('start', rotateIfActiveLocal)"
      >
        <v-icon left>
          mdi-record-circle
        </v-icon>
        Start recording
      </v-btn>
      <v-btn
        color="secondary"
        class="mb-2"
        :disabled="!serviceAlive || commandPending"
        :loading="commandPending && pendingAction === 'stop'"
        @click="$emit('stop')"
      >
        <v-icon left>
          mdi-stop-circle
        </v-icon>
        Stop recording
      </v-btn>
      <v-alert
        v-if="lastAckMessage"
        :type="lastAckAccepted ? 'success' : 'warning'"
        dense
        class="mt-2 mb-0"
      >
        {{ lastAckMessage }}
      </v-alert>
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
export default {
  name: 'RecorderControlsCard',
  props: {
    serviceAlive: {
      type: Boolean,
      required: true,
    },
    commandPending: {
      type: Boolean,
      default: false,
    },
    pendingAction: {
      type: String as () => '' | 'start' | 'stop',
      default: '',
    },
    lastAckAccepted: {
      type: Boolean,
      default: true,
    },
    lastAckMessage: {
      type: String,
      default: '',
    },
  },
  data() {
    return {
      rotateIfActiveLocal: false,
    }
  },
}
</script>
