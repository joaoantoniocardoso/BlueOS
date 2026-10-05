import { prettifySize } from '@/utils/helper_functions'

import type { LibraryRecording } from './types'
import { formatDuration } from './view-logic'

/** What a selection holds, for the selection bar. */
export interface SelectionSummary {
  count: number
  sizeBytes: number
  /** The sum of the known durations. */
  durationSeconds: number
  unknownDurationCount: number
  needsRepairCount: number
}

/** Adds or removes one path without duplicates. */
export function togglePathSelection(selectedPaths: readonly string[], path: string): string[] {
  if (selectedPaths.includes(path)) {
    return selectedPaths.filter((entry) => entry !== path)
  }
  return [...selectedPaths, path]
}

/**
 * Selects exactly `chosen` among the visible paths. Selections hidden by filters stay selected so a filter change
 * does not drop them.
 */
export function setVisibleSelection(
  selectedPaths: readonly string[],
  visible: readonly LibraryRecording[],
  chosen: readonly LibraryRecording[],
): string[] {
  const visiblePaths = new Set(visible.map((file) => file.path))
  const hidden = selectedPaths.filter((path) => !visiblePaths.has(path))
  return [...hidden, ...chosen.map((file) => file.path)]
}

export function allVisibleSelected(
  selectedPaths: readonly string[],
  visible: readonly LibraryRecording[],
): boolean {
  return visible.length > 0 && visible.every((file) => selectedPaths.includes(file.path))
}

export function someVisibleSelected(
  selectedPaths: readonly string[],
  visible: readonly LibraryRecording[],
): boolean {
  return visible.some((file) => selectedPaths.includes(file.path))
}

/** Selected recordings that are visible, in the same order as `visible`. */
export function selectedVisibleRecordings(
  selectedPaths: readonly string[],
  visible: readonly LibraryRecording[],
): LibraryRecording[] {
  const selected = new Set(selectedPaths)
  return visible.filter((file) => selected.has(file.path))
}

/** Drops paths that are no longer in the library. */
export function pruneSelection(selectedPaths: readonly string[], knownPaths: readonly string[]): string[] {
  const known = new Set(knownPaths)
  return selectedPaths.filter((path) => known.has(path))
}

/** Sums `files`; the file being written counts as long as `withLiveDuration` made it. */
export function selectionSummary(files: readonly LibraryRecording[]): SelectionSummary {
  const durations = files.map((file) => file.duration_seconds)
  return {
    count: files.length,
    sizeBytes: files.reduce((total, file) => total + file.size_bytes, 0),
    durationSeconds: durations.reduce<number>((total, duration) => total + (duration ?? 0), 0),
    unknownDurationCount: durations.filter((duration) => duration === null).length,
    needsRepairCount: files.filter((file) => file.state === 'needs_repair').length,
  }
}

/** For example "27 selected \u00B7 1.4 GB \u00B7 31h 12m 00s \u00B7 5 need repair"; a part that is zero is left out. */
export function selectionSummaryLabel(summary: SelectionSummary): string {
  const parts = [`${summary.count} selected`]
  if (summary.sizeBytes > 0) {
    parts.push(prettifySize(summary.sizeBytes / 1024))
  }
  const duration = [
    summary.durationSeconds > 0 ? formatDuration(summary.durationSeconds) : '',
    summary.unknownDurationCount > 0 ? `(${summary.unknownDurationCount} without a known duration)` : '',
  ].filter(Boolean).join(' ')
  if (duration) {
    parts.push(duration)
  }
  if (summary.needsRepairCount > 0) {
    parts.push(`${summary.needsRepairCount} need repair`)
  }
  return parts.join(' \u00B7 ')
}
