import { ExtensionData, SourceCandidate, SourceConflictDetail } from '@/types/kraken'

type EntryWithConflicts = Pick<ExtensionData, 'conflicts'> & Partial<ExtensionData>

export function needsSourceChoice(entry: Pick<ExtensionData, 'conflicts'> | undefined): boolean {
  return (entry?.conflicts?.length ?? 0) > 0
}

export function entryCandidates(entry: EntryWithConflicts): SourceCandidate[] {
  if (!needsSourceChoice(entry)) {
    return []
  }
  const preferred: SourceCandidate = {
    manifest: entry.manifest,
    manifest_name: entry.manifest_name,
    docker: entry.docker ?? '',
    name: entry.name ?? '',
    tags: Object.keys(entry.versions ?? {}),
  }
  return [preferred, ...(entry.conflicts ?? [])]
}

export function parseSourceConflict(error: any): SourceConflictDetail | undefined {
  const detail = error?.response?.data?.detail
  if (error?.response?.status !== 409 || detail?.code !== 'extension_source_conflict') {
    return undefined
  }
  return detail as SourceConflictDetail
}

export function isInstalledCandidate(candidate: SourceCandidate, installedDocker: string | undefined): boolean {
  return installedDocker !== undefined && candidate.docker === installedDocker
}

export function isSourceSwitch(candidate: SourceCandidate, installedDocker: string | undefined): boolean {
  return installedDocker !== undefined && candidate.docker !== installedDocker
}

export function formatCandidate(candidate: SourceCandidate): { repo: string, source: string, tags: string } {
  return {
    repo: candidate.docker,
    source: candidate.manifest_name || candidate.name,
    tags: candidate.tags.length > 0 ? candidate.tags.join(', ') : '-',
  }
}
