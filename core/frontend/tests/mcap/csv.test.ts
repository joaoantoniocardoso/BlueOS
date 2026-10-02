import { describe, expect, it } from 'vitest'

import { exportChannelsAsCsv } from '@/libs/mcap/logic/csv'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'

import { buildJsonTelemetryMcap } from './build-mcap'
import MemoryByteSource from './memory-byte-source'

describe('exportChannelsAsCsv', () => {
  it('exports selected channels as CSV rows', async () => {
    const bytes = await buildJsonTelemetryMcap(3)
    const reader = await McapIndexedReader.open(new MemoryByteSource(bytes))
    const channel = [...reader.summary.channels.values()][0]
    const { startTime } = reader.summary
    const durationSeconds = Number(reader.summary.endTime - startTime) / 1e9
    const blob = await exportChannelsAsCsv(
      { reader, startTime, durationSeconds },
      [channel.id],
    )
    const text = await blob.text()
    expect(text.split('\n')[0]).toContain('log_time')
    expect(text).toContain('/telemetry/depth')
    expect(text).toContain('depth_m')
    expect(text).toContain('sample-0')
  })
})
