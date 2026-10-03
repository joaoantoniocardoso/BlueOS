import type { ThumbnailCacheKey } from '@/libs/mcap/adapters/thumbnail-cache'
import {
  getCachedThumbnail,
  setCachedThumbnail,
} from '@/libs/mcap/adapters/thumbnail-cache'
import { extractMcapThumbnail } from '@/libs/mcap/adapters/thumbnail'
import { abortError } from '@/libs/mcap/logic/abort'

/** Thumbnails pulled over the vehicle link at once; a long list waits its turn instead of flooding the link. */
const MAX_CONCURRENT_EXTRACTIONS = 2

export interface RecordingThumbnailRequest {
  downloadUrl: string
  cacheKey: ThumbnailCacheKey
  signal?: AbortSignal
}

let runningExtractions = 0
const waitingExtractions: (() => void)[] = []

/** Loads a JPEG preview, using the persistent cache when the file identity is unchanged. */
export async function loadRecordingThumbnail(
  request: RecordingThumbnailRequest,
): Promise<Blob | null> {
  const cached = await getCachedThumbnail(request.cacheKey)
  if (cached) {
    return cached
  }
  await extractionTurn(request.signal)
  try {
    const thumbnail = await extractMcapThumbnail(request.downloadUrl, { signal: request.signal })
    if (thumbnail) {
      await setCachedThumbnail(request.cacheKey, thumbnail)
    }
    return thumbnail
  } finally {
    const next = waitingExtractions.shift()
    if (next) {
      next()
    } else {
      runningExtractions -= 1
    }
  }
}

/** Waits until fewer than the maximum extractions run; a finishing one hands its turn straight to the next. */
function extractionTurn(signal?: AbortSignal): Promise<void> {
  if (runningExtractions < MAX_CONCURRENT_EXTRACTIONS) {
    runningExtractions += 1
    return Promise.resolve()
  }
  return new Promise((resolve, reject) => {
    function onAbort(): void {
      waitingExtractions.splice(waitingExtractions.indexOf(start), 1)
      reject(abortError())
    }
    function start(): void {
      signal?.removeEventListener('abort', onAbort)
      resolve()
    }
    waitingExtractions.push(start)
    signal?.addEventListener('abort', onAbort, { once: true })
  })
}
