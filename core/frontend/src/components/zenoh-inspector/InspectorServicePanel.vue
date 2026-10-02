<template>
  <v-card
    outlined
    class="mt-4"
  >
    <v-card-title class="subtitle-1">
      Service: {{ selectedService }}
    </v-card-title>
    <v-card-text>
      <div v-if="!serviceInfo" class="caption grey--text">
        Loading service info…
      </div>
      <template v-else>
        <div class="mb-2">
          <div><strong>Version:</strong> {{ serviceInfo.version }}</div>
          <div v-if="serviceDescription">
            <strong>Description:</strong> {{ serviceDescription }}
          </div>
        </div>
        <div
          v-for="kind in endpointKinds"
          :key="kind"
          class="mb-4"
        >
          <div class="subtitle-2 mb-2 text-capitalize">
            {{ kind }}
          </div>
          <v-expansion-panels flat>
            <v-expansion-panel
              v-for="endpoint in endpointsForKind(kind)"
              :key="endpoint.key"
            >
              <v-expansion-panel-header>
                {{ endpoint.name }}
                <span class="caption grey--text ml-2">{{ endpoint.key }}</span>
              </v-expansion-panel-header>
              <v-expansion-panel-content>
                <div
                  v-if="isRequestEndpoint(endpoint)"
                  class="mb-3"
                >
                  <v-textarea
                    :value="requestText"
                    label="Request JSON"
                    outlined
                    dense
                    auto-grow
                    rows="4"
                    hide-details
                    class="mb-2"
                    @input="$emit('request-text', $event)"
                  />
                  <div class="d-flex flex-wrap">
                    <v-btn
                      v-if="endpoint.request_schema"
                      small
                      class="mr-2 mb-2"
                      @click="$emit('fill-default', endpoint.request_schema)"
                    >
                      Fill defaults
                    </v-btn>
                    <v-btn
                      small
                      color="primary"
                      class="mb-2"
                      :loading="sending"
                      @click="$emit('send-request', endpoint)"
                    >
                      Send
                    </v-btn>
                  </div>
                </div>
                <div class="caption grey--text">
                  Request schema: {{ endpoint.request_schema || 'none' }}
                </div>
                <div class="caption grey--text">
                  Response schema: {{ endpoint.response_schema || 'none' }}
                </div>
              </v-expansion-panel-content>
            </v-expansion-panel>
          </v-expansion-panels>
        </div>
      </template>

      <v-divider class="my-4" />

      <div class="subtitle-2 mb-2">
        Raw query
      </div>
      <v-text-field
        v-model="rawQueryKey"
        label="Key"
        outlined
        dense
        hide-details
        class="mb-2"
      />
      <v-textarea
        v-model="rawQueryPayload"
        label="Payload (JSON or text)"
        outlined
        dense
        auto-grow
        rows="3"
        hide-details
        class="mb-2"
      />
      <v-btn
        small
        color="primary"
        :loading="sending"
        @click="submitRawQuery"
      >
        Query
      </v-btn>

      <inspector-request-result-view
        class="mt-4"
        :result="lastRequestResult"
      />
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
import type { EndpointInfo, ServiceInfo } from '@blueos-idl/messages'
import Vue, { PropType } from 'vue'

import type { LastRequestResult } from '@/libs/zenoh-inspector/inspector-controller'

import InspectorRequestResultView from './InspectorRequestResultView.vue'

const REQUEST_ENDPOINT_KINDS = new Set(['command', 'query', 'io_query'])

export default Vue.extend({
  name: 'InspectorServicePanel',
  components: {
    InspectorRequestResultView,
  },
  props: {
    selectedService: {
      type: String,
      required: true,
    },
    serviceInfo: {
      type: Object as PropType<ServiceInfo | null>,
      default: null,
    },
    serviceEndpoints: {
      type: Object as PropType<Record<string, EndpointInfo[]> | null>,
      default: null,
    },
    requestText: {
      type: String,
      required: true,
    },
    lastRequestResult: {
      type: Object as PropType<LastRequestResult | null>,
      default: null,
    },
    sending: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      rawQueryKey: '',
      rawQueryPayload: '{}',
    }
  },
  computed: {
    endpointKinds(): string[] {
      if (!this.serviceEndpoints) {
        return []
      }
      return Object.keys(this.serviceEndpoints).sort()
    },
    serviceDescription(): string {
      const info = this.serviceInfo as (ServiceInfo & { description?: string }) | null
      return info?.description?.trim() ?? ''
    },
  },
  methods: {
    endpointsForKind(kind: string): EndpointInfo[] {
      return this.serviceEndpoints?.[kind] ?? []
    },
    isRequestEndpoint(endpoint: EndpointInfo): boolean {
      return REQUEST_ENDPOINT_KINDS.has(endpoint.kind)
    },
    submitRawQuery(): void {
      const key = this.rawQueryKey.trim()
      if (!key) {
        return
      }
      this.$emit('raw-query', key, this.rawQueryPayload)
    },
  },
})
</script>
