<template>
  <v-card
    v-if="selectedTopic"
    outlined
    class="d-flex flex-column flex-grow-1"
    min-height="400"
  >
    <v-card-title class="pb-2">
      <inspector-topic-header :topic="selectedTopic" />
    </v-card-title>
    <v-divider />
    <v-card-text class="flex-grow-1 overflow-auto pt-4">
      <inspector-view-switcher
        :views="availableViews"
        :selected-view-id="selectedViewId"
        @select-view="$emit('select-view', $event)"
      />
      <inspector-video-view
        v-if="selectedViewId === 'video'"
        :controller="controller"
        :topic-key="selectedTopic.key"
        :catalog-loaded="catalogLoaded"
        :active="true"
      />
      <inspector-json-view
        v-else
        :topic="selectedTopic"
        :decoded="selectedDecoded"
      />
    </v-card-text>
  </v-card>
  <div
    v-else
    class="select-topic d-flex align-center justify-center fill-height"
  >
    <span class="text-h6 font-weight-medium">
      Select a topic to view its messages.
    </span>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import type { InspectorController } from '@/libs/zenoh-inspector/inspector-controller'
import type {
  DecodedPayload, TopicInfo, ViewDescriptor,
} from '@/libs/zenoh-inspector/logic/types'

import InspectorJsonView from './InspectorJsonView.vue'
import InspectorTopicHeader from './InspectorTopicHeader.vue'
import InspectorVideoView from './InspectorVideoView.vue'
import InspectorViewSwitcher from './InspectorViewSwitcher.vue'

export default Vue.extend({
  name: 'InspectorTopicDetail',
  components: {
    InspectorJsonView,
    InspectorTopicHeader,
    InspectorVideoView,
    InspectorViewSwitcher,
  },
  props: {
    controller: {
      type: Object as () => InspectorController,
      required: true,
    },
    selectedTopic: {
      type: Object as PropType<TopicInfo | null>,
      default: null,
    },
    availableViews: {
      type: Array as PropType<ViewDescriptor[]>,
      required: true,
    },
    selectedViewId: {
      type: String,
      required: true,
    },
    selectedDecoded: {
      type: Object as PropType<DecodedPayload | null>,
      default: null,
    },
    catalogLoaded: {
      type: Boolean,
      required: true,
    },
  },
})
</script>

<style scoped>
.select-topic {
  min-height: 400px;
  text-align: center;
}
</style>
