import { listMcapChannels } from '../logic/channels'
import {
  CLIP_STEP_SECONDS,
  coverageEnd,
  coverageStart,
  mergedVideoCoverage,
  playheadHasBufferedMedia,
  trackCoversAt,
} from '../logic/playback-ui'
import type { PrefixScanProgress } from '../logic/reader'
import type { RecordingIndexSource } from '../logic/recording-index'
import type { VideoTrack } from '../logic/video-track'
import { listVideoTracks, timeRangesCover } from '../logic/video-track'
import { exportTrackAsMp4, type Mp4ExportProgress, type Mp4ExportRange } from './export'
import {
  isMediaSourceSupported,
  McapVideoRecording,
  McapVideoStats,
  McapVideoSummary,
  openMcapVideoRecording,
} from './player'

/** How far a stream may drift from the one being controlled before it is nudged back into place. */
const SYNC_TOLERANCE_SECONDS = 0.5
/** `HTMLMediaElement.HAVE_CURRENT_DATA`: the element has a frame for its current position. */
const HAVE_CURRENT_DATA = 2

export interface StreamPlaybackControl {
  channelId: number
  seek(seconds: number): void
  play(): void
  pause(): void
}

export interface McapPlaybackViewState {
  recording: McapVideoRecording | null
  tracks: VideoTrack[]
  selectedChannelIds: number[]
  videos: Record<number, HTMLVideoElement>
  streamStats: Record<number, McapVideoStats>
  bytesDownloaded: number
  error: string | null
  opening: boolean
  openProgress: PrefixScanProgress | null
  openingBytesPerSecond: number
  statistics: boolean
  csvOpen: boolean
  csvBusy: boolean
  namingChannels: boolean
  namingProgress: { named: number, total: number, bytes: number, bytesPerSecond: number } | null
  cutEnabled: boolean
  clipRange: [number, number]
  movedBound: number
  position: number
  exportProgress: Mp4ExportProgress | null
  exportTrackName: string | null
  exportTrackIndex: number
  exportTrackCount: number
  playing: boolean
  pendingSeek: number | null
  timelineDragging: boolean
  pointerOverVideo: boolean
  streamSearch: string
  bufferedRanges: { start: number, end: number }[]
  pointerSeconds: number | null
  lastKnownWrittenSize: number
}

export interface McapPlaybackCallbacks {
  onState: (state: McapPlaybackViewState) => void
  onBusy: (busy: boolean) => void
  onSummary: (summary: McapVideoSummary) => void
  onMp4Saved: (blob: Blob, fileName: string) => void
}

export interface McapPlaybackControllerOptions {
  url: string
  indexSource?: RecordingIndexSource
  ongoing: boolean
  writtenSizeBytes?: number
  callbacks: McapPlaybackCallbacks
}

export class McapRecordingPlaybackController {
  private readonly options: McapPlaybackControllerOptions

  private openController = new AbortController()

  private exportController: AbortController | null = null

  private namingController: AbortController | null = null

  private streamControls: StreamPlaybackControl[] = []

  private state: McapPlaybackViewState

  constructor(options: McapPlaybackControllerOptions) {
    this.options = options
    this.state = this.emptyState()
  }

  getState(): McapPlaybackViewState {
    return this.state
  }

  private emptyState(): McapPlaybackViewState {
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
      lastKnownWrittenSize: this.options.writtenSizeBytes ?? 0,
    }
  }

  private patch(partial: Partial<McapPlaybackViewState>): void {
    this.state = { ...this.state, ...partial }
    this.options.callbacks.onState({ ...this.state })
    this.options.callbacks.onBusy(Boolean(this.state.exportProgress) || this.state.csvBusy)
  }

  destroy(): void {
    this.openController.abort()
    this.exportController?.abort()
    this.namingController?.abort()
    this.options.callbacks.onBusy(false)
  }

  setStreamControls(controls: StreamPlaybackControl[]): void {
    this.streamControls = controls
  }

  registerVideo(channelId: number, video: HTMLVideoElement): void {
    this.patch({ videos: { ...this.state.videos, [channelId]: video } })
  }

  setCsvOpen(open: boolean): void {
    this.patch({ csvOpen: open })
    if (!open) {
      this.namingController?.abort()
    }
  }

  setCsvBusy(busy: boolean): void {
    this.patch({ csvBusy: busy })
  }

  setStatistics(enabled: boolean): void {
    this.patch({ statistics: enabled })
  }

  setStreamSearch(search: string): void {
    this.patch({ streamSearch: search })
  }

  setSelectedChannelIds(channelIds: number[]): void {
    this.patch({ selectedChannelIds: [...channelIds] })
  }

  selectAllStreams(): void {
    this.patch({ selectedChannelIds: this.state.tracks.map((track) => track.channelId) })
  }

  selectNoStreams(): void {
    this.patch({ selectedChannelIds: [] })
  }

  toggleStream(channelId: number): void {
    const { selectedChannelIds, tracks } = this.state
    if (selectedChannelIds.includes(channelId)) {
      this.patch({ selectedChannelIds: selectedChannelIds.filter((selected) => selected !== channelId) })
      return
    }
    this.patch({
      selectedChannelIds: tracks
        .map((track) => track.channelId)
        .filter((selected) => selected === channelId || selectedChannelIds.includes(selected)),
    })
  }

  onWrittenSizeBytes(sizeBytes: number): void {
    if (!this.options.ongoing || sizeBytes <= this.state.lastKnownWrittenSize) {
      return
    }
    this.patch({ lastKnownWrittenSize: sizeBytes })
    this.extendWrittenPrefix().catch(() => undefined)
  }

  async mount(): Promise<void> {
    try {
      const startedAt = Date.now()
      const recording = await openMcapVideoRecording(this.options.url, {
        indexSource: this.options.indexSource,
        signal: this.openController.signal,
        onProgress: (progress) => {
          const elapsed = (Date.now() - startedAt) / 1000
          this.patch({
            openProgress: progress,
            openingBytesPerSecond: elapsed >= 0.25 ? progress.bytesRead / elapsed : 0,
          })
        },
      })
      const { tracks } = recording
      const clipRange: [number, number] = [0, recording.durationSeconds]
      const position = this.options.ongoing
        ? coverageEnd(tracks) || recording.durationSeconds
        : coverageStart(tracks)
      this.patch({
        recording,
        tracks,
        selectedChannelIds: tracks.map((track) => track.channelId),
        clipRange,
        position,
      })
      this.emitSummary()
      recording.reader.loadRemainingChunkIndexes(this.openController.signal, () => {
        this.onExtended()
      }).catch((error) => {
        if (!this.openController.signal.aborted) {
          console.warn('Failed to finish loading the recording index:', error)
        }
      })
      if (tracks.length > 0 && !isMediaSourceSupported()) {
        this.patch({
          error: 'This browser cannot play recordings, as it does not support Media Source Extensions.',
        })
      }
    } catch (error) {
      if (!this.openController.signal.aborted) {
        this.patch({ error: error instanceof Error ? error.message : String(error) })
      }
    }
    this.patch({ opening: false })
  }

  onStreamStats(channelId: number, stats: McapVideoStats): void {
    this.patch({
      streamStats: { ...this.state.streamStats, [channelId]: stats },
      bytesDownloaded: stats.bytesDownloaded,
    })
  }

  onStreamTime(channelId: number, seconds: number): void {
    if (this.state.timelineDragging) {
      return
    }
    const clockId = this.clockChannelId()
    if (channelId !== clockId) {
      return
    }
    const duration = this.state.recording?.durationSeconds ?? 0
    if (this.state.pendingSeek !== null) {
      const leader = this.clockVideo()
      const target = this.state.pendingSeek
      if (leader && !leader.seeking && playheadHasBufferedMedia(leader)) {
        this.patch({ pendingSeek: null, position: this.playbackPosition() })
        return
      }
      this.patch({ position: target })
      return
    }
    this.patch({ position: Math.min(seconds, duration) })
  }

  updateBuffered(): void {
    const leader = this.clockVideo()
    if (!leader) {
      this.patch({ bufferedRanges: [] })
      return
    }
    const ranges = []
    for (let index = 0; index < leader.buffered.length; index += 1) {
      ranges.push({ start: leader.buffered.start(index), end: leader.buffered.end(index) })
    }
    this.patch({ bufferedRanges: ranges })
  }

  onLeaderPlay(): void {
    this.patch({ playing: true })
    this.syncFollowers()
  }

  onLeaderPause(): void {
    const leader = this.clockVideo()
    if (leader && !leader.paused) {
      return
    }
    this.patch({ playing: false })
    this.syncFollowers()
  }

  togglePlayback(): void {
    const coverage = mergedVideoCoverage(this.visibleTracks())
    if (this.state.playing) {
      this.streamControls.forEach((stream) => stream.pause())
      this.patch({ playing: false })
      return
    }
    if (!timeRangesCover(coverage, this.state.position) && coverage.length > 0) {
      const last = coverageEnd(this.state.tracks)
      this.seekTo(this.options.ongoing ? last : coverage[0].start)
      return
    }
    this.streamControls.forEach((stream) => stream.play())
    this.patch({ playing: true })
  }

  seekTo(seconds: number): void {
    const coverage = mergedVideoCoverage(this.visibleTracks())
    if (!timeRangesCover(coverage, seconds)) {
      return
    }
    const duration = this.state.recording?.durationSeconds ?? 0
    const target = Math.max(0, Math.min(seconds, duration))
    this.patch({ pendingSeek: target, position: target })
    for (const stream of this.streamControls) {
      stream.seek(target)
      stream.play()
    }
    this.patch({ playing: true })
  }

  skipToLatest(): void {
    this.seekTo(coverageEnd(this.visibleTracks().length > 0 ? this.visibleTracks() : this.state.tracks))
  }

  onTimelinePointer(fraction: number, phase: 'move' | 'down' | 'up'): void {
    const duration = this.state.recording?.durationSeconds ?? 0
    const seconds = Math.max(0, Math.min(1, fraction)) * duration
    const coverage = mergedVideoCoverage(this.visibleTracks())
    const overVideo = timeRangesCover(coverage, seconds)
    if (phase === 'down') {
      this.patch({ pointerSeconds: seconds, pointerOverVideo: overVideo })
      if (!overVideo) {
        return
      }
      this.patch({
        timelineDragging: true,
        pendingSeek: seconds,
        position: seconds,
      })
      return
    }
    if (phase === 'move') {
      this.patch({ pointerSeconds: seconds, pointerOverVideo: overVideo })
      if (!this.state.timelineDragging || !overVideo) {
        return
      }
      this.patch({ pendingSeek: seconds, position: seconds })
      return
    }
    if (this.state.timelineDragging) {
      this.patch({ timelineDragging: false })
      this.seekTo(seconds)
    }
  }

  onTimelineLeave(): void {
    if (this.state.timelineDragging) {
      return
    }
    this.patch({ pointerSeconds: null, pointerOverVideo: false })
  }

  onRangeInput(range: number[]): void {
    const movedBound = Math.abs(range[0] - this.state.clipRange[0]) >= Math.abs(range[1] - this.state.clipRange[1])
      ? 0
      : 1
    this.patch({ clipRange: [range[0], range[1]], movedBound })
  }

  onRangeSettled(range: number[]): void {
    const target = range[this.state.movedBound]
    if (Math.abs(this.state.position - target) > CLIP_STEP_SECONDS) {
      this.seekTo(target)
    }
  }

  markClipStart(): void {
    const [, end] = this.state.clipRange
    const at = this.playbackPosition()
    const duration = this.state.recording?.durationSeconds ?? 0
    this.patch({ clipRange: [at, end > at ? end : duration] })
  }

  markClipEnd(): void {
    const [start] = this.state.clipRange
    const at = this.playbackPosition()
    this.patch({ clipRange: [start < at ? start : 0, at] })
  }

  resetClip(): void {
    const duration = this.state.recording?.durationSeconds ?? 0
    this.patch({ clipRange: [0, duration] })
  }

  onCutToggle(enabled: boolean): void {
    this.patch({ cutEnabled: enabled })
    if (enabled && this.state.clipRange[1] <= this.state.clipRange[0]) {
      this.resetClip()
    }
  }

  async openCsvExport(): Promise<void> {
    this.patch({ csvOpen: true })
    const { recording } = this.state
    if (!recording || recording.reader.unnamedChannelCount === 0) {
      return
    }
    const total = recording.channels.length + recording.reader.unnamedChannelCount
    const controller = new AbortController()
    this.namingController = controller
    this.patch({
      namingProgress: {
        named: recording.channels.length, total, bytes: 0, bytesPerSecond: 0,
      },
      namingChannels: true,
    })
    const startedAt = Date.now()
    let startedBytes: number | null = null
    try {
      const named = await recording.reader.nameRemainingChannels(controller.signal, (progress) => {
        startedBytes ??= progress.bytesRead
        const elapsed = (Date.now() - startedAt) / 1000
        const bytes = progress.bytesRead - startedBytes
        this.patch({
          namingProgress: {
            named: total - recording.reader.unnamedChannelCount,
            total,
            bytes,
            bytesPerSecond: elapsed >= 0.25 ? bytes / elapsed : 0,
          },
        })
      })
      if (named) {
        recording.channels = listMcapChannels(recording.reader)
        this.patch({ recording })
      }
    } catch (error) {
      if (!controller.signal.aborted) {
        this.patch({ error: error instanceof Error ? error.message : String(error) })
      }
    }
    if (this.namingController === controller) {
      this.patch({ namingChannels: false, namingProgress: null })
      this.namingController = null
    }
  }

  async saveMp4(recordingName: string, clip: Mp4ExportRange | null): Promise<void> {
    const { recording, tracks, exportProgress } = this.state
    if (!recording || tracks.length === 0 || exportProgress) {
      return
    }
    const controller = new AbortController()
    const end = Math.min(clip?.endSeconds ?? Infinity, recording.durationSeconds)
    const durationSeconds = Math.max(end - (clip?.startSeconds ?? 0), 0)
    this.exportController = controller
    this.patch({ exportTrackCount: tracks.length })
    try {
      for (let index = 0; index < tracks.length; index += 1) {
        if (controller.signal.aborted) {
          return
        }
        const track = tracks[index]
        this.patch({
          exportTrackIndex: index,
          exportTrackName: track.name,
          exportProgress: { seconds: 0, durationSeconds, bytes: 0 },
        })
        // eslint-disable-next-line no-await-in-loop
        const file = await exportTrackAsMp4(recording, track, {
          range: clip ?? undefined,
          signal: controller.signal,
          onProgress: (progress) => {
            this.patch({ exportProgress: progress })
          },
        })
        const endLabel = Number.isFinite(clip?.endSeconds) ? `${Math.round(clip?.endSeconds ?? 0)}s` : 'end'
        const fileName = clip
          ? `${recordingName}-${track.name}-${Math.round(clip.startSeconds)}s-${endLabel}.mp4`
          : `${recordingName}-${track.name}.mp4`
        this.options.callbacks.onMp4Saved(file, fileName)
      }
    } catch (error) {
      if (!(error instanceof Error) || error.name !== 'AbortError') {
        this.patch({ error: error instanceof Error ? error.message : String(error) })
      }
    } finally {
      this.exportController = null
      this.patch({
        exportProgress: null,
        exportTrackName: null,
        exportTrackIndex: 0,
        exportTrackCount: 0,
      })
    }
  }

  cancelExport(): void {
    this.exportController?.abort()
  }

  setError(message: string): void {
    this.patch({ error: message })
  }

  handleExtended(): void {
    this.onExtended()
  }

  visibleTracks(): VideoTrack[] {
    const selected = new Set(this.state.selectedChannelIds)
    return this.state.tracks.filter((track) => selected.has(track.channelId))
  }

  private async extendWrittenPrefix(): Promise<void> {
    if (!this.state.recording) {
      return
    }
    if (await this.state.recording.reader.extendWrittenPrefix(this.openController.signal)) {
      this.onExtended()
    }
  }

  private onExtended(): void {
    const { recording } = this.state
    if (!recording) {
      return
    }
    const { reader } = recording
    recording.durationSeconds = Number(reader.summary.endTime - recording.startTime) / 1e9
    const listed = listVideoTracks(reader)
    const selected = new Set(this.state.selectedChannelIds)
    const known = new Set(this.state.tracks.map((track) => track.channelId))
    for (const track of listed) {
      if (!known.has(track.channelId)) {
        selected.add(track.channelId)
      }
    }
    this.patch({
      recording,
      tracks: listed,
      selectedChannelIds: listed
        .map((track) => track.channelId)
        .filter((channelId) => selected.has(channelId)),
      clipRange: [this.state.clipRange[0], recording.durationSeconds],
    })
    this.emitSummary()
  }

  private emitSummary(): void {
    const { recording } = this.state
    if (!recording) {
      return
    }
    this.options.callbacks.onSummary({
      durationSeconds: recording.durationSeconds,
      started: Number(recording.startTime) / 1e9,
      ended: Number(recording.startTime) / 1e9 + recording.durationSeconds,
      tracks: this.state.tracks,
      channels: recording.channels,
      bytesRead: recording.reader.source.bytesRead,
    })
  }

  private clockChannelId(): number | null {
    const available = this.visibleTracks().find((track) => trackCoversAt(track, this.state.position))
    return available?.channelId ?? this.visibleTracks()[0]?.channelId ?? null
  }

  private clockVideo(): HTMLVideoElement | null {
    const channelId = this.clockChannelId()
    return channelId === null ? null : this.state.videos[channelId] ?? null
  }

  private playbackPosition(): number {
    const leader = this.clockVideo()
    const duration = this.state.recording?.durationSeconds ?? 0
    return Math.min(leader?.currentTime ?? this.state.position, duration)
  }

  private syncFollowers(): void {
    const leader = this.clockVideo()
    if (!leader || leader.seeking || leader.readyState < HAVE_CURRENT_DATA) {
      return
    }
    for (const track of this.visibleTracks()) {
      const follower = this.state.videos[track.channelId]
      if (!follower || follower === leader || !trackCoversAt(track, leader.currentTime)) {
        continue
      }
      if (follower.playbackRate !== leader.playbackRate) {
        follower.playbackRate = leader.playbackRate
      }
      const drifted = Math.abs(follower.currentTime - leader.currentTime) > SYNC_TOLERANCE_SECONDS
      if (drifted && !follower.seeking) {
        follower.currentTime = leader.currentTime
      }
      if (leader.paused && !follower.paused) {
        follower.pause()
      } else if (!leader.paused && follower.paused) {
        follower.play().catch(() => undefined)
      }
    }
  }
}
