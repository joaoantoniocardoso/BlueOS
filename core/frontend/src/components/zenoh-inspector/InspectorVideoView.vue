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
import { parse as parseMessageDefinition } from '@foxglove/rosmsg'
import { MessageReader } from '@foxglove/rosmsg2-serialization'
import Vue from 'vue'

import type { Unsubscribe } from '@/libs/blueos-api/zenoh-helpers'
import { parseCompressedVideo } from '@/libs/mcap/logic/video-track'
import type { InspectorController } from '@/libs/zenoh-inspector/inspector-controller'

import RawVideoPlayer from './RawVideoPlayer.vue'

const COMPRESSED_VIDEO_SCHEMA = 'foxglove_msgs/msg/CompressedVideo'

interface InspectorVideoViewBindings {
  messageReader: MessageReader | null
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
          this.ensureMessageReader().catch(() => undefined)
        }
      },
    },
    active(active: boolean) {
      if (active && this.catalogLoaded) {
        this.ensureMessageReader().catch(() => undefined)
      }
    },
  },
  created() {
    videoViewBindings(this).messageReader = null
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
    videoViewBindings(this).messageReader = null
  },
  methods: {
    async ensureMessageReader(): Promise<void> {
      const bindings = videoViewBindings(this)
      if (bindings.messageReader || this.readerError !== null) {
        return
      }
      try {
        const catalog = await import('@blueos-idl/catalog')
        const schemaText = catalog.CATALOG_SCHEMAS[COMPRESSED_VIDEO_SCHEMA]
        if (!schemaText) {
          this.readerError = `Schema ${COMPRESSED_VIDEO_SCHEMA} is missing from the catalog`
          return
        }
        const definition = parseMessageDefinition(schemaText, { ros2: true })
        bindings.messageReader = new MessageReader(definition)
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error)
        this.readerError = message
      }
    },
    forwardFrame(payload: Uint8Array): void {
      const { messageReader } = videoViewBindings(this)
      if (!messageReader) {
        return
      }
      const player = this.$refs.video_player as {
        pushFrame?: (data: Uint8Array, format: import('@/libs/mcap').VideoFormat, timestamp?: number) => void
      } | undefined
      if (!player?.pushFrame) {
        return
      }
      try {
        const frame = parseCompressedVideo(messageReader, payload)
        player.pushFrame(frame.data, frame.format, frame.timestampSeconds)
      } catch {
        // Stay on the last good frame; a bad payload should not tear down the player.
      }
    },
  },
})
</script>
