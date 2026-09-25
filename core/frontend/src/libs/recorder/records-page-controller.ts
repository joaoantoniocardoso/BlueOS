import {
  deleteCachedThumbnail,
  extractMcapThumbnail,
  getCachedThumbnail,
  McapVideoSummary,
  readMcapVideoSummary,
  setCachedThumbnail,
} from '@/libs/mcap'

import type { RecorderClient } from './client'
import { SNAPSHOT_WAIT_TIMEOUT_MS } from './constants'
import type { LibraryRecording, RecordingOperationEvent } from './types'
import {
  browsingAllowedWhileArmed,
  isSnapshotOperationForPath,
  operationFailureMessage,
  readySnapshotDownloadPath,
  snapshotDownloadPath,
} from './view-logic'

export interface RecordsPageState {
  vehicleArmed: boolean
  recordings: LibraryRecording[]
  libraryLoading: boolean
  recorderServiceRunning: boolean
  playerOpen: boolean
  playerBusy: boolean
  activeRecord: LibraryRecording | null
  selectedDate: string
  layout: 'cards' | 'table'
  summaries: Record<string, McapVideoSummary>
  thumbnails: Record<string, string>
  selectedPaths: string[]
  deleteDialog: boolean
  deleteTargets: LibraryRecording[]
  repairDialog: boolean
  repairTargets: LibraryRecording[]
  bulkDownloading: boolean
  bulkRepairing: boolean
}

export interface RecordsPageCallbacks {
  onState: (state: RecordsPageState) => void
  onNotifyError: (type: string, message: string) => void
  onTriggerDownload: (url: string, fileName: string) => void
}

interface SnapshotWaiter {
  resolve: (outputPath: string) => void
  reject: (error: Error) => void
  timeoutId: ReturnType<typeof setTimeout>
}

export class RecordsPageController {
  private readonly recorder: RecorderClient

  private readonly callbacks: RecordsPageCallbacks

  private state: RecordsPageState

  private summaryController: AbortController | null = null

  private thumbnailController: AbortController | null = null

  private snapshotWaiters: Record<string, SnapshotWaiter> = {}

  constructor(recorder: RecorderClient, callbacks: RecordsPageCallbacks, layout: 'cards' | 'table') {
    this.recorder = recorder
    this.callbacks = callbacks
    this.state = {
      vehicleArmed: false,
      recordings: [],
      libraryLoading: true,
      recorderServiceRunning: true,
      playerOpen: false,
      playerBusy: false,
      activeRecord: null,
      selectedDate: '',
      layout,
      summaries: {},
      thumbnails: {},
      selectedPaths: [],
      deleteDialog: false,
      deleteTargets: [],
      repairDialog: false,
      repairTargets: [],
      bulkDownloading: false,
      bulkRepairing: false,
    }
  }

  getState(): RecordsPageState {
    return this.state
  }

  private patch(partial: Partial<RecordsPageState>): void {
    this.state = { ...this.state, ...partial }
    this.callbacks.onState({ ...this.state })
  }

  destroy(): void {
    this.pauseNetworkActivity()
    for (const url of Object.values(this.state.thumbnails)) {
      URL.revokeObjectURL(url)
    }
    for (const waiter of Object.values(this.snapshotWaiters)) {
      clearTimeout(waiter.timeoutId)
      waiter.reject(new Error('Records page closed'))
    }
    this.snapshotWaiters = {}
  }

  setRecorderServiceRunning(running: boolean): void {
    if (running) {
      this.patch({ recorderServiceRunning: true })
      return
    }
    for (const waiter of Object.values(this.snapshotWaiters)) {
      clearTimeout(waiter.timeoutId)
      waiter.reject(new Error('Recorder service is not running'))
    }
    this.snapshotWaiters = {}
    this.patch({
      recorderServiceRunning: false,
      libraryLoading: false,
      recordings: [],
      bulkDownloading: false,
    })
  }

  setLibrary(files: LibraryRecording[]): void {
    const known = new Set(files.map((file) => file.path))
    const selectedPaths = this.state.selectedPaths.filter((path) => known.has(path))
    this.patch({ recordings: files, libraryLoading: false, selectedPaths })
    this.resolveSnapshotWaiters()
    if (this.browsingAllowed()) {
      this.loadSummaries().catch(() => undefined)
    }
    this.notifyActiveRecordingGrowth(files)
  }

  setVehicleArmed(armed: boolean): void {
    this.patch({ vehicleArmed: armed })
    if (this.browsingAllowed()) {
      this.loadSummaries().catch(() => undefined)
      return
    }
    this.pauseNetworkActivity()
  }

  browsingAllowed(): boolean {
    return browsingAllowedWhileArmed(this.state.vehicleArmed)
  }

  setLayout(layout: 'cards' | 'table'): void {
    this.patch({ layout })
  }

  setSelectedDate(selectedDate: string): void {
    this.patch({ selectedDate })
  }

  setPlayerBusy(busy: boolean): void {
    this.patch({ playerBusy: busy })
  }

  onPlayerSummary(summary: McapVideoSummary, path: string): void {
    this.patch({ summaries: { ...this.state.summaries, [path]: summary } })
  }

  recordingStreamUrl(file: LibraryRecording): string {
    return this.recorder.recordingUrl(file.path)
  }

  recordingIndexSource(file: LibraryRecording): ReturnType<RecorderClient['indexSource']> {
    return this.recorder.indexSource(file.path)
  }

  activeWrittenSizeBytes(): number | undefined {
    const file = this.state.activeRecord
    if (!file || file.state !== 'recording') {
      return undefined
    }
    const current = this.state.recordings.find((entry) => entry.path === file.path)
    return current?.size_bytes
  }

  handleOperation(event: RecordingOperationEvent): void {
    const failure = operationFailureMessage(event, event.path)
    if (failure) {
      this.callbacks.onNotifyError('RECORDS_OPERATION_FAILED', failure)
    }
    const waiter = this.snapshotWaiters[event.path]
    if (waiter && isSnapshotOperationForPath(event, event.path)) {
      const outputPath = snapshotDownloadPath(event)
      delete this.snapshotWaiters[event.path]
      clearTimeout(waiter.timeoutId)
      if (outputPath) {
        waiter.resolve(outputPath)
      } else {
        waiter.reject(new Error(event.error || 'Snapshot failed'))
      }
    }
  }

  pauseNetworkActivity(): void {
    this.thumbnailController?.abort()
    this.thumbnailController = null
    this.summaryController?.abort()
    this.summaryController = null
    if (this.state.playerOpen) {
      this.patch({
        playerOpen: false,
        activeRecord: null,
        playerBusy: false,
      })
    }
  }

  setSelectedPaths(selectedPaths: string[]): void {
    this.patch({ selectedPaths: [...selectedPaths] })
  }

  askDelete(targets: LibraryRecording[]): void {
    this.patch({ deleteTargets: targets, deleteDialog: true })
  }

  clearDeleteDialog(): void {
    this.patch({ deleteDialog: false, deleteTargets: [] })
  }

  askRepair(targets: LibraryRecording[]): void {
    this.patch({ repairTargets: targets, repairDialog: true })
  }

  clearRepairDialog(): void {
    this.patch({ repairDialog: false, repairTargets: [] })
  }

  openPlayer(file: LibraryRecording): void {
    this.thumbnailController?.abort()
    this.patch({ activeRecord: file, playerOpen: true })
  }

  closePlayer(): void {
    this.patch({ playerOpen: false, activeRecord: null })
    this.loadThumbnails().catch(() => undefined)
  }

  async confirmDelete(targets: LibraryRecording[]): Promise<void> {
    for (const file of targets) {
      // eslint-disable-next-line no-await-in-loop
      await this.deleteRecording(file)
    }
  }

  async confirmRepair(targets: LibraryRecording[]): Promise<void> {
    this.patch({ bulkRepairing: true })
    try {
      for (const file of targets) {
        // eslint-disable-next-line no-await-in-loop
        await this.repairRecording(file)
      }
    } finally {
      this.patch({ bulkRepairing: false })
    }
  }

  async repairRecording(file: LibraryRecording): Promise<void> {
    const result = await this.recorder.repairRecording(file.path)
    if (!result.accepted) {
      this.callbacks.onNotifyError('RECORDS_REPAIR_REJECTED', result.reason)
    }
  }

  async cancelRepair(file: LibraryRecording): Promise<void> {
    const result = await this.recorder.cancelRepair(file.path)
    if (!result.accepted) {
      this.callbacks.onNotifyError('RECORDS_CANCEL_REPAIR_REJECTED', result.reason)
    }
  }

  async deleteRecording(file: LibraryRecording): Promise<void> {
    await deleteCachedThumbnail({
      path: file.path,
      sizeBytes: file.size_bytes,
      modified: file.created,
    })
    this.forgetThumbnail(file.path)
    this.forgetSummary(file.path)
    if (this.state.activeRecord?.path === file.path) {
      this.patch({ playerOpen: false, activeRecord: null, playerBusy: false })
    }
    const result = await this.recorder.deleteRecording(file.path)
    if (!result.accepted) {
      this.callbacks.onNotifyError('RECORDS_DELETE_REJECTED', result.reason)
    }
  }

  async downloadRecording(file: LibraryRecording, canDownload: boolean): Promise<void> {
    if (!canDownload) {
      return
    }
    let downloadPath = file.path
    if (file.state === 'recording') {
      const result = await this.recorder.snapshotRecording(file.path)
      if (!result.accepted) {
        this.callbacks.onNotifyError('RECORDS_SNAPSHOT_REJECTED', result.reason)
        return
      }
      try {
        downloadPath = await this.waitForSnapshot(file.path)
      } catch (error) {
        this.patch({ bulkDownloading: false })
        this.callbacks.onNotifyError(
          'RECORDS_SNAPSHOT_FAILED',
          error instanceof Error ? error.message : String(error),
        )
        return
      }
    }
    const name = downloadPath.split('/').pop() ?? file.name
    this.callbacks.onTriggerDownload(this.recorder.recordingUrl(downloadPath), name)
  }

  async downloadSelected(files: LibraryRecording[]): Promise<void> {
    this.patch({ bulkDownloading: true })
    try {
      for (const file of files) {
        // eslint-disable-next-line no-await-in-loop
        await this.downloadRecording(file, true)
      }
    } finally {
      this.patch({ bulkDownloading: false })
    }
  }

  private waitForSnapshot(path: string): Promise<string> {
    const existing = readySnapshotDownloadPath(path, this.state.recordings)
    if (existing) {
      return Promise.resolve(existing)
    }
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        delete this.snapshotWaiters[path]
        reject(new Error('Timed out waiting for the snapshot file'))
      }, SNAPSHOT_WAIT_TIMEOUT_MS)
      this.snapshotWaiters[path] = { resolve, reject, timeoutId }
    })
  }

  private resolveSnapshotWaiters(): void {
    for (const [path, waiter] of Object.entries(this.snapshotWaiters)) {
      const outputPath = readySnapshotDownloadPath(path, this.state.recordings)
      if (!outputPath) {
        continue
      }
      delete this.snapshotWaiters[path]
      clearTimeout(waiter.timeoutId)
      waiter.resolve(outputPath)
    }
  }

  private notifyActiveRecordingGrowth(files: LibraryRecording[]): void {
    const active = this.state.activeRecord
    if (!active || active.state !== 'recording') {
      return
    }
    const updated = files.find((file) => file.path === active.path)
    if (updated && updated.size_bytes !== active.size_bytes) {
      this.patch({ activeRecord: updated })
    }
  }

  private readyFiles(): LibraryRecording[] {
    return this.state.recordings.filter((file) => file.state === 'ready')
  }

  private async loadSummaries(): Promise<void> {
    if (!this.browsingAllowed()) {
      return
    }
    this.summaryController?.abort()
    const controller = new AbortController()
    this.summaryController = controller
    const pending = this.readyFiles().filter((file) => !this.state.summaries[file.path])
    try {
      for (const file of pending) {
        if (!this.browsingAllowed() || controller.signal.aborted) {
          return
        }
        try {
          // eslint-disable-next-line no-await-in-loop
          const summary = await readMcapVideoSummary(this.recordingStreamUrl(file), controller.signal)
          this.patch({ summaries: { ...this.state.summaries, [file.path]: summary } })
        } catch (error) {
          if (controller.signal.aborted || !this.browsingAllowed()) {
            return
          }
          console.warn(`Failed to read video summary for ${file.name}:`, error)
        }
      }
    } finally {
      if (this.summaryController === controller) {
        this.summaryController = null
      }
    }
    await this.loadThumbnails()
  }

  private async loadThumbnails(): Promise<void> {
    if (!this.browsingAllowed()) {
      return
    }
    const pending = this.readyFiles().filter((file) => {
      if (this.state.thumbnails[file.path]) {
        return false
      }
      const summary = this.state.summaries[file.path]
      return Boolean(summary?.tracks.some((track) => track.frameCount > 0))
    })
    for (const file of pending) {
      if (!this.browsingAllowed() || this.state.playerOpen) {
        return
      }
      // eslint-disable-next-line no-await-in-loop
      await this.loadThumbnail(file)
    }
  }

  private async loadThumbnail(file: LibraryRecording): Promise<void> {
    const cacheKey = {
      path: file.path,
      sizeBytes: file.size_bytes,
      modified: file.created,
    }
    const cached = await getCachedThumbnail(cacheKey)
    if (cached) {
      this.rememberThumbnail(file.path, cached)
      return
    }
    if (!this.browsingAllowed()) {
      return
    }
    this.thumbnailController?.abort()
    const controller = new AbortController()
    this.thumbnailController = controller
    try {
      const blob = await extractMcapThumbnail(this.recordingStreamUrl(file), { signal: controller.signal })
      if (!blob || controller.signal.aborted || !this.browsingAllowed()) {
        return
      }
      await setCachedThumbnail(cacheKey, blob)
      this.rememberThumbnail(file.path, blob)
    } catch (error) {
      if (controller.signal.aborted || !this.browsingAllowed()) {
        return
      }
      console.warn(`Failed to build a preview for ${file.name}:`, error)
    } finally {
      if (this.thumbnailController === controller) {
        this.thumbnailController = null
      }
    }
  }

  private rememberThumbnail(path: string, blob: Blob): void {
    const previous = this.state.thumbnails[path]
    if (previous) {
      URL.revokeObjectURL(previous)
    }
    this.patch({
      thumbnails: { ...this.state.thumbnails, [path]: URL.createObjectURL(blob) },
    })
  }

  private forgetThumbnail(path: string): void {
    const url = this.state.thumbnails[path]
    if (url) {
      URL.revokeObjectURL(url)
    }
    const thumbnails = { ...this.state.thumbnails }
    delete thumbnails[path]
    this.patch({ thumbnails })
  }

  private forgetSummary(path: string): void {
    const file = this.state.recordings.find((recording) => recording.path === path)
    if (file) {
      deleteCachedThumbnail({
        path: file.path,
        sizeBytes: file.size_bytes,
        modified: file.created,
      }).catch(() => undefined)
    }
    this.forgetThumbnail(path)
    const summaries = { ...this.state.summaries }
    delete summaries[path]
    this.patch({ summaries })
  }
}
