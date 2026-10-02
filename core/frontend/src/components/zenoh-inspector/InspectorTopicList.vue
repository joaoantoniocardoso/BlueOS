<template>
  <v-card
    class="mx-auto height-limited"
    max-height="700px"
  >
    <v-card-title>
      <v-text-field
        :value="filter"
        :label="`Search Topics (${topicCount})`"
        clearable
        prepend-inner-icon="mdi-magnify"
        single-line
        hide-details
        class="mt-0 pt-0"
        @input="$emit('filter', $event)"
      />
    </v-card-title>
    <v-divider />
    <v-virtual-scroll
      :bench="24"
      :items="rows"
      :item-height="52"
      height="580"
    >
      <template #default="{ item }">
        <div
          v-if="item.kind === 'heading'"
          :key="item.rowKey"
          class="topic-list-heading caption grey--text text--darken-1 px-4 pt-3 pb-1"
        >
          {{ item.title }}
        </div>
        <div
          v-else-if="item.kind === 'service'"
          :key="item.rowKey"
          class="topic-list-service d-flex align-center px-4 py-1"
        >
          <span class="subtitle-2">{{ item.service }}</span>
          <v-chip
            v-tooltip="serviceLivelinessTooltip(item.alive)"
            x-small
            class="ml-2"
            :color="livelinessColor(item.alive)"
          >
            {{ livelinessLabel(item.alive) }}
          </v-chip>
        </div>
        <inspector-topic-list-item
          v-else
          :key="item.rowKey"
          :topic="item.topic"
          :selected="selectedKey === item.topic.key"
          @select="$emit('select', $event)"
        />
      </template>
    </v-virtual-scroll>
  </v-card>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import {
  flattenTopicGroups,
  topicCountInGroups,
  type TopicListRow,
} from '@/libs/zenoh-inspector/logic/topic-list'
import type { TopicGroup } from '@/libs/zenoh-inspector/logic/types'

import InspectorTopicListItem from './InspectorTopicListItem.vue'
import { livelinessColor, livelinessLabel } from './topic-display'

export default Vue.extend({
  name: 'InspectorTopicList',
  components: {
    InspectorTopicListItem,
  },
  props: {
    filter: {
      type: String,
      required: true,
    },
    topicGroups: {
      type: Array as PropType<TopicGroup[]>,
      required: true,
    },
    selectedKey: {
      type: String as PropType<string | null>,
      default: null,
    },
  },
  computed: {
    rows(): TopicListRow[] {
      return flattenTopicGroups(this.topicGroups)
    },
    topicCount(): number {
      return topicCountInGroups(this.topicGroups)
    },
  },
  methods: {
    livelinessColor,
    livelinessLabel,
    serviceLivelinessTooltip(alive: boolean | undefined): string {
      return `Service liveliness: ${livelinessLabel(alive)}`
    },
  },
})
</script>

<style scoped>
.height-limited {
  overflow: hidden;
}

.topic-list-heading {
  min-height: 28px;
}

.topic-list-service {
  min-height: 36px;
  background: rgba(0, 0, 0, 0.03);
}
</style>
