const LOG_LEVEL_LABELS: Record<number, string> = {
  10: 'trace',
  20: 'debug',
  30: 'info',
  40: 'warn',
  50: 'error',
  60: 'fatal',
}

export function formatRecordingBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) {
    return '--'
  }
  if (bytes < 1024) {
    return `${bytes} B`
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KiB`
  }
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(2)} MiB`
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GiB`
}

export function logLevelLabel(level: number): string {
  return LOG_LEVEL_LABELS[level] ?? String(level)
}
