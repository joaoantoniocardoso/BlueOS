<template>
  <div v-if="progress" class="records-repair-progress">
    <v-progress-linear
      :value="progress.percent"
      color="primary"
      height="8"
      rounded
    />
    <div class="caption mt-1">
      {{ progress.label }}
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import type { LibraryRecording } from '@/libs/recorder/types'
import { repairProgress } from '@/libs/recorder/view-logic'

export default Vue.extend({
  name: 'RecordsRepairProgress',
  props: {
    file: {
      type: Object as PropType<LibraryRecording>,
      required: true,
    },
  },
  computed: {
    progress(): { percent: number, label: string } | null {
      return repairProgress(this.file)
    },
  },
})
</script>

<style scoped>
.records-repair-progress {
  min-width: 120px;
}
</style>
