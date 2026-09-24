<template>
  <v-container>
    <v-row>
      <v-col cols="12">
        <v-card>
          <RecorderServiceHeader
            title="Recorder"
            :service-alive="serviceAlive"
            :service-version="serviceVersion"
          />
          <v-card-subtitle>
            Zenoh IDL control plane for the Rust recorder service (<code>recorder</code>).
          </v-card-subtitle>
          <v-card-text>
            <RecorderStateCard
              :recording-state="recordingState"
              :status-detail="statusDetail"
              :policy-mavlink-only-when-armed="policyDraft.record_mavlink_only_when_armed"
            />
            <RecorderControlsCard
              :service-alive="serviceAlive"
              :command-pending="commandPending"
              :pending-action="pendingAction"
              :last-ack-accepted="lastCommandAckAccepted"
              :last-ack-message="lastCommandAckMessage"
              @start="startRecording"
              @stop="stopRecording"
            />
            <RecorderPolicyCard
              :service-alive="serviceAlive"
              :policy="policyDraft"
              :restart-fields="restartFields"
              :saving="policySaving"
              :policy-ack-accepted="lastPolicyAckAccepted"
              :policy-ack-message="lastPolicyAckMessage"
              @save-settings="saveSettings"
              @set-policy="setPolicy"
            />
            <RecorderRecordingsLink />
            <RecorderLogTail
              :entries="logEntries"
              @clear="clearLogs"
            />
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import type {
  Log,
  RecordingPolicy,
  RecordingState,
  SettingsEnvelope,
} from '@blueos-idl/messages'

import RecorderControlsCard from '@/components/recorder/RecorderControlsCard.vue'
import RecorderLogTail from '@/components/recorder/RecorderLogTail.vue'
import RecorderPolicyCard from '@/components/recorder/RecorderPolicyCard.vue'
import RecorderRecordingsLink from '@/components/recorder/RecorderRecordingsLink.vue'
import RecorderServiceHeader from '@/components/recorder/RecorderServiceHeader.vue'
import RecorderStateCard from '@/components/recorder/RecorderStateCard.vue'
import { blueosApiMixin } from '@/libs/blueos-api/vue2'

const SERVICE = 'recorder'
const RECORDING_STATE_SCHEMA = 'blueos_recorder_msgs/msg/RecordingState'
const SET_POLICY_SCHEMA = 'blueos_recorder_msgs/msg/SetPolicyCommand'
const START_RECORDING_SCHEMA = 'blueos_recorder_msgs/msg/StartRecordingCommand'
const STOP_RECORDING_SCHEMA = 'blueos_recorder_msgs/msg/StopRecordingCommand'
const SERVICE_STATUS_SCHEMA = 'blueos_msgs/msg/ServiceStatus'
const SETTINGS_COMMAND_SCHEMA = 'blueos_msgs/msg/SettingsEnvelope'

type RecorderSettingsDocument = RecordingPolicy & {
  VERSION: number,
}

function defaultPolicy(): RecordingPolicy {
  return {
    record_mavlink_only_when_armed: true,
    auto_start_recording: true,
  }
}

export default {
  name: 'RecorderView',
  components: {
    RecorderControlsCard,
    RecorderLogTail,
    RecorderPolicyCard,
    RecorderRecordingsLink,
    RecorderServiceHeader,
    RecorderStateCard,
  },
  mixins: [blueosApiMixin],
  data() {
    return {
      serviceAlive: false,
      serviceVersion: '',
      recordingState: null as RecordingState | null,
      statusDetail: '',
      policyDraft: defaultPolicy(),
      settingsVersion: 1,
      restartFields: [] as string[],
      commandPending: false,
      pendingAction: '' as '' | 'start' | 'stop',
      lastCommandAckAccepted: true,
      lastCommandAckMessage: '',
      policySaving: false,
      lastPolicyAckAccepted: true,
      lastPolicyAckMessage: '',
      logEntries: [] as Log[],
    }
  },
  mounted() {
    this.blueosWatchServiceAlive(SERVICE, (alive) => {
      this.serviceAlive = alive
    })
    this.blueosGetServiceInfo(SERVICE)
      .then((info) => {
        this.serviceVersion = info?.version ?? ''
      })
      .catch(() => {
        this.serviceVersion = ''
      })
    this.blueosWatchState(SERVICE, 'recording', RECORDING_STATE_SCHEMA, (state) => {
      this.recordingState = state
    })
    this.blueosWatchState(SERVICE, 'status', SERVICE_STATUS_SCHEMA, (status) => {
      this.statusDetail = status?.detail ?? ''
    })
    this.blueosWatchSettings(SERVICE, (envelope: SettingsEnvelope) => {
      this.applySettingsEnvelope(envelope)
    })
    this.blueosWatchLogs(SERVICE, (entry) => {
      this.logEntries.push(entry)
    })
  },
  methods: {
    applySettingsEnvelope(envelope: SettingsEnvelope) {
      try {
        const document = JSON.parse(envelope.document_json) as RecorderSettingsDocument
        this.settingsVersion = document.VERSION ?? 1
        this.policyDraft = {
          record_mavlink_only_when_armed: document.record_mavlink_only_when_armed
            ?? this.policyDraft.record_mavlink_only_when_armed,
          auto_start_recording: document.auto_start_recording ?? this.policyDraft.auto_start_recording,
        }
      } catch {
        // ignore malformed settings snapshot
      }
      this.restartFields = (envelope.fields ?? [])
        .filter((field) => field.restart_required)
        .map((field) => field.path)
    },
    commandAckMessage(accepted: boolean, reason: string, label: string): string {
      if (accepted) {
        return `${label} accepted.`
      }
      const detail = reason?.trim()
      return detail ? `${label} rejected: ${detail}` : `${label} rejected.`
    },
    async startRecording(rotateIfActive: boolean) {
      this.commandPending = true
      this.pendingAction = 'start'
      try {
        const ack = await this.blueosSendCommand(
          SERVICE,
          'StartRecording',
          START_RECORDING_SCHEMA,
          { rotate_if_active: rotateIfActive },
        )
        this.lastCommandAckAccepted = ack.accepted
        this.lastCommandAckMessage = this.commandAckMessage(ack.accepted, ack.reason, 'StartRecording')
      } catch (error) {
        this.lastCommandAckAccepted = false
        this.lastCommandAckMessage = `StartRecording failed: ${String(error)}`
      } finally {
        this.commandPending = false
        this.pendingAction = ''
      }
    },
    async stopRecording() {
      this.commandPending = true
      this.pendingAction = 'stop'
      try {
        const ack = await this.blueosSendCommand(
          SERVICE,
          'StopRecording',
          STOP_RECORDING_SCHEMA,
          { reserved: 0 },
        )
        this.lastCommandAckAccepted = ack.accepted
        this.lastCommandAckMessage = this.commandAckMessage(ack.accepted, ack.reason, 'StopRecording')
      } catch (error) {
        this.lastCommandAckAccepted = false
        this.lastCommandAckMessage = `StopRecording failed: ${String(error)}`
      } finally {
        this.commandPending = false
        this.pendingAction = ''
      }
    },
    async setPolicy(policy: RecordingPolicy) {
      this.policySaving = true
      try {
        const ack = await this.blueosSendCommand(SERVICE, 'SetPolicy', SET_POLICY_SCHEMA, { policy })
        this.lastPolicyAckAccepted = ack.accepted
        this.lastPolicyAckMessage = this.commandAckMessage(ack.accepted, ack.reason, 'SetPolicy')
        if (ack.accepted) {
          this.policyDraft = { ...policy }
        }
      } catch (error) {
        this.lastPolicyAckAccepted = false
        this.lastPolicyAckMessage = `SetPolicy failed: ${String(error)}`
      } finally {
        this.policySaving = false
      }
    },
    async saveSettings(document: RecordingPolicy) {
      this.policySaving = true
      const documentJson = JSON.stringify({
        VERSION: this.settingsVersion,
        record_mavlink_only_when_armed: document.record_mavlink_only_when_armed,
        auto_start_recording: document.auto_start_recording,
      })
      try {
        const ack = await this.blueosSendCommand(SERVICE, 'UpdateSettings', SETTINGS_COMMAND_SCHEMA, {
          document_json: documentJson,
          fields: [],
        })
        this.lastPolicyAckAccepted = ack.accepted
        this.lastPolicyAckMessage = this.commandAckMessage(ack.accepted, ack.reason, 'UpdateSettings')
      } catch (error) {
        this.lastPolicyAckAccepted = false
        this.lastPolicyAckMessage = `UpdateSettings failed: ${String(error)}`
      } finally {
        this.policySaving = false
      }
    },
    clearLogs() {
      this.logEntries = []
    },
  },
}
</script>
