<template>
  <div class="player">
    <v-alert
      v-if="error"
      type="error"
      dense
      class="mb-2"
    >
      {{ error }}
    </v-alert>

    <div v-if="opening" class="d-flex flex-column align-center py-8 px-4">
      <v-progress-circular
        :indeterminate="opening_percent === null"
        :value="opening_percent ?? 0"
        color="primary"
        size="64"
        width="5"
      >
        <span v-if="opening_percent !== null" class="caption">
          {{ Math.round(opening_percent) }}%
        </span>
      </v-progress-circular>
      <span class="mt-3 caption grey--text text-center">
        {{ opening_message }}
      </span>
      <span v-if="opening_status" class="mt-1 caption grey--text text-center">
        {{ opening_status }}
      </span>
    </div>

    <div
      v-if="recording && tracks.length > 0"
      class="player-stage"
    >
      <div
        v-if="visible_tracks.length === 0"
        class="stream-empty caption text-center py-8"
      >
        Select at least one stream to play.
      </div>
      <div
        v-else
        class="stream-grid"
        :style="grid_style"
      >
        <mcap-video-stream
          v-for="track in visible_tracks"
          :key="track.channelId"
          ref="stream"
          :recording="recording"
          :track="track"
          :ongoing="ongoing"
          :available="trackCovers(track, position)"
          :position="position"
          :statistics="statistics"
          @ready="onStreamReady(track.channelId, $event)"
          @stats="onStats(track.channelId, $event)"
          @timeupdate="onStreamTime(track.channelId, $event)"
          @play="onLeaderPlay"
          @pause="onLeaderPause"
          @progress="updateBuffered"
          @extended="handleExtended"
        />
      </div>

      <div class="playback-bar">
        <div class="d-flex align-center">
          <v-btn
            v-tooltip="playing ? 'Pause' : 'Play'"
            icon
            small
            dark
            @click="togglePlayback"
          >
            <v-icon>
              {{ playing ? 'mdi-pause' : 'mdi-play' }}
            </v-icon>
          </v-btn>
          <v-btn
            v-if="ongoing"
            v-tooltip="at_latest
              ? 'Showing the latest recorded frames'
              : 'Skip to the latest recorded frames'"
            icon
            small
            dark
            class="ml-1"
            :color="at_latest ? 'primary' : 'white'"
            @click="skipToLatest"
          >
            <v-icon>
              mdi-fast-forward
            </v-icon>
          </v-btn>
          <span class="caption mx-2 playback-time">
            {{ positionLabel(position) }} / {{ positionLabel(duration) }}
          </span>
          <v-spacer />
          <span
            v-if="ongoing && !at_latest"
            class="caption mr-2"
          >
            {{ frame_age_label }}
          </span>
          <v-menu
            v-if="tracks.length > 0"
            v-model="stream_menu"
            offset-y
            left
            :close-on-content-click="false"
            max-height="420"
          >
            <template #activator="{ on, attrs }">
              <v-btn
                v-tooltip="'Choose which streams to show'"
                small
                dark
                text
                class="stream-picker-btn"
                v-bind="attrs"
                v-on="on"
              >
                <v-icon small left>
                  mdi-video-outline
                </v-icon>
                Streams {{ visible_tracks.length }}/{{ tracks.length }}
              </v-btn>
            </template>
            <v-card class="stream-picker" dark>
              <v-card-text class="pb-1">
                <v-text-field
                  v-model="stream_search"
                  dense
                  hide-details
                  outlined
                  clearable
                  dark
                  label="Search streams"
                  prepend-inner-icon="mdi-magnify"
                />
              </v-card-text>
              <v-list
                dense
                dark
                class="py-0 stream-picker-list"
              >
                <v-list-item
                  v-for="track in filtered_tracks"
                  :key="track.channelId"
                  @click="toggleStream(track.channelId)"
                >
                  <v-list-item-action class="mr-2">
                    <v-checkbox
                      :input-value="selected_channel_ids.includes(track.channelId)"
                      dense
                      hide-details
                      color="primary"
                      @click.stop="toggleStream(track.channelId)"
                    />
                  </v-list-item-action>
                  <v-list-item-content>
                    <v-list-item-title>{{ track.name }}</v-list-item-title>
                    <v-list-item-subtitle>
                      {{ track.frameCount.toLocaleString() }} frames
                    </v-list-item-subtitle>
                  </v-list-item-content>
                </v-list-item>
                <v-list-item v-if="filtered_tracks.length === 0">
                  <v-list-item-title class="grey--text">
                    No matching streams
                  </v-list-item-title>
                </v-list-item>
              </v-list>
              <v-card-actions>
                <v-btn small text @click="selectAllStreams">
                  Select all
                </v-btn>
                <v-btn small text @click="selectNoStreams">
                  None
                </v-btn>
              </v-card-actions>
            </v-card>
          </v-menu>
        </div>
        <div
          ref="timeline"
          class="timeline mt-1"
          :class="{ 'timeline-over-video': pointer_over_video }"
          role="slider"
          :aria-valuemin="0"
          :aria-valuemax="duration"
          :aria-valuenow="position"
          @pointerdown="onTimelineDown"
          @pointermove="onTimelineMove"
          @pointerup="onTimelineUp"
          @pointercancel="onTimelineUp"
          @pointerleave="onTimelineLeave"
        >
          <div class="timeline-track" />
          <div
            v-for="(range, index) in video_styles"
            :key="`video-${index}`"
            class="timeline-video"
            :style="range.style"
          >
            <span class="timeline-mark timeline-mark-start">&gt;</span>
            <span class="timeline-mark timeline-mark-end">&lt;</span>
          </div>
          <div
            v-for="(range, index) in buffered_styles"
            :key="`buffered-${index}`"
            class="timeline-buffered"
            :style="range"
          />
          <div
            class="timeline-playhead"
            :style="{ left: playhead_percent }"
          />
          <div
            v-if="pointer_seconds !== null"
            class="timeline-hover"
            :style="{ left: hover_percent }"
          >
            {{ positionLabel(pointer_seconds) }}
          </div>
        </div>
      </div>
    </div>

    <v-alert
      v-if="recording && tracks.length === 0"
      type="info"
      dense
      class="mb-2"
    >
      This recording has no video streams. CSV export is still available.
    </v-alert>

    <div
      v-if="recording && (tracks.length > 0 || recording.channels.length > 0)"
      class="player-footer mt-3"
    >
      <div class="d-flex align-center caption grey--text text--darken-1 mb-2 flex-wrap">
        <span
          v-if="bytes_downloaded > 0"
          v-tooltip="'Only the parts of the recording you watch are downloaded from the vehicle'"
        >
          {{ downloaded }} downloaded
        </span>
        <span v-if="tracks.length > 1" class="ml-3">
          {{ tracks.length }} streams sharing the same download
        </span>
        <v-spacer />
        <div
          v-if="tracks.length > 0"
          v-tooltip="statistics ? 'Hide detailed statistics' : 'Show detailed statistics'"
          class="d-flex align-center stats-toggle"
        >
          <span class="mr-2">Stats</span>
          <v-switch
            v-model="statistics"
            dense
            hide-details
            color="primary"
            class="mt-0 pt-0"
          />
        </div>
      </div>

      <v-checkbox
        v-model="cut_enabled"
        v-tooltip="'Export only part of the recording. Leave this off to keep the whole file.'"
        dense
        hide-details
        class="mt-0 pt-0 mb-2 cut-toggle"
        :disabled="Boolean(export_progress)"
        label="Cut the data before exporting"
        @change="onCutToggle"
      />

      <div v-if="cut_enabled" class="export-panel pa-3">
        <div class="d-flex align-center caption grey--text text--darken-1">
          <span>{{ positionLabel(0) }}</span>
          <div class="range-wrapper mx-2">
            <v-range-slider
              :value="clip_range"
              :max="duration"
              :step="clip_step"
              :min="0"
              hide-details
              dense
              thumb-label
              color="primary"
              @input="onRangeInput"
              @change="onRangeSettled"
            >
              <template #thumb-label="{ value }">
                {{ positionLabel(value) }}
              </template>
            </v-range-slider>
            <div
              :class="['playhead', $vuetify.theme.dark ? 'grey lighten-1' : 'grey darken-2']"
              :style="playhead_style"
            />
          </div>
          <span>{{ positionLabel(duration) }}</span>
        </div>

        <div class="d-flex align-center flex-wrap mt-1">
          <div class="bound-group mr-4 mb-1">
            <div class="caption grey--text text--darken-1">
              Start
            </div>
            <div class="bound-value primary--text">
              {{ positionLabel(clip_range[0]) }}
            </div>
          </div>
          <div class="bound-group mr-4 mb-1">
            <div class="caption grey--text text--darken-1">
              End
            </div>
            <div class="bound-value primary--text">
              {{ positionLabel(clip_range[1]) }}
            </div>
          </div>
          <div class="bound-group mr-4 mb-1">
            <div class="caption grey--text text--darken-1">
              Duration
            </div>
            <div class="bound-value primary--text">
              {{ clip_duration_label }}
            </div>
          </div>
          <v-spacer />
          <v-btn
            v-tooltip="'Use the whole recording again'"
            small
            text
            class="mr-1"
            @click="resetClip"
          >
            Whole recording
          </v-btn>
          <v-btn
            v-tooltip="'Move the start of the saved part to the playback position'"
            small
            text
            class="mr-1"
            @click="markClipStart"
          >
            Set start to playhead
          </v-btn>
          <v-btn
            v-tooltip="'Move the end of the saved part to the playback position'"
            small
            text
            @click="markClipEnd"
          >
            Set end to playhead
          </v-btn>
        </div>
      </div>

      <div class="d-flex align-center flex-wrap mt-3 action-row">
        <template v-if="export_progress">
          <div class="export-progress flex-grow-1 mr-3 mb-2">
            <div class="d-flex align-center caption mb-1">
              <span>{{ export_status }}</span>
              <v-spacer />
              <span>{{ export_percentage }}%</span>
            </div>
            <v-progress-linear
              :value="export_percentage"
              height="6"
              rounded
              color="primary"
            />
            <div class="caption warning--text mt-1">
              Keep this page open. Leaving now will stop this export.
            </div>
          </div>
          <v-btn
            small
            text
            class="mr-2 mb-2"
            @click="cancelExport"
          >
            Cancel
          </v-btn>
        </template>
        <v-spacer />
        <div class="action-group d-flex flex-wrap justify-end">
          <v-btn
            v-if="recording && recording.channels.length > 0"
            v-tooltip="'Choose topics and save them as CSV. Listing every topic downloads more of the recording.'"
            small
            outlined
            color="primary"
            class="mr-2 mb-2"
            :disabled="Boolean(export_progress)"
            @click="openCsvExport"
          >
            <v-icon small left>
              mdi-table-arrow-down
            </v-icon>
            Export data
          </v-btn>
          <v-btn
            v-tooltip="save_label"
            small
            outlined
            color="primary"
            class="mb-2"
            :loading="Boolean(export_progress)"
            :disabled="Boolean(export_progress) || tracks.length === 0"
            @click="saveMp4"
          >
            <v-icon small left>
              mdi-video-outline
            </v-icon>
            Export video
          </v-btn>
        </div>
      </div>

      <div
        v-if="statistics && leader_stats"
        class="stats-chips d-flex align-center flex-wrap caption grey--text text--darken-1 mt-2"
      >
        <span class="stats-chip mr-3">
          <v-icon x-small class="mr-1">
            mdi-image-multiple-outline
          </v-icon>
          {{ (leader_stats.framesRead ?? 0).toLocaleString() }} frames
        </span>
        <span :class="['stats-chip', 'mr-3', (leader_stats.framesLost ?? 0) > 0 ? 'warning--text' : '']">
          <v-icon x-small class="mr-1">
            mdi-alert-outline
          </v-icon>
          {{ leader_stats.framesLost ?? 0 }} lost
        </span>
        <span :class="['stats-chip', 'mr-3', (leader_stats.framesCorrupt ?? 0) > 0 ? 'error--text' : '']">
          <v-icon x-small class="mr-1">
            mdi-shield-check-outline
          </v-icon>
          {{ leader_stats.framesCorrupt ?? 0 }} corrupt
        </span>
        <span v-if="leader_stats.frameRate" class="stats-chip">
          <v-icon x-small class="mr-1">
            mdi-speedometer
          </v-icon>
          {{ leader_stats.frameRate.toFixed(1) }} fps
        </span>
      </div>
    </div>

    <v-alert
      v-if="page_busy"
      type="warning"
      dense
      class="mt-3 mb-0"
    >
      Keep this page open. This export runs in the browser and will stop if you leave or close this window.
    </v-alert>

    <v-dialog
      v-model="csv_open"
      :fullscreen="$vuetify.breakpoint.xsOnly"
      max-width="800"
      scrollable
      :persistent="csv_busy"
    >
      <v-card>
        <v-card-title class="d-flex align-center">
          Export data
          <v-spacer />
          <v-btn
            v-tooltip="csv_busy ? 'Wait until the export finishes' : 'Close'"
            icon
            small
            color="primary"
            :disabled="csv_busy"
            @click="csv_open = false"
          >
            <v-icon>mdi-close</v-icon>
          </v-btn>
        </v-card-title>
        <v-card-text>
          <div v-if="naming_channels" class="d-flex flex-column align-center py-8 px-4">
            <v-progress-circular
              :indeterminate="naming_percent === null"
              :value="naming_percent ?? 0"
              color="primary"
              size="64"
              width="5"
            >
              <span v-if="naming_percent !== null" class="caption">
                {{ Math.round(naming_percent) }}%
              </span>
            </v-progress-circular>
            <span class="mt-3 caption grey--text text-center">
              Listing every topic in this recording...
            </span>
            <span v-if="naming_status" class="mt-1 caption grey--text text-center">
              {{ naming_status }}
            </span>
          </div>
          <mcap-csv-export
            v-else-if="recording"
            :recording="recording"
            :clip="clip"
            :name="name"
            @error="onCsvError"
            @busy="onCsvBusy"
          />
        </v-card-text>
      </v-card>
    </v-dialog>
  </div>
</template>

<script lang="ts">
import { saveAs } from 'file-saver'
import Vue, { PropType } from 'vue'

import McapCsvExport from '@/components/records/McapCsvExport.vue'
import McapVideoStream from '@/components/records/McapVideoStream.vue'
import {
  atLatestPosition,
  bufferedRangeStyles,
  CLIP_STEP_SECONDS,
  clipDurationLabel,
  clipExportRange,
  coverageEnd,
  exportPercentage,
  filterTracksBySearch,
  formatFrameAge,
  formatPlaybackPosition,
  gridColumnCount,
  gridRowCount,
  gridStyle,
  McapPlaybackViewState,
  McapRecordingPlaybackController,
  mergedVideoCoverage,
  mp4SaveLabel,
  namingPercent,
  namingStatusText,
  openingMessage,
  openingPercent,
  openingStatusText,
  type RecordingIndexSource,
  recordingNameFromUrl,
  timelinePercent,
  timelineRangeStyles,
  trackCoversAt,
  visibleTracks,
} from '@/libs/mcap'
import { prettifySize } from '@/utils/helper_functions'

function emptyView(): McapPlaybackViewState {
  return {
    recording: null,
    tracks: [],
    selectedChannelIds: [],
    videos: {},
    streamStats: {},
    bytesDownloaded: 0,
    error: null,
    opening: true,
    openProgress: null,
    openingBytesPerSecond: 0,
    statistics: false,
    csvOpen: false,
    csvBusy: false,
    namingChannels: false,
    namingProgress: null,
    cutEnabled: false,
    clipRange: [0, 0],
    movedBound: 0,
    position: 0,
    exportProgress: null,
    exportTrackName: null,
    exportTrackIndex: 0,
    exportTrackCount: 0,
    playing: false,
    pendingSeek: null,
    timelineDragging: false,
    pointerOverVideo: false,
    streamSearch: '',
    bufferedRanges: [],
    pointerSeconds: null,
    lastKnownWrittenSize: 0,
  }
}

export default Vue.extend({
  name: 'McapVideoPlayer',
  components: { McapCsvExport, McapVideoStream },
  props: {
    url: { type: String, required: true },
    indexSource: { type: Object as PropType<RecordingIndexSource | undefined>, default: undefined },
    ongoing: { type: Boolean, default: false },
    writtenSizeBytes: { type: Number, default: undefined },
  },
  data() {
    return {
      controller: null as McapRecordingPlaybackController | null,
      view: emptyView(),
      stream_menu: false,
    }
  },
  computed: {
    recording() { return this.view.recording },
    tracks() { return this.view.tracks },
    selected_channel_ids() { return this.view.selectedChannelIds },
    error() { return this.view.error },
    opening() { return this.view.opening },
    open_progress() { return this.view.openProgress },
    opening_bytes_per_second() { return this.view.openingBytesPerSecond },
    csv_busy() { return this.view.csvBusy },
    naming_channels() { return this.view.namingChannels },
    naming_progress() { return this.view.namingProgress },
    clip_range() { return this.view.clipRange },
    position() { return this.view.position },
    export_progress() { return this.view.exportProgress },
    export_track_name() { return this.view.exportTrackName },
    export_track_index() { return this.view.exportTrackIndex },
    export_track_count() { return this.view.exportTrackCount },
    playing() { return this.view.playing },
    stream_search: {
      get(): string { return this.view.streamSearch },
      set(value: string) { this.controller?.setStreamSearch(value) },
    },
    statistics: {
      get(): boolean { return this.view.statistics },
      set(value: boolean) { this.controller?.setStatistics(value) },
    },
    cut_enabled: {
      get(): boolean { return this.view.cutEnabled },
      set(value: boolean) { this.controller?.onCutToggle(value) },
    },
    csv_open: {
      get(): boolean { return this.view.csvOpen },
      set(value: boolean) { this.controller?.setCsvOpen(value) },
    },
    clip_step() { return CLIP_STEP_SECONDS },
    buffered_ranges() { return this.view.bufferedRanges },
    pointer_seconds() { return this.view.pointerSeconds },
    pointer_over_video() { return this.view.pointerOverVideo },
    visible_tracks() { return visibleTracks(this.tracks, this.selected_channel_ids) },
    filtered_tracks() { return filterTracksBySearch(this.tracks, this.stream_search) },
    columns() {
      return gridColumnCount(this.visible_tracks.length, this.$vuetify.breakpoint.smAndDown)
    },
    grid_rows() { return gridRowCount(this.visible_tracks.length, this.columns) },
    grid_style() { return gridStyle(this.columns, this.grid_rows) },
    duration() { return this.recording?.durationSeconds ?? 0 },
    clip() { return clipExportRange(this.cut_enabled, this.clip_range, this.duration) },
    clip_duration_label() { return clipDurationLabel(this.clip_range) },
    save_label() { return mp4SaveLabel(this.tracks.length, this.clip, this.clip_range) },
    page_busy() { return Boolean(this.export_progress) || this.csv_busy },
    export_percentage() {
      return exportPercentage(this.export_progress, this.export_track_index, this.export_track_count)
    },
    export_status(): string {
      const size = prettifySize((this.export_progress?.bytes ?? 0) / 1024)
      if (this.export_track_count > 1 && this.export_track_name) {
        return `Saving ${this.export_track_index + 1}/${this.export_track_count}`
          + ` · ${this.export_track_name} · ${size}`
      }
      return `Saving MP4 · ${size}`
    },
    playhead_style() {
      const fraction = this.duration > 0 ? Math.min(this.position / this.duration, 1) : 0
      return { left: `calc(8px + (100% - 16px) * ${fraction})` }
    },
    bytes_downloaded() { return this.view.bytesDownloaded },
    downloaded() { return prettifySize(this.view.bytesDownloaded / 1024) },
    leader_stats() {
      const [track] = this.visible_tracks
      return track ? this.view.streamStats[track.channelId] ?? null : null
    },
    opening_message() { return openingMessage(this.ongoing) },
    opening_status() {
      return openingStatusText(this.open_progress, this.opening_bytes_per_second, prettifySize)
    },
    opening_percent() { return openingPercent(this.open_progress) },
    naming_status() { return namingStatusText(this.naming_progress, prettifySize) },
    naming_percent() { return namingPercent(this.naming_progress) },
    name() { return recordingNameFromUrl(this.url) },
    video_coverage() { return mergedVideoCoverage(this.visible_tracks) },
    last_video_time() {
      return coverageEnd(this.visible_tracks.length > 0 ? this.visible_tracks : this.tracks)
    },
    at_latest() { return atLatestPosition(this.ongoing, this.last_video_time, this.position) },
    frame_age_label() { return formatFrameAge(this.last_video_time - this.position) },
    playhead_percent() { return timelinePercent(this.position, this.duration) },
    hover_percent() { return timelinePercent(this.pointer_seconds, this.duration) },
    video_styles() { return timelineRangeStyles(this.video_coverage, this.duration) },
    buffered_styles() { return bufferedRangeStyles(this.buffered_ranges, this.duration) },
  },
  watch: {
    page_busy: { immediate: true, handler(busy: boolean) { this.$emit('busy', busy) } },
    csv_open(open: boolean) { this.controller?.setCsvOpen(open) },
    writtenSizeBytes(sizeBytes: number | undefined) {
      if (sizeBytes !== undefined) {
        this.controller?.onWrittenSizeBytes(sizeBytes)
      }
    },
  },
  mounted() {
    this.controller = new McapRecordingPlaybackController({
      url: this.url,
      indexSource: this.indexSource,
      ongoing: this.ongoing,
      writtenSizeBytes: this.writtenSizeBytes,
      callbacks: {
        onState: (state) => { this.view = { ...state } },
        onBusy: (busy) => this.$emit('busy', busy),
        onSummary: (summary) => this.$emit('summary', summary),
        onMp4Saved: (blob, fileName) => saveAs(blob, fileName),
      },
    })
    this.controller.mount().then(() => this.syncStreamControls())
    if (this.writtenSizeBytes !== undefined) {
      this.controller.onWrittenSizeBytes(this.writtenSizeBytes)
    }
  },
  updated() {
    this.syncStreamControls()
  },
  beforeDestroy() {
    this.controller?.destroy()
    this.$emit('busy', false)
  },
  methods: {
    syncStreamControls(): void {
      const streams = this.streamRefs()
      this.controller?.setStreamControls(
        this.visible_tracks.map((track, index) => {
          const stream = streams[index]
          return {
            channelId: track.channelId,
            seek: (seconds) => stream?.seek(seconds),
            play: () => stream?.play(),
            pause: () => stream?.pause(),
          }
        }),
      )
    },
    streamRefs(): { seek: (seconds: number) => void, play: () => void, pause: () => void }[] {
      const streams = this.$refs.stream as
        | { seek: (s: number) => void, play: () => void, pause: () => void }[]
        | { seek: (s: number) => void, play: () => void, pause: () => void }
      if (!streams) return []
      return Array.isArray(streams) ? streams : [streams]
    },
    trackCovers(track: { coverage: { start: number, end: number }[] }, seconds: number): boolean {
      return trackCoversAt(track as never, seconds)
    },
    onStreamReady(channelId: number, video: HTMLVideoElement): void {
      this.controller?.registerVideo(channelId, video)
    },
    onStreamTime(channelId: number, seconds: number): void {
      this.controller?.onStreamTime(channelId, seconds)
    },
    updateBuffered(): void { this.controller?.updateBuffered() },
    onLeaderPlay(): void { this.controller?.onLeaderPlay() },
    onLeaderPause(): void { this.controller?.onLeaderPause() },
    togglePlayback(): void { this.controller?.togglePlayback() },
    skipToLatest(): void { this.controller?.skipToLatest() },
    onTimelineDown(event: PointerEvent): void {
      this.controller?.onTimelinePointer(this.fractionFromEvent(event), 'down')
    },
    onTimelineMove(event: PointerEvent): void {
      this.controller?.onTimelinePointer(this.fractionFromEvent(event), 'move')
    },
    onTimelineUp(event: PointerEvent): void {
      this.controller?.onTimelinePointer(this.fractionFromEvent(event), 'up')
    },
    onTimelineLeave(): void { this.controller?.onTimelineLeave() },
    fractionFromEvent(event: PointerEvent): number {
      const timeline = this.$refs.timeline as HTMLElement
      const rect = timeline.getBoundingClientRect()
      if (rect.width <= 0) return 0
      return (event.clientX - rect.left) / rect.width
    },
    toggleStream(channelId: number): void { this.controller?.toggleStream(channelId) },
    selectAllStreams(): void { this.controller?.selectAllStreams() },
    selectNoStreams(): void { this.controller?.selectNoStreams() },
    handleExtended(): void { this.controller?.handleExtended() },
    positionLabel(seconds: number): string { return formatPlaybackPosition(seconds) },
    onRangeInput(range: number[]): void { this.controller?.onRangeInput(range) },
    onRangeSettled(range: number[]): void { this.controller?.onRangeSettled(range) },
    markClipStart(): void { this.controller?.markClipStart() },
    markClipEnd(): void { this.controller?.markClipEnd() },
    resetClip(): void { this.controller?.resetClip() },
    onCutToggle(enabled: boolean): void { this.controller?.onCutToggle(enabled) },
    onCsvBusy(busy: boolean): void { this.controller?.setCsvBusy(busy) },
    async openCsvExport(): Promise<void> { await this.controller?.openCsvExport() },
    onStats(channelId: number, stats: unknown): void {
      this.controller?.onStreamStats(channelId, stats as never)
    },
    async saveMp4(): Promise<void> {
      await this.controller?.saveMp4(this.name, this.clip)
    },
    cancelExport(): void { this.controller?.cancelExport() },
    onCsvError(message: string): void { this.controller?.setError(message) },
  },
})

</script>

<style scoped>
.player {
  width: 100%;
}

.player-stage {
  background: #000;
  border-radius: 4px;
  overflow: hidden;
}

.stream-grid {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(var(--grid-columns, 1), minmax(0, 1fr));
  padding: 8px 8px 0;
}

.stream-empty {
  color: #e5e7eb;
}

.playback-bar {
  background: #000;
  color: #e5e7eb;
  padding: 4px 12px 12px;
}

.playback-time {
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.stream-picker-btn {
  text-transform: none;
}

.stream-picker {
  min-width: 280px;
  background: #000;
}

.stream-picker-list {
  max-height: 280px;
  overflow-y: auto;
}

.timeline {
  position: relative;
  height: 18px;
  cursor: default;
  touch-action: none;
}

.timeline-over-video {
  cursor: pointer;
}

.timeline-track {
  position: absolute;
  left: 0;
  right: 0;
  top: 8px;
  height: 4px;
  border-radius: 2px;
  background: #2a2a2a;
}

.timeline-video {
  position: absolute;
  top: 8px;
  height: 4px;
  border-radius: 2px;
  background: #6b7280;
  pointer-events: none;
}

.timeline-mark {
  position: absolute;
  top: -9px;
  font-size: 10px;
  line-height: 1;
  color: #fff;
  pointer-events: none;
}

.timeline-mark-start {
  left: 0;
  transform: translateX(-50%);
}

.timeline-mark-end {
  right: 0;
  transform: translateX(50%);
}

.timeline-buffered {
  position: absolute;
  top: 8px;
  height: 4px;
  border-radius: 2px;
  background: #d1d5db;
  pointer-events: none;
}

.timeline-playhead {
  position: absolute;
  top: 5px;
  width: 10px;
  height: 10px;
  margin-left: -5px;
  border-radius: 50%;
  background: #fff;
  pointer-events: none;
}

.timeline-hover {
  position: absolute;
  bottom: 100%;
  z-index: 2;
  padding: 2px 6px;
  margin-bottom: 4px;
  border-radius: 3px;
  background: rgba(0, 0, 0, 0.85);
  color: #fff;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  line-height: 1.2;
  white-space: nowrap;
  pointer-events: none;
  transform: translateX(-50%);
}

.action-group {
  min-width: 0;
}

.export-panel {
  border: 1px solid rgba(128, 128, 128, 0.35);
  border-radius: 8px;
  background: rgba(128, 128, 128, 0.08);
}

.range-wrapper {
  position: relative;
  flex: 1;
}

.playhead {
  position: absolute;
  top: 50%;
  width: 2px;
  height: 14px;
  transform: translate(-1px, -50%);
  opacity: 0.7;
  pointer-events: none;
}

.bound-group {
  min-width: 64px;
}

.bound-value {
  font-variant-numeric: tabular-nums;
  font-weight: 500;
}

.cut-toggle {
  max-width: 100%;
}

.stats-toggle ::v-deep .v-input--selection-controls {
  margin-top: 0;
  padding-top: 0;
}

.export-progress {
  min-width: 0;
}

.stats-chip {
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
}
</style>
