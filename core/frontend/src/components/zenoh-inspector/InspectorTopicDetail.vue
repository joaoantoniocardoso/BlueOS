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
      <raw-video-player
        v-if="isVideoTopic"
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
import { isVideoTopicKey } from '@/libs/zenoh-inspector/logic/topic-list'
import type { DecodedPayload, SampleRecord, TopicInfo } from '@/libs/zenoh-inspector/logic/types'

import InspectorJsonView from './InspectorJsonView.vue'
import InspectorTopicHeader from './InspectorTopicHeader.vue'
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
    selectedDecoded: {
      type: Object as PropType<DecodedPayload | null>,
      default: null,
    },
  },
  computed: {
    isVideoTopic(): boolean {
      return this.selectedTopic !== null && isVideoTopicKey(this.selectedTopic.key)
    },
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
      immediate: true,
      handler(topic: TopicInfo | null): void {
        detailBindings(this).sampleUnsubscribe?.()
        detailBindings(this).sampleUnsubscribe = null
        detailBindings(this).latestSample = null
        if (topic === null || !isVideoTopicKey(topic.key)) {
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
  },
  async mounted() {
    const CompressedVideo = await axios.get('/msgs/CompressedVideo.msg').then((response) => response.data as string)
    detailBindings(this).videoReader = new MessageReader(parseMessageDefinition(CompressedVideo))
  },
  beforeDestroy() {
    detailBindings(this).sampleUnsubscribe?.()
    detailBindings(this).sampleUnsubscribe = null
  },
})
</script>

<style scoped>
.select-topic {
  min-height: 400px;
  text-align: center;
}
</style>
