import type { LibraryRecording, RecordingState } from './types'

/** `null` means "do not filter on this". `date` is a calendar day in the browser's time zone, `YYYY-MM-DD`. */
export interface RecordingFilters {
  search: string
  state: RecordingState | null
  date: string | null
}

export const NO_RECORDING_FILTERS: RecordingFilters = { search: '', state: null, date: null }

/** The `YYYY-MM-DD` day of a timestamp in the browser's time zone, the one the cards and the table display. */
export function localCalendarDay(timestampSeconds: number): string {
  const date = new Date(timestampSeconds * 1000)
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${date.getFullYear()}-${month}-${day}`
}

/** The recordings matching every set filter, in their original order. */
export function filterRecordings(
  recordings: LibraryRecording[],
  filters: RecordingFilters,
): LibraryRecording[] {
  const search = filters.search.trim().toLowerCase()
  return recordings.filter((file) => (!search || file.name.toLowerCase().includes(search))
    && (filters.state === null || file.state === filters.state)
    && (filters.date === null || localCalendarDay(file.created) === filters.date))
}

/** One option per local day that has a recording, newest first. */
export function dateFilterOptions(recordings: LibraryRecording[]): { text: string, value: string }[] {
  const days = new Set(recordings.map((file) => localCalendarDay(file.created)))
  return Array.from(days)
    .sort((left, right) => right.localeCompare(left))
    .map((day) => ({ text: day, value: day }))
}
