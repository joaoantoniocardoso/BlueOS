<template>
  <v-container>
    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-title class="d-flex align-center">
            Example Rust service
            <v-spacer />
            <v-chip :color="serviceAlive ? 'success' : 'error'" small>
              {{ serviceAlive ? 'live' : 'offline' }}
            </v-chip>
          </v-card-title>
          <v-card-subtitle>
            Teaching example for blueos-api (pirate mode). Service name: <code>example</code>.
          </v-card-subtitle>
          <v-card-text>
            <v-alert v-if="restartFields.length > 0" type="warning" dense class="mb-4">
              Restart required after changing: {{ restartFields.join(', ') }}
            </v-alert>
            <div class="mb-4">
              <div class="text-subtitle-1">
                Pump level: {{ pumpState?.level ?? '—' }} / {{ pumpState?.max_level ?? '—' }}
              </div>
              <div class="text-caption">
                Self-test:
                {{ selfTestLabel }}
              </div>
              <div v-if="statusDetail" class="text-caption">
                Status: {{ statusDetail }}
              </div>
            </div>
            <v-slider
              v-model="levelDraft"
              :max="pumpState?.max_level ?? 100"
              :disabled="!serviceAlive || pumpState?.self_test_active"
              label="Target level"
              class="mb-2"
            />
            <v-btn
              class="mr-2"
              color="primary"
              :disabled="!serviceAlive || pumpState?.self_test_active"
              @click="applyLevel"
            >
              Set level
            </v-btn>
            <v-btn
              class="mr-2"
              color="secondary"
              :disabled="!serviceAlive || pumpState?.self_test_active"
              @click="startSelfTest"
            >
              Start self-test
            </v-btn>
            <v-btn
              color="warning"
              :disabled="!serviceAlive || !pumpState?.self_test_active"
              @click="cancelSelfTest"
            >
              Cancel self-test
            </v-btn>
            <v-divider class="my-4" />
            <div class="text-subtitle-2 mb-2">
              Jobs
            </div>
            <v-simple-table dense>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Name</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="job in jobsList" :key="job.job_id">
                  <td>{{ job.job_id }}</td>
                  <td>{{ job.name || '—' }}</td>
                  <td>{{ jobStatusLabel(job.status) }}</td>
                </tr>
              </tbody>
            </v-simple-table>
            <v-divider class="my-4" />
            <v-text-field
              v-model.number="settingsDraft.max_level"
              type="number"
              label="max_level (runtime)"
              dense
              class="mb-2"
            />
            <v-text-field
              v-model.number="settingsDraft.self_test_timeout_seconds"
              type="number"
              label="self_test_timeout_seconds (runtime)"
              dense
              class="mb-2"
            />
            <v-text-field
              v-model="settingsDraft.device_model"
              label="device_model (restart required)"
              dense
              class="mb-2"
            />
            <v-btn color="primary" :disabled="!serviceAlive" @click="applySettings">
              Update settings
            </v-btn>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import type { JobList, PumpState, SettingsEnvelope } from '@blueos-idl/messages'

import { blueosApiMixin } from '@/libs/blueos-api/vue2'

const SERVICE = 'example'
const PUMP_STATE_SCHEMA = 'blueos_example_msgs/msg/PumpState'
const SET_LEVEL_SCHEMA = 'blueos_example_msgs/msg/SetLevelRequest'
const EMPTY_REQUEST_SCHEMA = 'blueos_example_msgs/msg/EmptyRequest'
const STATUS_SCHEMA = 'blueos_msgs/msg/ServiceStatus'
const JOB_STATUS_LABELS: Record<number, string> = {
  0: 'queued',
  1: 'running',
  2: 'cancelling',
  3: 'succeeded',
  4: 'failed',
  5: 'cancelled',
}

const SELF_TEST_PHASE_LABELS: Record<number, string> = {
  0: 'idle',
  1: 'running',
  2: 'passed',
  3: 'failed',
  4: 'cancelled',
}

export default {
  name: 'ExampleServiceView',
  mixins: [blueosApiMixin],
  data() {
    return {
      serviceAlive: false,
      pumpState: null as PumpState | null,
      statusDetail: '',
      jobsList: [] as JobList['jobs'],
      restartFields: [] as string[],
      levelDraft: 0,
      settingsDraft: {
        VERSION: 1,
        max_level: 100,
        self_test_timeout_seconds: 30,
        device_model: 'simulated-pump-v1',
      },
    }
  },
  computed: {
    selfTestLabel(): string {
      const phase = this.pumpState?.self_test_phase ?? 0
      return SELF_TEST_PHASE_LABELS[phase] ?? `phase ${phase}`
    },
  },
  mounted() {
    this.blueosWatchServiceAlive(SERVICE, (alive) => {
      this.serviceAlive = alive
    })
    this.blueosWatchState(SERVICE, 'pump', PUMP_STATE_SCHEMA, (state) => {
      this.pumpState = state
      if (state?.level !== undefined) {
        this.levelDraft = state.level
      }
    })
    this.blueosWatchState(SERVICE, 'status', STATUS_SCHEMA, (status) => {
      this.statusDetail = status?.detail ?? ''
    })
    this.blueosWatchJobs(SERVICE, (jobs) => {
      this.jobsList = jobs?.jobs ?? []
    })
    this.blueosWatchSettings(SERVICE, (envelope: SettingsEnvelope) => {
      try {
        const document = JSON.parse(envelope.document_json) as typeof this.settingsDraft
        this.settingsDraft = { ...this.settingsDraft, ...document }
      } catch {
        // ignore malformed settings snapshot
      }
      this.restartFields = (envelope.fields ?? [])
        .filter((field) => field.restart_required)
        .map((field) => field.path)
    })
  },
  methods: {
    jobStatusLabel(status: number): string {
      return JOB_STATUS_LABELS[status] ?? String(status)
    },
    async applyLevel() {
      await this.blueosSendCommand(SERVICE, 'SetLevel', SET_LEVEL_SCHEMA, {
        level: this.levelDraft,
      })
    },
    async startSelfTest() {
      await this.blueosSendCommand(SERVICE, 'StartSelfTest', EMPTY_REQUEST_SCHEMA, { padding: 0 })
    },
    async cancelSelfTest() {
      await this.blueosSendCommand(SERVICE, 'CancelSelfTest', EMPTY_REQUEST_SCHEMA, { padding: 0 })
    },
    async applySettings() {
      const documentJson = JSON.stringify(this.settingsDraft)
      await this.blueosSendCommand(SERVICE, 'UpdateSettings', SETTINGS_SCHEMA, {
        document_json: documentJson,
        fields: [],
      })
    },
  },
}
</script>
