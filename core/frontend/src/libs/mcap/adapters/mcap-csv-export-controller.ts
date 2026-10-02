import type { McapRecordingChannel } from '../logic/channels'
import { defaultSelectedChannelIds } from '../logic/channels'
import type { CsvExportProgress } from '../logic/csv'
import { exportChannelsAsCsv } from '../logic/csv'
import type { Mp4ExportRange } from './export'
import type { McapVideoRecording } from './player'

export interface McapCsvExportState {
  selected: McapRecordingChannel[]
  search: string
  exportProgress: CsvExportProgress | null
}

export interface McapCsvExportCallbacks {
  onState: (state: McapCsvExportState) => void
  onBusy: (busy: boolean) => void
  onError: (message: string) => void
  onSaved: (blob: Blob, fileName: string) => void
}

export class McapCsvExportController {
  private state: McapCsvExportState

  private exportController: AbortController | null = null

  constructor(
    private readonly recording: McapVideoRecording,
    private readonly clip: Mp4ExportRange | null,
    private readonly name: string,
    private readonly callbacks: McapCsvExportCallbacks,
  ) {
    this.state = {
      selected: [],
      search: '',
      exportProgress: null,
    }
  }

  getState(): McapCsvExportState {
    return this.state
  }

  private emit(): void {
    this.callbacks.onState({ ...this.state, selected: [...this.state.selected] })
    this.callbacks.onBusy(Boolean(this.state.exportProgress))
  }

  mount(): void {
    this.selectDefaults()
  }

  destroy(): void {
    this.exportController?.abort()
  }

  setSearch(search: string): void {
    this.state = { ...this.state, search }
    this.emit()
  }

  setSelected(selected: McapRecordingChannel[]): void {
    this.state = { ...this.state, selected: [...selected] }
    this.emit()
  }

  selectAll(): void {
    this.setSelected([...this.recording.channels])
  }

  selectNone(): void {
    this.setSelected([])
  }

  selectDefaults(): void {
    const ids = new Set(defaultSelectedChannelIds(this.recording.channels))
    this.setSelected(this.recording.channels.filter((channel) => ids.has(channel.channelId)))
  }

  cancelExport(): void {
    this.exportController?.abort()
  }

  async saveCsv(fileName: string): Promise<void> {
    if (this.state.exportProgress || this.state.selected.length === 0) {
      return
    }
    const controller = new AbortController()
    const { clip } = this
    const end = Math.min(clip?.endSeconds ?? Infinity, this.recording.durationSeconds)
    const durationSeconds = Math.max(end - (clip?.startSeconds ?? 0), 0)
    this.exportController = controller
    this.state = {
      ...this.state,
      exportProgress: {
        seconds: 0, durationSeconds, bytes: 0, messages: 0,
      },
    }
    this.emit()
    try {
      const file = await exportChannelsAsCsv(
        this.recording,
        this.state.selected.map((channel) => channel.channelId),
        {
          range: clip ?? undefined,
          signal: controller.signal,
          onProgress: (progress) => {
            this.state = { ...this.state, exportProgress: progress }
            this.emit()
          },
        },
      )
      this.callbacks.onSaved(file, fileName)
    } catch (error) {
      if (!(error instanceof Error) || error.name !== 'AbortError') {
        this.callbacks.onError(error instanceof Error ? error.message : String(error))
      }
    } finally {
      this.exportController = null
      this.state = { ...this.state, exportProgress: null }
      this.emit()
    }
  }
}
