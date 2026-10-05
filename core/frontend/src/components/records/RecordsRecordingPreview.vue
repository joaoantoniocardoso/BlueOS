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
      :height="compact ? 54 : 180"
      :class="[previewBackground, 'preview-clickable']"
      aspect-ratio="16/9"
      contain
    >
      <div v-if="canPlay && !compact" class="preview-overlay d-flex align-center justify-center">
        <v-btn
          v-tooltip="`Play ${file.name}`"
          :aria-label="`Play ${file.name}`"
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
      class="preview-placeholder d-flex flex-column align-center justify-center"
      :class="[previewBackground, { 'preview-compact': compact }]"
    >
      <v-progress-circular indeterminate color="primary" :size="compact ? 24 : 48" />
      <span v-if="!compact" class="mt-2 caption grey--text text--darken-1">Processing thumbnail...</span>
    </div>
    <div
      v-else
      class="preview-placeholder d-flex flex-column align-center justify-center preview-clickable"
      :class="[previewBackground, { 'preview-compact': compact }]"
    >
      <v-icon :large="!compact" color="grey darken-1">
        mdi-multimedia
      </v-icon>
      <v-btn
        v-if="canPlay && !compact"
        v-tooltip="`Play ${file.name}`"
        :aria-label="`Play ${file.name}`"
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

import type { ByteSource } from '@/libs/mcap/logic/byte-source'
import { loadRecordingThumbnail } from '@/libs/recorder/thumbnail-loader'
import type { LibraryRecording } from '@/libs/recorder/types'
import { canLoadThumbnail, canPlayRecording } from '@/libs/recorder/view-logic'

export default Vue.extend({
  name: 'RecordsRecordingPreview',
  props: {
    file: {
      type: Object as PropType<LibraryRecording>,
      required: true,
    },
    byteSource: {
      type: Function as PropType<(path: string) => ByteSource | undefined>,
      required: true,
    },
    disabled: {
      type: Boolean,
      default: false,
    },
    /** A small thumbnail for a table cell, without the Play overlay. */
    compact: {
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
    previewBackground(): string {
      return this.$vuetify.theme.dark ? 'grey darken-4' : 'grey lighten-3'
    },
    fileSizeBytes(): number {
      return this.file.size_bytes
    },
  },
  watch: {
    'file.path': {
      immediate: true,
      handler() {
        this.loadThumbnail()
      },
    },
    fileSizeBytes() {
      this.loadThumbnail()
    },
    canPlay() {
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
      const source = this.byteSource(this.file.path)
      if (!this.canPlay || !canLoadThumbnail(this.file) || !source) {
        this.controller?.abort()
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
          source,
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

.preview-placeholder.preview-compact {
  height: 54px;
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
