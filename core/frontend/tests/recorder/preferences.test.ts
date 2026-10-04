import { describe, expect, it } from 'vitest'

import {
  RECORDS_BYTE_SOURCE_STORAGE_KEY,
  RECORDS_LAYOUT_STORAGE_KEY,
  RECORDS_SORT_STORAGE_KEY,
  storedByteSource,
  storedLayout,
  storedSort,
  storeLayout,
  storeSort,
} from '@/libs/recorder/preferences'

function memoryStorage(items: Record<string, string> = {}): () => Pick<Storage, 'getItem' | 'setItem'> {
  return () => ({
    getItem: (key) => items[key] ?? null,
    setItem: (key, value) => { items[key] = value },
  })
}

function blockedStorage(): Pick<Storage, 'getItem' | 'setItem'> {
  throw new Error('SecurityError')
}

describe('Records page preferences', () => {
  it('reads back the layout and sort it stored', () => {
    const storage = memoryStorage()
    storeLayout(storage, 'list')
    storeSort(storage, { key: 'size_bytes', descending: false })
    expect(storedLayout(storage)).toBe('list')
    expect(storedSort(storage)).toEqual({ key: 'size_bytes', descending: false })
  })

  it('falls back to cards, newest first, when nothing valid is stored', () => {
    expect(storedLayout(memoryStorage())).toBe('cards')
    expect(storedSort(memoryStorage())).toEqual({ key: 'created', descending: true })
    const garbage = memoryStorage({
      [RECORDS_LAYOUT_STORAGE_KEY]: 'table',
      [RECORDS_SORT_STORAGE_KEY]: '{"key":"color","descending":"yes"',
    })
    expect(storedLayout(garbage)).toBe('cards')
    expect(storedSort(garbage)).toEqual({ key: 'created', descending: true })
    const unknownKey = memoryStorage({ [RECORDS_SORT_STORAGE_KEY]: '{"key":"color","descending":false}' })
    expect(storedSort(unknownKey)).toEqual({ key: 'created', descending: true })
  })

  it('keeps working when the browser refuses storage', () => {
    expect(storedLayout(blockedStorage)).toBe('cards')
    expect(storedSort(blockedStorage)).toEqual({ key: 'created', descending: true })
    expect(() => storeLayout(blockedStorage, 'list')).not.toThrow()
    expect(() => storeSort(blockedStorage, { key: 'name', descending: false })).not.toThrow()
  })

  it('reads recording bytes over HTTP unless the byte source is set to zenoh', () => {
    expect(storedByteSource(memoryStorage())).toBe('http')
    expect(storedByteSource(blockedStorage)).toBe('http')
    expect(storedByteSource(memoryStorage({ [RECORDS_BYTE_SOURCE_STORAGE_KEY]: 'websocket' }))).toBe('http')
    expect(storedByteSource(memoryStorage({ [RECORDS_BYTE_SOURCE_STORAGE_KEY]: 'zenoh' }))).toBe('zenoh')
  })
})
