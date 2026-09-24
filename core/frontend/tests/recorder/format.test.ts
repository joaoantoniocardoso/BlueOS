import { describe, expect, it } from 'vitest'

import { formatRecordingBytes, logLevelLabel } from '@/components/recorder/format'

describe('formatRecordingBytes', () => {
  it('formats small and large values', () => {
    expect(formatRecordingBytes(512)).toBe('512 B')
    expect(formatRecordingBytes(2048)).toBe('2.0 KiB')
    expect(formatRecordingBytes(5 * 1024 * 1024)).toBe('5.00 MiB')
  })

  it('handles invalid input', () => {
    expect(formatRecordingBytes(Number.NaN)).toBe('--')
  })
})

describe('logLevelLabel', () => {
  it('maps known levels', () => {
    expect(logLevelLabel(30)).toBe('info')
    expect(logLevelLabel(50)).toBe('error')
  })

  it('falls back to numeric string', () => {
    expect(logLevelLabel(99)).toBe('99')
  })
})
