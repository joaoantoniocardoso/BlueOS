import type { RecordingIndex } from '@blueos-idl/messages'

import { query } from '@/libs/blueos-api'
import type { RecordingIndexPage, RecordingIndexSource } from '@/libs/mcap'

import {
  INDEX_PAGE_LIMIT,
  RECORDER_SERVICE,
  RECORDING_INDEX_REQUEST_SCHEMA,
  RECORDING_INDEX_SCHEMA,
} from './constants'

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

/* eslint-disable import/prefer-default-export */
export function createRecordingIndexSource(path: string): RecordingIndexSource {
  return {
    page(fromOffset: number, limit: number, signal?: AbortSignal): Promise<RecordingIndexPage> {
      if (signal?.aborted) {
        return Promise.reject(new DOMException('Aborted', 'AbortError'))
      }
      const boundedLimit = Math.min(Math.max(1, limit), INDEX_PAGE_LIMIT)
      return query(
        RECORDER_SERVICE,
        'index',
        RECORDING_INDEX_SCHEMA,
        RECORDING_INDEX_REQUEST_SCHEMA,
        {
          path,
          from_offset: fromOffset,
          limit: boundedLimit,
        },
      ).then((response) => {
        if (signal?.aborted) {
          return Promise.reject(new DOMException('Aborted', 'AbortError'))
        }
        return mapIndexPage(response)
      })
    },
  }
}
