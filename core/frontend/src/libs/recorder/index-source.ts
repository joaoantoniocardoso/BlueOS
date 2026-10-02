import type { RecordingIndex } from '@blueos-idl/messages'

import { query } from '@/libs/blueos-api/command'
import { index } from '@/libs/blueos-api/services/recorder'
import type { Transport } from '@/libs/blueos-api/transport'
import type { RecordingIndexPage, RecordingIndexSource } from '@/libs/mcap/logic/recording-index'

/** Maximum chunks per vehicle index request; matches the recorder service default. */
export const INDEX_PAGE_LIMIT = 20_000

function recordsToUint8Array(records: number[] | Uint8Array): Uint8Array {
  if (records instanceof Uint8Array) {
    return records
  }
  return Uint8Array.from(records)
}

function mapIndexPage(page: RecordingIndex): RecordingIndexPage {
  return {
    size: page.size,
    offset: page.offset,
    closed: page.closed,
    chunks: page.chunks,
    message_counts: page.message_counts,
    records: recordsToUint8Array(page.records),
  }
}

export async function queryRecordingIndexPage(
  transport: Transport,
  path: string,
  fromOffset: number,
  limit: number,
  signal?: AbortSignal,
): Promise<RecordingIndexPage> {
  if (signal?.aborted) {
    return Promise.reject(new DOMException('Aborted', 'AbortError'))
  }
  const boundedLimit = Math.min(Math.max(1, limit), INDEX_PAGE_LIMIT)
  const response = await query(transport, index, {
    path,
    from_offset: fromOffset,
    limit: boundedLimit,
  })
  if (signal?.aborted) {
    return Promise.reject(new DOMException('Aborted', 'AbortError'))
  }
  return mapIndexPage(response)
}

export function createRecordingIndexSource(
  transport: Transport,
  path: string,
): RecordingIndexSource {
  return {
    page(fromOffset, limit, signal) {
      return queryRecordingIndexPage(transport, path, fromOffset, limit, signal)
    },
  }
}

interface CachedPage {
  sizeBytes: number
  page: RecordingIndexPage
}

export function createCachedRecordingIndexSource(
  transport: Transport,
  path: string,
  sizeBytesForPath: () => number,
): RecordingIndexSource {
  const cache = new Map<string, CachedPage>()

  return {
    async page(fromOffset, limit, signal) {
      const sizeBytes = sizeBytesForPath()
      const cacheKey = `${fromOffset}:${limit}`
      const hit = cache.get(cacheKey)
      if (hit && hit.sizeBytes === sizeBytes) {
        return hit.page
      }
      const page = await queryRecordingIndexPage(transport, path, fromOffset, limit, signal)
      cache.set(cacheKey, { sizeBytes, page })
      return page
    },
  }
}
