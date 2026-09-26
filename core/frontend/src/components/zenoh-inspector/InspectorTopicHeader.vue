<template>
  <div class="d-flex flex-wrap align-center">
    <span class="text-h6 text-truncate mr-2">
      {{ primaryLabel }}
    </span>
    <v-chip
      v-tooltip="'Zenoh key expression'"
      x-small
      class="mr-1 mb-1"
      color="grey darken-1"
      text-color="white"
    >
      {{ topic.key }}
    </v-chip>
    <v-chip
      v-tooltip="'Topic source'"
      x-small
      class="mr-1 mb-1"
      color="primary"
    >
      {{ topic.source }}
    </v-chip>
    <v-chip
      v-if="topic.schemaName"
      v-tooltip="'Resolved schema'"
      x-small
      class="mr-1 mb-1"
      color="info"
    >
      {{ topic.schemaName }}
    </v-chip>
    <v-chip
      v-if="topic.encoding"
      v-tooltip="'Payload encoding'"
      x-small
      class="mr-1 mb-1"
      color="secondary"
    >
      {{ topic.encoding }}
    </v-chip>
    <v-chip
      v-tooltip="'Samples received'"
      x-small
      class="mr-1 mb-1"
    >
      {{ topic.sampleCount }}
    </v-chip>
    <v-chip
      v-tooltip="'Topic liveliness'"
      x-small
      class="mr-1 mb-1"
      :color="livelinessChipColor"
    >
      {{ livelinessChipLabel }}
    </v-chip>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import { topicPrimaryLabel } from '@/libs/zenoh-inspector/logic/topic-list'
import type { TopicInfo } from '@/libs/zenoh-inspector/logic/types'

import { livelinessColor, livelinessLabel } from './topic-display'

export default Vue.extend({
  name: 'InspectorTopicHeader',
  props: {
    topic: {
      type: Object as PropType<TopicInfo>,
      required: true,
    },
  },
  computed: {
    primaryLabel(): string {
      return topicPrimaryLabel(this.topic)
    },
    livelinessChipLabel(): string {
      return livelinessLabel(this.topic.alive)
    },
    livelinessChipColor(): string {
      return livelinessColor(this.topic.alive)
    },
  },
})
</script>
