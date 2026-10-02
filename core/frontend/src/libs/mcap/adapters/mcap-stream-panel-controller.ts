import type { VideoTrack } from '../logic/video-track'
import { McapVideoPlayer, type McapVideoRecording, type McapVideoStats } from './player'

export interface McapStreamPanelState {
  stats: Partial<McapVideoStats>
  error: string | null
  loading: boolean
  waiting: boolean
  loadingMessage: string
}

export interface McapStreamPanelCallbacks {
  onState: (state: McapStreamPanelState) => void
  onStats?: (stats: McapVideoStats) => void
  onTimeUpdate?: (seconds: number) => void
  onPlay?: () => void
  onPause?: () => void
  onProgress?: () => void
  onExtended?: () => void
  onError?: (error: Error) => void
}

/** Owns one per-stream MSE player instance for the multi-stream recording view. */
export class McapStreamPanelController {
  private player: McapVideoPlayer | null = null

  private state: McapStreamPanelState

  constructor(
    private readonly recording: McapVideoRecording,
    private readonly track: VideoTrack,
    private readonly ongoing: boolean,
    private readonly callbacks: McapStreamPanelCallbacks,
  ) {
    this.state = {
      stats: {},
      error: null,
      loading: true,
      waiting: false,
      loadingMessage: ongoing ? 'Loading latest frame...' : 'Loading video...',
    }
  }

  getState(): McapStreamPanelState {
    return this.state
  }

  private emit(): void {
    this.callbacks.onState({ ...this.state })
  }

  setAvailable(available: boolean, video: HTMLVideoElement | null, position: number): void {
    if (available && video) {
      this.open(video, position)
      return
    }
    this.close()
  }

  seek(seconds: number): void {
    this.player?.seek(seconds)
  }

  play(video: HTMLVideoElement): void {
    this.player?.setWantPlaying(true)
    video.play().catch(() => undefined)
  }

  pause(video: HTMLVideoElement): void {
    this.player?.setWantPlaying(false)
    video.pause()
  }

  handleTimeUpdate(video: HTMLVideoElement): void {
    this.callbacks.onTimeUpdate?.(video.currentTime)
  }

  handlePlay(): void {
    this.callbacks.onPlay?.()
  }

  handlePause(available: boolean): void {
    if (!available) {
      return
    }
    if (this.ongoing && this.player?.wantPlaying) {
      return
    }
    this.callbacks.onPause?.()
  }

  handleProgress(): void {
    this.callbacks.onProgress?.()
  }

  destroy(): void {
    this.close()
  }

  private open(video: HTMLVideoElement, position: number): void {
    if (this.player) {
      this.player.seek(position)
      return
    }
    this.state = { ...this.state, error: null, loading: true }
    this.emit()
    this.player = new McapVideoPlayer(video, this.recording, this.track, {
      startSeconds: position,
      startAtEnd: this.ongoing,
      follow: this.ongoing,
      onStats: (stats) => {
        this.state = {
          ...this.state,
          stats,
          loading: stats.loading,
          waiting: stats.waiting,
        }
        this.emit()
        this.callbacks.onStats?.(stats)
      },
      onError: (error) => {
        this.state = { ...this.state, loading: false, error: error.message }
        this.emit()
        this.callbacks.onError?.(error)
      },
      onExtended: () => this.callbacks.onExtended?.(),
    })
    this.player.start().catch((error) => {
      this.state = {
        ...this.state,
        loading: false,
        error: error instanceof Error ? error.message : String(error),
      }
      this.emit()
    })
  }

  private close(): void {
    this.player?.destroy()
    this.player = null
    this.state = { ...this.state, loading: false, waiting: false }
    this.emit()
  }
}
