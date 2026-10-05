<template>
  <div>
    <v-alert
      v-if="readerError"
      type="error"
      dense
      class="mb-2"
    >
      {{ readerError }}
    </v-alert>
    <raw-video-player
      v-else
      ref="video_player"
      :key="topicKey"
    />
  </div>
</template>

<script lang="ts">
/* eslint-disable import/no-extraneous-dependencies */
import Vue from 'vue'

import type { VideoFormat } from '@/libs/mcap'
import { createVideoFrameReader } from '@/libs/zenoh-inspector/adapters/video-frame-reader'
import type { InspectorController } from '@/libs/zenoh-inspector/inspector-controller'
import type { Unsubscribe } from '@/libs/zenoh-inspector/logic/types'

import RawVideoPlayer from './RawVideoPlayer.vue'

const COMPRESSED_VIDEO_SCHEMA = 'foxglove_msgs/msg/CompressedVideo'

interface InspectorVideoViewBindings {
  readFrame: ReturnType<typeof createVideoFrameReader> | null
}

function videoViewBindings(component: Vue): InspectorVideoViewBindings {
  return component as unknown as InspectorVideoViewBindings
}

export default Vue.extend({
  name: 'InspectorVideoView',
  components: {
    RawVideoPlayer,
  },
  props: {
    controller: {
      type: Object as () => InspectorController,
      required: true,
    },
    topicKey: {
      type: String,
      required: true,
    },
    catalogLoaded: {
      type: Boolean,
      required: true,
    },
    active: {
      type: Boolean,
      required: true,
    },
  },
  data() {
    return {
      readerError: null as string | null,
      unsubscribeSample: null as Unsubscribe | null,
    }
  },
  watch: {
    catalogLoaded: {
      immediate: true,
      handler(loaded: boolean) {
        if (loaded) {
          this.ensureFrameReader().catch(() => undefined)
        }
      },
    },
    active(active: boolean) {
      if (active && this.catalogLoaded) {
        this.ensureFrameReader().catch(() => undefined)
      }
    },
  },
  created() {
    videoViewBindings(this).readFrame = null
  },
  mounted() {
    this.unsubscribeSample = this.controller.subscribeSelectedSample((sample) => {
      if (!this.active) {
        return
      }
      this.forwardFrame(sample.payload)
    })
  },
  beforeDestroy() {
    this.unsubscribeSample?.()
    this.unsubscribeSample = null
    videoViewBindings(this).readFrame = null
  },
  methods: {
    async ensureFrameReader(): Promise<void> {
      const bindings = videoViewBindings(this)
      if (bindings.readFrame || this.readerError !== null) {
        return
      }
      try {
        const catalog = await import('@blueos-idl/catalog')
        const schemaText = catalog.CATALOG_SCHEMAS[COMPRESSED_VIDEO_SCHEMA]
        if (!schemaText) {
          this.readerError = `Schema ${COMPRESSED_VIDEO_SCHEMA} is missing from the catalog`
          return
        }
        bindings.readFrame = createVideoFrameReader(schemaText)
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        this.readerError = message
      }
    },
    forwardFrame(payload: Uint8Array): void {
      const { readFrame } = videoViewBindings(this)
      const player = this.$refs.video_player as {
        pushFrame?: (data: Uint8Array, format: VideoFormat, timestamp?: number) => void
      } | undefined
      if (!readFrame || !player?.pushFrame) {
        return
      }
      const frame = readFrame(payload)
      if (frame) {
        player.pushFrame(frame.data, frame.format, frame.timestampSeconds)
      }
    },
  },
})
</script>
