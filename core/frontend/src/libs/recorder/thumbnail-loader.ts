import { extractMcapThumbnail } from '@/libs/mcap/adapters/thumbnail'
import type { ThumbnailCacheKey } from '@/libs/mcap/adapters/thumbnail-cache'
import {
  getCachedThumbnail,
  setCachedThumbnail,
} from '@/libs/mcap/adapters/thumbnail-cache'

export interface RecordingThumbnailRequest {
  downloadUrl: string
  cacheKey: ThumbnailCacheKey
  signal?: AbortSignal
}

/** Loads a JPEG preview, using the persistent cache when the file identity is unchanged. */
export async function loadRecordingThumbnail(
  request: RecordingThumbnailRequest,
): Promise<Blob | null> {
  const cached = await getCachedThumbnail(request.cacheKey)
  if (cached) {
    return cached
  }
  const thumbnail = await extractMcapThumbnail(request.downloadUrl, { signal: request.signal })
  if (thumbnail) {
    await setCachedThumbnail(request.cacheKey, thumbnail)
  }
  return thumbnail
}
