/**
 * Parses the sort time for a recording from its file name (UTC), falling back to file mtime.
 * Shared with `blueos-recorder-library` via `tests/vectors/recording_file_names.json`.
 */
// eslint-disable-next-line import/prefer-default-export
export function createdUnixSecondsFromFilename(
  fileName: string,
  fileTimeUnixSeconds: number,
): number {
  if (!isAscii(fileName)) {
    return fileTimeUnixSeconds
  }
  const copyOrSnapshot = copyOrSnapshotTimestamp(fileName)
  if (copyOrSnapshot !== undefined) {
    return copyOrSnapshot
  }
  const legacySplit = legacySplitTimestamp(fileName)
  if (legacySplit !== undefined) {
    return legacySplit
  }
  const recorderPrefix = recorderPrefixTimestamp(fileName)
  if (recorderPrefix !== undefined) {
    return recorderPrefix
  }
  return fileTimeUnixSeconds
}

function copyOrSnapshotTimestamp(name: string): number | undefined {
  const lower = name.toLowerCase()
  const copyMarker = lower.lastIndexOf('.copy-')
  const snapshotMarker = lower.lastIndexOf('.snapshot-')
  const marker = copyMarker >= 0 ? copyMarker : snapshotMarker
  if (marker < 0) {
    return undefined
  }
  return parseIsoSuffixTimestamp(lower.slice(marker + 1))
}

function legacySplitTimestamp(name: string): number | undefined {
  const lower = name.toLowerCase()
  const marker = lower.lastIndexOf('_split_')
  if (marker < 0) {
    return undefined
  }
  return parseLegacySplitSuffix(lower.slice(marker + '_split_'.length))
}

function recorderPrefixTimestamp(name: string): number | undefined {
  const lower = name.toLowerCase()
  if (!lower.startsWith('recorder_')) {
    return undefined
  }
  return parseRecorderPrefix(lower.slice('recorder_'.length))
}

function isAscii(fileName: string): boolean {
  for (const character of fileName) {
    if (character.charCodeAt(0) > 0x7f) {
      return false
    }
  }
  return true
}

function parseIsoSuffixTimestamp(suffix: string): number | undefined {
  let marker: string | undefined
  if (suffix.startsWith('copy-')) {
    marker = suffix.slice('copy-'.length)
  } else if (suffix.startsWith('snapshot-')) {
    marker = suffix.slice('snapshot-'.length)
  }
  if (marker === undefined || !marker.endsWith('z.mcap')) {
    return undefined
  }
  const body = marker.slice(0, -'z.mcap'.length)
  return parseIsoTimestampBody(body)
}

function parseLegacySplitSuffix(suffix: string): number | undefined {
  if (!suffix.endsWith('.mcap')) {
    return undefined
  }
  const body = suffix.slice(0, -'.mcap'.length)
  if (body.length !== 15 || body.charAt(8) !== '_') {
    return undefined
  }
  return parseUtcTimestamp(body.slice(0, 8), body.slice(9))
}

function parseRecorderPrefix(rest: string): number | undefined {
  if (rest.length < 15 || rest.charAt(8) !== '_') {
    return undefined
  }
  return parseUtcTimestamp(rest.slice(0, 8), rest.slice(9, 15))
}

function parseIsoTimestampBody(body: string): number | undefined {
  if (
    body.length !== 19
    || body.charAt(10) !== 'T' && body.charAt(10) !== 't'
  ) {
    return undefined
  }
  const date = `${body.slice(0, 4)}${body.slice(5, 7)}${body.slice(8, 10)}`
  const time = `${body.slice(11, 13)}${body.slice(14, 16)}${body.slice(17, 19)}`
  return parseUtcTimestamp(date, time)
}

function parseUtcTimestamp(date: string, time: string): number | undefined {
  if (date.length !== 8 || time.length !== 6) {
    return undefined
  }
  const year = parseDigits(date, 0, 4)
  const month = parseDigits(date, 4, 6)
  const day = parseDigits(date, 6, 8)
  const hour = parseDigits(time, 0, 2)
  const minute = parseDigits(time, 2, 4)
  const second = parseDigits(time, 4, 6)
  if (
    year === undefined
    || month === undefined
    || day === undefined
    || hour === undefined
    || minute === undefined
    || second === undefined
  ) {
    return undefined
  }
  if (
    month < 1
    || month > 12
    || day < 1
    || day > 31
    || hour > 23
    || minute > 59
    || second > 59
  ) {
    return undefined
  }
  return Date.UTC(year, month - 1, day, hour, minute, second) / 1000
}

function parseDigits(value: string, start: number, end: number): number | undefined {
  const slice = value.slice(start, end)
  const parsed = Number.parseInt(slice, 10)
  return Number.isNaN(parsed) ? undefined : parsed
}
