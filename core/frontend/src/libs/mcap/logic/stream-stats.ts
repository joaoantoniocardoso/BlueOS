import type { McapVideoStats } from '../adapters/player'
import type { VideoTrack } from './video-track'

export interface StreamStatRow {
  label: string
  value: string
  /** Vuetify text colour class, used to point out the counters that indicate trouble. */
  tone?: string
}

function share(value: number, total: number): string {
  return value > 0 && total > 0 ? ` (${(value / total * 100).toFixed(1)}%)` : ''
}

export function prettifyBitrate(bitsPerSecond: number): string {
  if (bitsPerSecond >= 1e6) {
    return `${(bitsPerSecond / 1e6).toFixed(1)} Mbps`
  }
  return `${Math.round(bitsPerSecond / 1e3)} kbps`
}

/** Short codec family for the metadata row, e.g. avc1.640033 → H.264. */
export function codecFamily(codec: string): string {
  if (codec.startsWith('avc1') || codec.startsWith('avc3')) {
    return 'H.264'
  }
  if (codec.startsWith('hvc1') || codec.startsWith('hev1')) {
    return 'H.265'
  }
  return codec
}

export function streamResolution(stats: Partial<McapVideoStats>): string {
  const { width, height } = stats
  return width && height ? `${width}x${height}` : ''
}

export function streamDetailStatRows(
  stats: Partial<McapVideoStats>,
  track: VideoTrack,
): StreamStatRow[] {
  const {
    framesRead = 0, keyframes = 0, framesLost = 0, framesSkipped = 0, framesCorrupt = 0,
    framesDecoded = 0, framesDropped = 0, decodeErrors = 0, frameRate = 0, bitrate = 0,
    bufferedAheadSeconds = 0, codec = '', width, height,
  } = stats
  const resolution = width && height ? `${width}x${height}` : ''
  return [
    { label: 'stream', value: [resolution, codec].filter((part) => part).join(' ') || '-' },
    { label: 'frames', value: `${framesRead} of ${track.frameCount} read` },
    { label: 'keyframes', value: `${keyframes}` },
    {
      label: 'lost',
      value: `${framesLost}${share(framesLost, framesRead + framesLost)}`,
      tone: framesLost > 0 ? 'warning--text' : undefined,
    },
    { label: 'skipped', value: `${framesSkipped}` },
    {
      label: 'corrupt',
      value: `${framesCorrupt}`,
      tone: framesCorrupt > 0 ? 'error--text' : undefined,
    },
    { label: 'decoded', value: `${framesDecoded}` },
    {
      label: 'dropped',
      value: `${framesDropped}${share(framesDropped, framesDecoded)}`,
      tone: framesDropped > 0 ? 'warning--text' : undefined,
    },
    {
      label: 'decode errors',
      value: `${decodeErrors}`,
      tone: decodeErrors > 0 ? 'error--text' : undefined,
    },
    { label: 'rate', value: `${frameRate.toFixed(1)} fps · ${prettifyBitrate(bitrate)}` },
    { label: 'buffered', value: `${bufferedAheadSeconds.toFixed(1)} s` },
  ]
}
