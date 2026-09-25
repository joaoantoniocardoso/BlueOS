import { DEFAULT_RECORDING_HTTP_PREFIX } from './constants'

/* eslint-disable import/prefer-default-export */
export function recordingUrl(path: string, basePrefix = DEFAULT_RECORDING_HTTP_PREFIX): string {
  const segments = path.split('/').filter((segment) => segment.length > 0)
  const encoded = segments.map((segment) => encodeURIComponent(segment)).join('/')
  return `${basePrefix}/${encoded}`
}
