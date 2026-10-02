import type {
  LibraryRecording,
  RecordingOperationEvent,
} from './types'

export function sortRecordingsNewestFirst(files: LibraryRecording[]): LibraryRecording[] {
  return [...files].sort((left, right) => {
    if (right.created !== left.created) {
      return right.created - left.created
    }
    return right.path.localeCompare(left.path)
  })
}

export function isSnapshotOperationForPath(
  event: RecordingOperationEvent,
  path: string,
): boolean {
  return event.operation === 'snapshot' && event.path === path
}

export function snapshotDownloadPath(event: RecordingOperationEvent): string | null {
  if (!event.succeeded || event.cancelled || !event.output_path) {
    return null
  }
  return event.output_path
}

function snapshotPathPrefix(sourcePath: string): string {
  const segments = sourcePath.split('/')
  const fileName = segments.pop() ?? sourcePath
  const parent = segments.join('/')
  const stem = fileName.replace(/\.mcap$/i, '')
  return parent ? `${parent}/${stem}.snapshot-` : `${stem}.snapshot-`
}

/** Ready snapshot MCAP paths for one source recording, newest first. */
export function snapshotPathsForSource(
  sourcePath: string,
  files: LibraryRecording[],
): string[] {
  const prefix = snapshotPathPrefix(sourcePath)
  return files
    .filter(
      (file) => file.state === 'ready'
        && file.path.startsWith(prefix)
        && file.path.toLowerCase().endsWith('.mcap'),
    )
    .sort((left, right) => right.created - left.created)
    .map((file) => file.path)
}

export function readySnapshotDownloadPath(
  sourcePath: string,
  files: LibraryRecording[],
  excludePaths: ReadonlySet<string> = new Set(),
): string | null {
  const prefix = snapshotPathPrefix(sourcePath)
  const match = files
    .filter(
      (file) => file.state === 'ready'
        && file.path.startsWith(prefix)
        && file.path.toLowerCase().endsWith('.mcap')
        && !excludePaths.has(file.path),
    )
    .sort((left, right) => right.created - left.created)[0]
  return match?.path ?? null
}

export function operationFailureMessage(
  event: RecordingOperationEvent,
  fileName: string,
): string | null {
  if (event.succeeded || event.cancelled) {
    return null
  }
  let verb = 'Delete'
  if (event.operation === 'repair') {
    verb = 'Repair'
  } else if (event.operation === 'snapshot') {
    verb = 'Snapshot'
  }
  return `${verb} failed for ${fileName}: ${event.error || 'unknown error'}`
}
