<template>
  <div
    class="preview-wrapper"
    role="button"
    tabindex="0"
    @click="onClick"
    @keydown.enter.prevent="onClick"
    @keydown.space.prevent="onClick"
  >
    <v-img
      v-if="objectUrl"
      :src="objectUrl"
      height="180"
      class="grey lighten-3 preview-clickable"
      aspect-ratio="16/9"
      contain
    >
      <div v-if="canPlay" class="preview-overlay d-flex align-center justify-center">
        <v-btn
          icon
          large
          color="primary"
          class="play-btn sheet_bg"
          @click.stop="onClick"
        >
          <v-icon large>
            mdi-play-circle
          </v-icon>
        </v-btn>
      </div>
    </v-img>
    <div
      v-else-if="loading"
      class="preview-placeholder grey lighten-3 d-flex flex-column align-center justify-center"
    >
      <v-progress-circular indeterminate color="primary" size="48" />
      <span class="mt-2 caption grey--text text--darken-1">Processing thumbnail...</span>
    </div>
    <div
      v-else
      class="preview-placeholder grey lighten-3 d-flex flex-column align-center justify-center preview-clickable"
    >
      <v-icon large color="grey darken-1">
        mdi-multimedia
      </v-icon>
      <v-btn
        v-if="canPlay"
        icon
        large
        color="primary"
        class="play-btn sheet_bg mt-2"
        @click.stop="onClick"
      >
        <v-icon large>
          mdi-play-circle
        </v-icon>
      </v-btn>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import { loadRecordingThumbnail } from '@/libs/recorder/thumbnail-loader'
import type { LibraryRecording } from '@/libs/recorder/types'
import { canPlayRecording } from '@/libs/recorder/view-logic'

export default Vue.extend({
  name: 'RecordsRecordingPreview',
  props: {
    file: {
      type: Object as PropType<LibraryRecording>,
      required: true,
    },
    downloadUrl: {
      type: String,
      required: true,
    },
    disabled: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      objectUrl: null as string | null,
      loading: false,
      controller: null as AbortController | null,
    }
  },
  computed: {
    canPlay(): boolean {
      return !this.disabled && canPlayRecording(this.file)
    },
    fileSizeBytes(): number {
      return this.file.size_bytes
    },
  },
  watch: {
    downloadUrl: {
      immediate: true,
      handler() {
        this.loadThumbnail()
      },
    },
    fileSizeBytes() {
      this.loadThumbnail()
    },
  },
  beforeDestroy() {
    this.revokeObjectUrl()
    this.controller?.abort()
  },
  methods: {
    onClick(): void {
      if (this.canPlay) {
        this.$emit('play')
      }
    },
    revokeObjectUrl(): void {
      if (this.objectUrl) {
        URL.revokeObjectURL(this.objectUrl)
        this.objectUrl = null
      }
    },
    async loadThumbnail(): Promise<void> {
      if (!this.canPlay || !this.downloadUrl) {
        this.revokeObjectUrl()
        this.loading = false
        return
      }
      this.controller?.abort()
      const controller = new AbortController()
      this.controller = controller
      const { signal } = controller
      this.loading = true
      try {
        const blob = await loadRecordingThumbnail({
          downloadUrl: this.downloadUrl,
          cacheKey: {
            path: this.file.path,
            sizeBytes: this.file.size_bytes,
            created: this.file.created,
          },
          signal,
        })
        if (signal.aborted) {
          return
        }
        this.revokeObjectUrl()
        if (blob) {
          this.objectUrl = URL.createObjectURL(blob)
        }
      } catch {
        this.revokeObjectUrl()
      } finally {
        if (!signal.aborted) {
          this.loading = false
        }
      }
    },
  },
})
</script>

<style scoped>
.preview-wrapper {
  position: relative;
}

.preview-placeholder {
  height: 180px;
}

.preview-overlay {
  position: relative;
  isolation: isolate;
  width: 100%;
  height: 100%;
}

.preview-overlay::before {
  content: '';
  position: absolute;
  inset: 0;
  background-color: var(--v-blue_whale-base);
  opacity: 0.25;
  z-index: 0;
  pointer-events: none;
}

.preview-overlay .play-btn {
  position: relative;
  z-index: 1;
  opacity: 0.85;
}

.preview-clickable {
  cursor: pointer;
}
</style>
