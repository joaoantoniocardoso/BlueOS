import { describe, expect, it } from 'vitest'

import { exportChannelsAsCsv, type McapCsvRecording } from '@/libs/mcap/logic/csv'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'

import { buildJsonTelemetryMcap, buildUint64ArrayMcap } from './build-mcap'
import MemoryByteSource from './memory-byte-source'

async function openRecording(bytes: Uint8Array): Promise<McapCsvRecording> {
  const reader = await McapIndexedReader.open(new MemoryByteSource(bytes))
  const { startTime } = reader.summary
  return { reader, startTime, durationSeconds: Number(reader.summary.endTime - startTime) / 1e9 }
}

function allChannelIds(reader: McapIndexedReader): number[] {
  return [...reader.summary.channels.keys()]
}

describe('exportChannelsAsCsv', () => {
  it('exports selected channels as CSV rows', async () => {
    const recording = await openRecording(await buildJsonTelemetryMcap(3))
    const blob = await exportChannelsAsCsv(recording, allChannelIds(recording.reader))
    const text = await blob.text()
    expect(text.split('\n')[0]).toContain('log_time')
    expect(text).toContain('/telemetry/depth')
    expect(text).toContain('depth_m')
    expect(text).toContain('sample-0')
  })

  it('exports the values of a 64-bit integer array as decimal numbers', async () => {
    const recording = await openRecording(await buildUint64ArrayMcap())

    const text = await (await exportChannelsAsCsv(recording, allChannelIds(recording.reader))).text()

    expect(text.trimEnd().split('\n')).toHaveLength(4)
    expect(text).toContain('""18446744073709551615""')
  })
})
