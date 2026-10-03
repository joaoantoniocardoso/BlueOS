import type { LibraryRecording } from './types'

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
