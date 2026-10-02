<template>
  <v-dialog
    :value="value"
    max-width="1200"
    scrollable
    @input="$emit('input', $event)"
  >
    <v-card>
      <v-app-bar dense>
        <v-spacer />
        <v-toolbar-title>
          Logs for {{ extensionName || extensionIdentifier }}
        </v-toolbar-title>
        <v-spacer />
        <v-checkbox
          v-model="follow_logs"
          label="Follow Logs"
          hide-details
        />
        <v-btn
          class="ml-3"
          icon
          @click="downloadCurrentLog"
        >
          <v-icon>mdi-download</v-icon>
        </v-btn>
        <v-btn
          class="ml-2"
          icon
          @click="closeModal"
        >
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-app-bar>
      <v-sheet>
        <v-card-text
          ref="logContainer"
          style="max-height: calc(600px - 48px); overflow-y: auto;"
        >
          <v-alert
            v-if="modal_error"
            type="error"
            dismissible
            class="ma-2"
            @input="modal_error = null"
          >
            <div class="font-weight-bold mb-1">
              Error
            </div>
            <div>{{ modal_error }}</div>
          </v-alert>
          <div
            v-if="modal_messages.length === 0 && !modal_error"
            class="text-center text--secondary py-8"
          >
            <v-progress-circular
              v-if="requesting_logs"
              indeterminate
              color="primary"
            />
            <div v-else>
              No logs received yet
            </div>
          </div>
          <div
            v-if="modal_messages.length > 0"
            class="logs-container"
            style="padding: 16px; font-family: monospace; font-size: 12px;"
          >
            <div
              v-for="(msg, index) in modal_messages"
              :key="`log-${index}`"
              class="mb-1"
            >
              <!-- eslint-disable -->
              <div
                v-if="!isMessageEmpty(msg)"
                class="log-line"
                v-html="formatLogMessage(msg)"
              />
              <!-- eslint-enable -->
              <br v-else />
            </div>
          </div>
        </v-card-text>
      </v-sheet>
    </v-card>
  </v-dialog>
</template>

<script lang="ts">
import AnsiUp from 'ansi_up'
import { saveAs } from 'file-saver'
import Vue from 'vue'

import { extensionLogKey } from '@/libs/blueos-api/keys'
import {
  requestExtensionLogs,
  watchExtensionLogs,
} from '@/libs/blueos-api/logs'
import type { Subscription, Transport } from '@/libs/blueos-api/transport'
import zenohTransport from '@/libs/blueos-api/zenoh-transport'
import zenoh from '@/libs/zenoh'
import { blueosApiMixin } from '@/mixins/blueosApi'

interface LogMessage {
  message: string
}

const ansi = new AnsiUp()
const KRAKEN_SERVICE = 'kraken'
const LOGS_QUERY_TIMEOUT_MS = 30000
const MAX_LOG_MESSAGES = 5000
const BUFFER_FLUSH_INTERVAL_MS = 16

export default Vue.extend({
  name: 'ExtensionLogsModal',
  mixins: [blueosApiMixin],
  props: {
    value: {
      type: Boolean,
      required: true,
    },
    extensionIdentifier: {
      type: String,
      required: true,
    },
    extensionName: {
      type: String,
      default: '',
    },
  },
  data() {
    return {
      transport: null as Transport | null,
      modal_messages: [] as LogMessage[],
      log_subscription: null as Subscription | null,
      current_modal_topic: '',
      modal_error: null as string | null,
      requesting_logs: false,
      query_timeout: LOGS_QUERY_TIMEOUT_MS,
      follow_logs: true,
      scroll_pending: false,
      message_buffer: [] as LogMessage[],
      buffer_flush_timer: null as number | null,
    }
  },
  watch: {
    value(val: boolean) {
      if (val) {
        this.openModal()
      } else {
        this.closeModal()
      }
    },
    follow_logs(val: boolean) {
      if (val) {
        this.scrollToBottom()
      }
    },
  },
  async created() {
    const session = await zenoh.getSession()
    this.transport = zenohTransport(session)
  },
  beforeDestroy() {
    this.cleanup()
  },
  methods: {
    async openModal() {
      this.modal_messages = []
      this.modal_error = null

      if (!this.transport) {
        this.setErrorAndStop('Zenoh transport is not ready')
        return
      }

      await this.requestHistoricalLogsForExtension(this.extensionIdentifier)

      if (!this.current_modal_topic) {
        await this.setupLogWatcher(extensionLogKey(KRAKEN_SERVICE, this.extensionIdentifier))
      }
    },
    closeModal() {
      this.cleanup()
      this.$emit('input', false)
    },
    cleanup() {
      if (this.log_subscription) {
        this.log_subscription.close().catch(() => undefined)
        this.log_subscription = null
      }
      if (this.buffer_flush_timer) {
        clearTimeout(this.buffer_flush_timer)
        this.buffer_flush_timer = null
      }
      this.flushMessageBuffer()
      this.modal_messages = []
      this.current_modal_topic = ''
      this.modal_error = null
      this.scroll_pending = false
    },
    flushMessageBuffer() {
      if (this.message_buffer.length === 0) {
        return
      }

      const batch = this.message_buffer.splice(0)
      this.modal_messages.push(...batch)

      if (this.modal_messages.length > MAX_LOG_MESSAGES) {
        const removeCount = this.modal_messages.length - MAX_LOG_MESSAGES
        this.modal_messages.splice(0, removeCount)
      }

      this.buffer_flush_timer = null
      this.scheduleScroll()
    },
    async setupLogWatcher(topic: string) {
      if (!this.transport || this.current_modal_topic === topic && this.log_subscription) {
        return
      }

      if (this.log_subscription) {
        await this.log_subscription.close()
        this.log_subscription = null
      }

      this.current_modal_topic = topic
      const identifier = this.extensionIdentifier
      this.log_subscription = await this.blueosTrackSubscription(watchExtensionLogs(
        this.transport,
        KRAKEN_SERVICE,
        identifier,
        {
          onLog: (entry) => {
            this.message_buffer.push({ message: entry.message })
            if (!this.buffer_flush_timer) {
              this.buffer_flush_timer = window.setTimeout(() => {
                this.flushMessageBuffer()
              }, BUFFER_FLUSH_INTERVAL_MS)
            }
          },
          onError: (error) => {
            const errorMessage = error instanceof Error ? error.message : String(error)
            this.setErrorAndStop(`Error decoding extension log: ${errorMessage}`)
          },
        },
      ))
    },
    async requestHistoricalLogsForExtension(identifier: string) {
      if (!this.transport) {
        return
      }
      this.requesting_logs = true
      this.modal_error = null
      try {
        const response = await Promise.race([
          requestExtensionLogs(this.transport, KRAKEN_SERVICE, identifier),
          new Promise<null>((resolve) => {
            window.setTimeout(() => resolve(null), this.query_timeout)
          }),
        ])

        if (!response) {
          this.setErrorAndStop('No response from logs service (timeout or connection issue)')
          return
        }

        if (response.error) {
          const errorSuffix = response.error_type ? ` (${response.error_type})` : ''
          this.setErrorAndStop(`Error from queryable: ${response.error}${errorSuffix}`)
          return
        }

        if (Array.isArray(response.messages)) {
          this.modal_messages = response.messages.map((line) => ({
            message: line.message != null ? String(line.message) : '',
          }))
          this.scrollToBottom()
        }

        if (response.topic && response.topic !== this.current_modal_topic) {
          await this.setupLogWatcher(response.topic)
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error)
        this.setErrorAndStop(`Error requesting historical logs: ${errorMessage}`)
      } finally {
        this.requesting_logs = false
      }
    },
    isMessageEmpty(msg: LogMessage): boolean {
      return String(msg?.message || '').trim().length === 0
    },
    extractLogMessage(msg: LogMessage): string {
      return String(msg?.message || '')
    },
    formatLogMessage(msg: LogMessage): string {
      const message = this.extractLogMessage(msg)
      return ansi.ansi_to_html(message)
    },
    scheduleScroll() {
      if (!this.follow_logs || this.scroll_pending) {
        return
      }
      this.scroll_pending = true
      requestAnimationFrame(() => {
        this.scroll_pending = false
        const container = this.$refs.logContainer as HTMLElement
        if (container) {
          container.scrollTop = container.scrollHeight
        }
      })
    },
    scrollToBottom() {
      if (!this.follow_logs) {
        return
      }
      this.scroll_pending = false
      this.$nextTick(() => {
        const container = this.$refs.logContainer as HTMLElement
        if (!container) {
          return
        }
        container.scrollTop = container.scrollHeight
      })
    },
    setErrorAndStop(message: string) {
      this.modal_error = message
      this.requesting_logs = false
    },
    downloadCurrentLog() {
      const logContent = this.modal_messages
        .map((msg) => String(msg.message || ''))
        .join('\n')
      const baseName = this.extensionName || this.extensionIdentifier
      const file = new File([logContent], `${baseName}.log`, { type: 'text/plain' })
      saveAs(file)
    },
  },
})
</script>
