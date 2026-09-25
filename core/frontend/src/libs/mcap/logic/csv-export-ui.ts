import type { Mp4ExportRange } from '../adapters/export'
import type { McapRecordingChannel } from './channels'
import type { CsvExportProgress } from './csv'

export function csvSelectionLabel(selectedCount: number, totalCount: number): string {
  if (selectedCount === 0) {
    return 'No channels selected'
  }
  if (selectedCount === totalCount) {
    return `All ${totalCount} channels`
  }
  return `${selectedCount} of ${totalCount} channels`
}

export function csvRangeLabel(clip: Mp4ExportRange | null): string {
  if (!clip) {
    return 'Whole recording'
  }
  const start = Math.round(clip.startSeconds)
  const end = Number.isFinite(clip.endSeconds) ? `${Math.round(clip.endSeconds)}s` : 'end'
  return `${start}s – ${end}`
}

export function csvExportPercentage(progress: CsvExportProgress | null): number {
  const { seconds, durationSeconds } = progress ?? { seconds: 0, durationSeconds: 0 }
  if (durationSeconds <= 0) {
    return 0
  }
  return Math.min(100, Math.round(seconds / durationSeconds * 100))
}

export function csvExportStatusText(
  progress: CsvExportProgress | null,
  formatKilobytes: (kilobytes: number) => string,
): string {
  const size = formatKilobytes((progress?.bytes ?? 0) / 1024)
  const messages = progress?.messages ?? 0
  return `Saving CSV · ${messages.toLocaleString()} messages · ${size}`
}

export function csvFileName(name: string, clip: Mp4ExportRange | null): string {
  if (!clip) {
    return `${name}.csv`
  }
  const end = Number.isFinite(clip.endSeconds) ? `${Math.round(clip.endSeconds)}s` : 'end'
  return `${name}-${Math.round(clip.startSeconds)}s-${end}.csv`
}

export function filterChannelsBySearch(
  channels: McapRecordingChannel[],
  query: string,
): McapRecordingChannel[] {
  const normalized = query.trim().toLowerCase()
  if (!normalized) {
    return channels
  }
  return channels.filter((channel) => channel.topic.toLowerCase().includes(normalized)
    || channel.schemaName.toLowerCase().includes(normalized))
}
