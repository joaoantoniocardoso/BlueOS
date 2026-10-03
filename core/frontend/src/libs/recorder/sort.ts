import type { LibraryRecording } from './types'

/** What the Records page sorts recordings by. */
export type RecordingSortKey =
  | 'created'
  | 'name'
  | 'size_bytes'
  | 'state'

/** The sort keys in the order the "Sort by" control lists them. */
export const RECORDING_SORT_OPTIONS: { text: string, value: RecordingSortKey }[] = [
  { text: 'Date', value: 'created' },
  { text: 'Name', value: 'name' },
  { text: 'Size', value: 'size_bytes' },
  { text: 'State', value: 'state' },
]

/** The value each key compares; `null` is unknown and sorts last in both directions. */
const SORT_VALUE: Record<RecordingSortKey, (file: LibraryRecording) => number | string | null> = {
  created: (file) => file.created,
  name: (file) => file.name,
  size_bytes: (file) => file.size_bytes,
  state: (file) => file.state,
}

/** A sorted copy of `files`; equal values keep a stable order by path, which also follows the direction. */
export function sortRecordings(
  files: LibraryRecording[],
  key: RecordingSortKey,
  descending: boolean,
): LibraryRecording[] {
  const direction = descending ? -1 : 1
  return [...files].sort((left, right) => {
    const leftValue = SORT_VALUE[key](left)
    const rightValue = SORT_VALUE[key](right)
    if (leftValue === null || rightValue === null) {
      if (leftValue !== rightValue) {
        return leftValue === null ? 1 : -1
      }
    } else if (leftValue !== rightValue) {
      return direction * (leftValue < rightValue ? -1 : 1)
    }
    return direction * left.path.localeCompare(right.path)
  })
}
