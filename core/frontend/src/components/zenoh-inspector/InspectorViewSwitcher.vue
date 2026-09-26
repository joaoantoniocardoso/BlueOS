<template>
  <v-btn-toggle
    v-if="views.length > 1"
    :value="selectedViewId"
    mandatory
    dense
    class="mb-3"
    @change="$emit('select-view', $event)"
  >
    <v-btn
      v-for="view in views"
      :key="view.id"
      :value="view.id"
      small
    >
      {{ view.label }}
    </v-btn>
  </v-btn-toggle>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import type { ViewDescriptor } from '@/libs/zenoh-inspector/logic/types'

export default Vue.extend({
  name: 'InspectorViewSwitcher',
  props: {
    views: {
      type: Array as PropType<ViewDescriptor[]>,
      required: true,
    },
    selectedViewId: {
      type: String,
      required: true,
    },
  },
})
</script>
