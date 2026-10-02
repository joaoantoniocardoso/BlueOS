<template>
  <v-list-item
    :input-value="selected"
    active-class="primary--text"
    @click="$emit('select', topic.key)"
  >
    <v-list-item-content>
      <v-list-item-title class="text-truncate">
        {{ primaryLabel }}
      </v-list-item-title>
      <v-list-item-subtitle
        v-if="topic.schemaName"
        class="text-truncate"
      >
        {{ topic.schemaName }}
      </v-list-item-subtitle>
    </v-list-item-content>
    <v-list-item-action v-if="topic.alive !== undefined">
      <v-chip
        v-tooltip="livelinessTooltip"
        x-small
        :color="livelinessChipColor"
      >
        {{ livelinessChipLabel }}
      </v-chip>
    </v-list-item-action>
  </v-list-item>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import { topicPrimaryLabel } from '@/libs/zenoh-inspector/logic/topic-list'
import type { TopicInfo } from '@/libs/zenoh-inspector/logic/types'

import { livelinessColor, livelinessLabel } from './topic-display'

export default Vue.extend({
  name: 'InspectorTopicListItem',
  props: {
    topic: {
      type: Object as PropType<TopicInfo>,
      required: true,
    },
    selected: {
      type: Boolean,
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
    livelinessTooltip(): string {
      return `Topic liveliness: ${this.livelinessChipLabel}`
    },
  },
})
</script>
