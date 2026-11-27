export type FinalizeResult =
  | { phase: 'success', statusText: string }
  | { phase: 'error', message: string }

export default function resolveFinalizeResult(thrownError?: unknown, trackerError?: string | null): FinalizeResult {
  if (thrownError !== undefined) {
    return { phase: 'error', message: String(thrownError) }
  }
  if (trackerError) {
    return { phase: 'error', message: trackerError }
  }
  return { phase: 'success', statusText: 'Extension installed successfully' }
}
