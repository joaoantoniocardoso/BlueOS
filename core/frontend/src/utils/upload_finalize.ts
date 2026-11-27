export type FinalizeResult =
  | { phase: 'success', statusText: string }
  | { phase: 'error', message: string }

export default function resolveFinalizeResult(thrownError?: unknown): FinalizeResult {
  if (thrownError !== undefined) {
    return { phase: 'error', message: String(thrownError) }
  }
  return { phase: 'success', statusText: 'Extension installed successfully' }
}
