import { describe, expect, it } from 'vitest'

import {
  mapRecordingFile,
  mapRecordingOperation,
  mapRecordingState,
  STATE_READY,
  STATE_RECORDING,
} from '@/libs/recorder'

describe('recorder map', () => {
  it('maps recording state codes to string unions', () => {
    expect(mapRecordingState(STATE_RECORDING)).toBe('recording')
    expect(mapRecordingState(STATE_READY)).toBe('ready')
  })

  it('maps IDL recording files to library recordings', () => {
    const mapped = mapRecordingFile({
      path: '2024/foo.mcap',
      name: 'foo.mcap',
      size_bytes: 1024,
      created: { sec: 1_700_000_000, nanosec: 500_000_000 },
      state: STATE_READY,
      repair_bytes_processed: 0,
      repair_total_bytes: 0,
      repair_bytes_per_second: 0,
      repair_error: '',
    })
    expect(mapped.created).toBe(1_700_000_000.5)
    expect(mapped.state).toBe('ready')
  })

  it('maps operation events', () => {
    const mapped = mapRecordingOperation({
      operation: 1,
      path: 'a.mcap',
      output_path: 'a.snapshot.mcap',
      succeeded: true,
      cancelled: false,
      error: '',
    })
    expect(mapped.operation).toBe('snapshot')
    expect(mapped.output_path).toBe('a.snapshot.mcap')
  })
})
