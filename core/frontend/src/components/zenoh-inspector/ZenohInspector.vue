<template>
  <v-container fluid>
    <v-alert
      v-if="inspector.sourceError"
      type="error"
      dense
      class="mb-4"
    >
      {{ inspector.sourceError }}
    </v-alert>
    <v-row>
      <v-col sm="4">
        <v-sheet
          rounded="lg"
          min-height="268"
        >
          <inspector-topic-list
            :filter="inspector.filter"
            :topic-groups="inspector.topicGroups"
            :selected-key="inspector.selectedKey"
            @filter="onFilter"
            @select="onSelectTopic"
          />
        </v-sheet>
      </v-col>
      <v-col sm="8">
        <v-card
          outlined
          width="100%"
          min-height="700"
          class="d-flex flex-column pa-0"
        >
          <inspector-topic-detail
            :controller="inspectorController"
            :selected-topic="inspector.selectedTopic"
            :available-views="inspector.availableViews"
            :selected-view-id="inspector.selectedViewId"
            :selected-decoded="inspector.selectedDecoded"
            :catalog-loaded="inspector.catalogLoaded"
            @select-view="onSelectView"
          />
          <v-card-text v-if="inspector.selectedService">
            <inspector-service-panel
              :selected-service="inspector.selectedService"
              :service-info="inspector.serviceInfo"
              :service-endpoints="inspector.serviceEndpoints"
              :request-text="inspector.requestText"
              :last-request-result="inspector.lastRequestResult"
              :sending="requestInFlight"
              @request-text="onRequestText"
              @fill-default="onFillDefault"
              @send-request="onSendRequest"
              @raw-query="onRawQuery"
            />
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts">
import type { EndpointInfo } from '@blueos-idl/messages'
import Vue from 'vue'

import {
  createInspectorController,
  type InspectorController,
  type InspectorViewState,
} from '@/libs/zenoh-inspector'
import type { TopicInfo } from '@/libs/zenoh-inspector/logic/types'

import InspectorServicePanel from './InspectorServicePanel.vue'
import InspectorTopicDetail from './InspectorTopicDetail.vue'
import InspectorTopicList from './InspectorTopicList.vue'

interface ZenohInspectorBindings {
  controller: InspectorController
}

function emptyInspectorViewState(): InspectorViewState {
  return {
    filter: '',
    topicGroups: [],
    selectedKey: null,
    selectedTopic: null,
    availableViews: [],
    selectedViewId: 'json',
    selectedDecoded: null,
    selectedService: null,
    serviceInfo: null,
    serviceEndpoints: null,
    requestText: '',
    lastRequestResult: null,
    catalogLoaded: false,
    sourceError: null,
  }
}

function inspectorBindings(component: Vue): ZenohInspectorBindings {
  return component as unknown as ZenohInspectorBindings
}

export default Vue.extend({
  name: 'ZenohInspector',
  components: {
    InspectorServicePanel,
    InspectorTopicDetail,
    InspectorTopicList,
  },
  data() {
    return {
      inspector: Object.freeze(emptyInspectorViewState()),
      requestInFlight: false,
    }
  },
  computed: {
    inspectorController(): InspectorController {
      return inspectorBindings(this).controller
    },
  },
  created() {
    inspectorBindings(this).controller = createInspectorController({
      onState: (state: InspectorViewState) => {
        this.inspector = Object.freeze(state)
      },
    })
  },
  mounted() {
    inspectorBindings(this).controller.start()
  },
  beforeDestroy() {
    inspectorBindings(this).controller.stop()
  },
  methods: {
    onFilter(text: string | null): void {
      inspectorBindings(this).controller.setFilter(text ?? '')
    },
    onSelectView(viewId: string): void {
      inspectorBindings(this).controller.selectView(viewId)
    },
    onSelectTopic(key: string): void {
      const { controller } = inspectorBindings(this)
      controller.selectTopic(key)
      let topic: TopicInfo | undefined
      for (const group of this.inspector.topicGroups) {
        topic = group.topics.find((entry) => entry.key === key)
        if (topic) {
          break
        }
      }
      const service = topic?.blueos?.service ?? null
      controller.selectService(service)
    },
    onRequestText(text: string): void {
      inspectorBindings(this).controller.setRequestText(text)
    },
    onFillDefault(schemaName: string): void {
      inspectorBindings(this).controller.fillDefaultRequestText(schemaName)
    },
    async onSendRequest(endpoint: EndpointInfo): Promise<void> {
      this.requestInFlight = true
      try {
        await inspectorBindings(this).controller.sendRequest(endpoint)
      } finally {
        this.requestInFlight = false
      }
    },
    async onRawQuery(key: string, text: string): Promise<void> {
      this.requestInFlight = true
      try {
        await inspectorBindings(this).controller.rawQuery(key, text)
      } finally {
        this.requestInFlight = false
      }
    },
  },
})
</script>

<style scoped>
.select-topic {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  text-align: center;
}
</style>
