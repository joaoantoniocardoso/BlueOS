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
      <raw-video-player
        v-if="selectedViewId === 'video'"
        :video-data="videoData"
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
/* eslint-disable import/no-extraneous-dependencies */
import { parse as parseMessageDefinition } from '@foxglove/rosmsg'
import { MessageReader } from '@foxglove/rosmsg2-serialization'
import axios from 'axios'
import Vue, { PropType } from 'vue'

import type { InspectorController } from '@/libs/zenoh-inspector/inspector-controller'
import type {
  DecodedPayload, SampleRecord, TopicInfo, ViewDescriptor,
} from '@/libs/zenoh-inspector/logic/types'

import InspectorJsonView from './InspectorJsonView.vue'
import InspectorTopicHeader from './InspectorTopicHeader.vue'
import InspectorViewSwitcher from './InspectorViewSwitcher.vue'
import RawVideoPlayer from './RawVideoPlayer.vue'

interface InspectorTopicDetailBindings {
  videoReader: MessageReader | null
  latestSample: SampleRecord | null
  sampleUnsubscribe: (() => void) | null
}

function detailBindings(component: Vue): InspectorTopicDetailBindings {
  return component as unknown as InspectorTopicDetailBindings
}

export default Vue.extend({
  name: 'InspectorTopicDetail',
  components: {
    InspectorJsonView,
    InspectorTopicHeader,
    InspectorViewSwitcher,
    RawVideoPlayer,
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
  },
  computed: {
    videoData(): Uint8Array {
      const sample = detailBindings(this).latestSample
      const reader = detailBindings(this).videoReader
      if (sample === null || reader === null) {
        return new Uint8Array()
      }
      const message = reader.readMessage(sample.payload) as { data: Uint8Array }
      return message.data
    },
  },
  watch: {
    selectedTopic: {
      handler(): void {
        this.syncVideoSampleSubscription()
      },
    },
    selectedViewId: {
      handler(): void {
        this.syncVideoSampleSubscription()
      },
    },
  },
  async created() {
    const CompressedVideo = await axios.get('/msgs/CompressedVideo.msg').then((response) => response.data as string)
    detailBindings(this).videoReader = new MessageReader(parseMessageDefinition(CompressedVideo))
  },
  mounted() {
    this.syncVideoSampleSubscription()
  },
  beforeDestroy() {
    detailBindings(this).sampleUnsubscribe?.()
    detailBindings(this).sampleUnsubscribe = null
  },
  methods: {
    syncVideoSampleSubscription(): void {
      detailBindings(this).sampleUnsubscribe?.()
      detailBindings(this).sampleUnsubscribe = null
      detailBindings(this).latestSample = null
      const topic = this.selectedTopic
      if (topic === null || this.selectedViewId !== 'video') {
        return
      }
      detailBindings(this).sampleUnsubscribe = this.controller.subscribeSelectedSample((sample) => {
        detailBindings(this).latestSample = sample
      })
      if (topic.lastSample) {
        detailBindings(this).latestSample = topic.lastSample
      }
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
