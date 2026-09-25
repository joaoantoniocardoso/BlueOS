<template>
  <div>
    <div class="stream-wrapper">
      <video
        ref="player"
        autoplay
        muted
        playsinline
        class="stream"
      >
        <track
          kind="captions"
          srclang="en"
          label="Captions not available"
          :src="empty_captions"
          default
        >
      </video>
      <div v-if="!available && !error" class="stream-overlay">
        <span class="caption text-center white--text">Not available at this timestamp</span>
      </div>
      <div v-else-if="waiting && !error" class="stream-overlay stream-waiting">
        <span class="caption white--text">Waiting for new data...</span>
      </div>
      <div v-else-if="loading && !error" class="stream-overlay">
        <v-progress-circular indeterminate color="primary" size="40" />
        <span class="mt-2 caption white--text">{{ loading_message }}</span>
      </div>
      <div v-if="error" class="stream-overlay stream-error px-4">
        <v-icon color="error">
          mdi-alert-circle-outline
        </v-icon>
        <span class="mt-2 caption text-center white--text">{{ error }}</span>
      </div>
      <div v-if="statistics && !error" class="stats-overlay">
        <table class="grey--text text--lighten-3">
          <tbody>
            <tr v-for="row in detail_stat_rows" :key="row.label">
              <td class="stats-label">
                {{ row.label }}
              </td>
              <td :class="['stats-value', row.tone]">
                {{ row.value }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="stream-meta d-flex align-center caption mt-2 white--text">
      <v-icon x-small :color="error ? 'error' : 'success'" class="mr-1">
        mdi-circle
      </v-icon>
      <span class="font-weight-medium text-truncate">{{ track.name }}</span>
      <span v-if="resolution" class="ml-2 grey--text text--lighten-1">{{ resolution }}</span>
      <span v-if="codec_label" class="ml-2 grey--text text--lighten-1">{{ codec_label }}</span>
      <span v-if="frame_rate" class="ml-2 grey--text text--lighten-1">{{ frame_rate }}</span>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import {
  codecFamily,
  McapStreamPanelController,
  McapVideoRecording,
  streamDetailStatRows,
  streamResolution,
  VideoTrack,
} from '@/libs/mcap'

export default Vue.extend({
  name: 'McapVideoStream',
  props: {
    recording: { type: Object as PropType<McapVideoRecording>, required: true },
    track: { type: Object as PropType<VideoTrack>, required: true },
    statistics: { type: Boolean, default: false },
    ongoing: { type: Boolean, default: false },
    available: { type: Boolean, default: true },
    position: { type: Number, default: 0 },
  },
  data() {
    return {
      panel: null as McapStreamPanelController | null,
      stats: {} as Record<string, unknown>,
      error: null as string | null,
      loading: true,
      waiting: false,
      loading_message: this.ongoing ? 'Loading latest frame...' : 'Loading video...',
      empty_captions: 'data:text/vtt,WEBVTT',
    }
  },
  computed: {
    resolution(): string {
      return streamResolution(this.stats)
    },
    codec_label(): string {
      const codec = (this.stats.codec as string | undefined) ?? ''
      return codec ? codecFamily(codec) : ''
    },
    frame_rate(): string {
      const frameRate = (this.stats.frameRate as number | undefined) ?? 0
      return frameRate > 0 ? `${frameRate.toFixed(1)} fps` : ''
    },
    detail_stat_rows() {
      return streamDetailStatRows(this.stats, this.track)
    },
  },
  watch: {
    available(available: boolean) {
      const video = this.$refs.player as HTMLVideoElement | undefined
      this.panel?.setAvailable(available, video ?? null, this.position)
    },
  },
  mounted() {
    const video = this.$refs.player as HTMLVideoElement
    this.panel = new McapStreamPanelController(this.recording, this.track, this.ongoing, {
      onState: (state) => {
        this.stats = state.stats
        this.error = state.error
        this.loading = state.loading
        this.waiting = state.waiting
        this.loading_message = state.loadingMessage
      },
      onStats: (stats) => this.$emit('stats', stats),
      onTimeUpdate: (seconds) => this.$emit('timeupdate', seconds),
      onPlay: () => this.$emit('play'),
      onPause: () => this.$emit('pause'),
      onProgress: () => this.$emit('progress'),
      onExtended: () => this.$emit('extended'),
      onError: (error) => this.$emit('error', error),
    })
    this.$emit('ready', video)
    this.panel.setAvailable(this.available, video, this.position)
    video.addEventListener('timeupdate', this.onTimeUpdate)
    video.addEventListener('play', this.onPlay)
    video.addEventListener('pause', this.onPause)
    video.addEventListener('progress', this.onProgress)
  },
  beforeDestroy() {
    const video = this.$refs.player as HTMLVideoElement
    video?.removeEventListener('timeupdate', this.onTimeUpdate)
    video?.removeEventListener('play', this.onPlay)
    video?.removeEventListener('pause', this.onPause)
    video?.removeEventListener('progress', this.onProgress)
    this.panel?.destroy()
  },
  methods: {
    onTimeUpdate(): void {
      const video = this.$refs.player as HTMLVideoElement
      this.panel?.handleTimeUpdate(video)
    },
    onPlay(): void {
      this.panel?.handlePlay()
    },
    onPause(): void {
      this.panel?.handlePause(this.available)
    },
    onProgress(): void {
      this.panel?.handleProgress()
    },
    // eslint-disable-next-line vue/no-unused-properties -- parent playback bar calls these through $refs
    seek(seconds: number): void {
      this.panel?.seek(seconds)
    },
    // eslint-disable-next-line vue/no-unused-properties -- parent playback bar calls these through $refs
    play(): void {
      const video = this.$refs.player as HTMLVideoElement
      this.panel?.play(video)
    },
    // eslint-disable-next-line vue/no-unused-properties -- parent playback bar calls these through $refs
    pause(): void {
      const video = this.$refs.player as HTMLVideoElement
      this.panel?.pause(video)
    },
  },
})

</script>

<style scoped>
.stream-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  max-height: calc(72vh / var(--grid-rows, 1));
  background: #000;
}

.stream {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  background: #000;
}

.stream-overlay {
  position: absolute;
  inset: 0;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}

.stream-error {
  background: rgba(0, 0, 0, 0.85);
}

.stream-waiting {
  justify-content: flex-end;
  padding-bottom: 12px;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.55));
}

.stream-meta {
  min-height: 20px;
  color: #e5e7eb;
}

.stats-overlay {
  position: absolute;
  top: 6px;
  left: 6px;
  z-index: 2;
  max-width: calc(100% - 12px);
  padding: 8px 10px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.7);
  pointer-events: none;
}

.stats-overlay table {
  border-collapse: collapse;
  font-family: monospace;
  font-size: 11px;
  line-height: 1.35;
}

.stats-label {
  padding-right: 8px;
  opacity: 0.7;
  white-space: nowrap;
}

.stats-value {
  text-align: right;
  white-space: nowrap;
}
</style>
