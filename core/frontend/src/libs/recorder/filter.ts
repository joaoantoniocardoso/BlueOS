import type { LibraryRecording, RecordingState } from './types'

/** `null` means "do not filter on this". `date` is a UTC calendar day, `YYYY-MM-DD`. */
export interface RecordingFilters {
  search: string
  state: RecordingState | null
  date: string | null
}

export const NO_RECORDING_FILTERS: RecordingFilters = { search: '', state: null, date: null }

export function utcCalendarDay(timestampSeconds: number): string {
  return new Date(timestampSeconds * 1000).toISOString().slice(0, 10)
}

/** The recordings matching every set filter, in their original order. */
export function filterRecordings(
  recordings: LibraryRecording[],
  filters: RecordingFilters,
): LibraryRecording[] {
  const search = filters.search.trim().toLowerCase()
  return recordings.filter((file) => (
    (!search || file.name.toLowerCase().includes(search))
    && (filters.state === null || file.state === filters.state)
    && (filters.date === null || utcCalendarDay(file.created) === filters.date)
  ))
}

/** One option per UTC day that has a recording, newest first. */
export function dateFilterOptions(recordings: LibraryRecording[]): { text: string, value: string }[] {
  const days = new Set(recordings.map((file) => utcCalendarDay(file.created)))
  return Array.from(days)
    .sort((left, right) => right.localeCompare(left))
    .map((day) => ({ text: `${day} UTC`, value: day }))
}
