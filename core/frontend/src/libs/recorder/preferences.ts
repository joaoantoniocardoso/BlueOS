import { RECORDING_SORT_OPTIONS, type RecordingSortKey } from './sort'

export const RECORDS_LAYOUT_STORAGE_KEY = 'blueos.records.layout'
export const RECORDS_SORT_STORAGE_KEY = 'blueos.records.sort'
export const RECORDS_BYTE_SOURCE_STORAGE_KEY = 'blueos.records.byteSource'

/** How the Records page lays out recordings. */
export type RecordsLayout = 'cards' | 'list'

/** Where the Records page reads recording bytes: HTTP ranges from nginx, or the recorder `bytes` Query. */
export type RecordsByteSource = 'http' | 'zenoh'

/** The order of the Records page, shared by the cards and the list. */
export interface RecordsSort {
  key: RecordingSortKey
  descending: boolean
}

/** Reaching `window.localStorage` itself throws when the browser blocks site data, so it is read on each use. */
type PreferenceStorage = () => Pick<Storage, 'getItem' | 'setItem'>

/** The browser's local storage, where the Records page keeps its layout and sort. */
export function browserStorage(): Pick<Storage, 'getItem' | 'setItem'> {
  return window.localStorage
}

/** The stored layout, or cards when none is stored or the browser refuses storage. */
export function storedLayout(storage: PreferenceStorage): RecordsLayout {
  return readItem(storage, RECORDS_LAYOUT_STORAGE_KEY) === 'list' ? 'list' : 'cards'
}

/** The stored sort, or newest first when none is stored, it no longer parses, or the browser refuses storage. */
export function storedSort(storage: PreferenceStorage): RecordsSort {
  try {
    const stored = JSON.parse(readItem(storage, RECORDS_SORT_STORAGE_KEY) ?? 'null')
    if (
      RECORDING_SORT_OPTIONS.some((option) => option.value === stored?.key)
      && typeof stored.descending === 'boolean'
    ) {
      return { key: stored.key, descending: stored.descending }
    }
  } catch {
    // A value from an older page that no longer parses: start from the default.
  }
  return { key: 'created', descending: true }
}

/** The stored byte source, or HTTP when none is stored or the browser refuses storage. */
export function storedByteSource(storage: PreferenceStorage): RecordsByteSource {
  return readItem(storage, RECORDS_BYTE_SOURCE_STORAGE_KEY) === 'zenoh' ? 'zenoh' : 'http'
}

export function storeLayout(storage: PreferenceStorage, layout: RecordsLayout): void {
  writeItem(storage, RECORDS_LAYOUT_STORAGE_KEY, layout)
}

export function storeSort(storage: PreferenceStorage, sort: RecordsSort): void {
  writeItem(storage, RECORDS_SORT_STORAGE_KEY, JSON.stringify(sort))
}

function readItem(storage: PreferenceStorage, key: string): string | null {
  try {
    return storage().getItem(key)
  } catch {
    return null
  }
}

function writeItem(storage: PreferenceStorage, key: string, value: string): void {
  try {
    storage().setItem(key, value)
  } catch {
    // Private mode or quota: the current session still keeps the choice.
  }
}
