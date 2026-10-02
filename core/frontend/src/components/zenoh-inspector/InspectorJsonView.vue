<template>
  <pre class="inspector-json">{{ jsonText }}</pre>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import { formatTopicJson } from '@/libs/zenoh-inspector/logic/decode'
import type { DecodedPayload, TopicInfo } from '@/libs/zenoh-inspector/logic/types'

export default Vue.extend({
  name: 'InspectorJsonView',
  props: {
    topic: {
      type: Object as PropType<TopicInfo>,
      required: true,
    },
    decoded: {
      type: Object as PropType<DecodedPayload | null>,
      default: null,
    },
  },
  computed: {
    jsonText(): string {
      if (!this.decoded) {
        return 'Waiting for a sample on this topic…'
      }
      return formatTopicJson(this.topic, this.decoded)
    },
  },
})
</script>

<style scoped>
.inspector-json {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 0.85rem;
}
</style>
