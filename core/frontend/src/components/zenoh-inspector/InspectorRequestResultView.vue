<template>
  <pre class="inspector-request-result">{{ formatted }}</pre>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import type { LastRequestResult } from '@/libs/zenoh-inspector/inspector-controller'
import { toDisplayValue } from '@/libs/zenoh-inspector/logic/decode'

export default Vue.extend({
  name: 'InspectorRequestResultView',
  props: {
    result: {
      type: Object as PropType<LastRequestResult | null>,
      default: null,
    },
  },
  computed: {
    formatted(): string {
      if (!this.result) {
        return 'No request sent yet.'
      }
      if (this.result.status === 'error') {
        return JSON.stringify({ error: this.result.message }, null, 2)
      }
      const { result } = this.result
      if (result.kind === 'ack') {
        return JSON.stringify(toDisplayValue(result.value), null, 2)
      }
      if (result.kind === 'cdr') {
        return JSON.stringify({
          schema: result.schemaName,
          value: toDisplayValue(result.value),
        }, null, 2)
      }
      const replies = result.replies.map((reply) => {
        let payload: unknown
        switch (reply.decoded.kind) {
          case 'cdr':
          case 'json':
            payload = toDisplayValue(reply.decoded.value)
            break
          case 'text':
            payload = reply.decoded.value
            break
          case 'binary':
            payload = { size: reply.decoded.size, preview: reply.decoded.preview }
            break
          case 'error':
            payload = {
              error: reply.decoded.message,
              size: reply.decoded.size,
              preview: reply.decoded.preview,
            }
            break
          default:
            payload = null
        }
        return { key: reply.key, payload }
      })
      return JSON.stringify({ replies }, null, 2)
    },
  },
})
</script>

<style scoped>
.inspector-request-result {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 0.85rem;
  max-height: 240px;
  overflow: auto;
}
</style>
