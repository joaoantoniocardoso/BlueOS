import {
  afterEach, describe, expect, it, vi,
} from 'vitest'

import { type CsvExportProgress, exportChannelsAsCsv, type McapCsvRecording } from '@/libs/mcap/logic/csv'
import { csvExportPercentage } from '@/libs/mcap/logic/csv-export-ui'
import { McapIndexedReader } from '@/libs/mcap/logic/reader'

import { buildJsonTelemetryMcap, buildTwoJsonTopicsMcap, buildUint64ArrayMcap } from './build-mcap'
import MemoryByteSource from './memory-byte-source'

async function openRecording(bytes: Uint8Array): Promise<McapCsvRecording> {
  const reader = await McapIndexedReader.open(new MemoryByteSource(bytes))
  const { startTime } = reader.summary
  return { reader, startTime, durationSeconds: Number(reader.summary.endTime - startTime) / 1e9 }
}

function allChannelIds(reader: McapIndexedReader): number[] {
  return [...reader.summary.channels.keys()]
}

/** Each data row keyed by the header, with the cells a short row does not have read as empty, like pandas does. */
function parseCsv(text: string): { header: string[], rows: Record<string, string>[] } {
  const [headerLine, ...lines] = text.trimEnd().split('\n')
  const header = headerLine.split(',')
  const rows = lines.map((line) => {
    const cells = line.split(',')
    return Object.fromEntries(header.map((column, index) => [column, cells[index] ?? '']))
  })
  return { header, rows }
}

describe('exportChannelsAsCsv', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('exports selected channels as CSV rows', async () => {
    const recording = await openRecording(await buildJsonTelemetryMcap(3))
    const blob = await exportChannelsAsCsv(recording, allChannelIds(recording.reader))
    const text = await blob.text()
    expect(text.split('\n')[0]).toContain('log_time')
    expect(text).toContain('/telemetry/depth')
    expect(text).toContain('depth_m')
    expect(text).toContain('sample-0')
  })

  it('writes every row with the union of the columns in the order they first appear, empty before that', async () => {
    const recording = await openRecording(await buildTwoJsonTopicsMcap())

    const blob = await exportChannelsAsCsv(recording, allChannelIds(recording.reader))
    const { header, rows } = parseCsv(await blob.text())

    expect(header).toEqual(['log_time', 'topic', 'sequence', 'depth_m', 'yaw', 'pitch'])
    expect(rows.map((row) => [row.topic, row.depth_m, row.yaw, row.pitch])).toEqual([
      ['/depth', '1', '', ''],
      ['/attitude', '', '90', ''],
      ['/depth', '2', '', ''],
      ['/attitude', '', '91', '5'],
      ['/depth', '3', '', ''],
    ])
  })

  it('exports the values of a 64-bit integer array as decimal numbers', async () => {
    const recording = await openRecording(await buildUint64ArrayMcap())

    const text = await (await exportChannelsAsCsv(recording, allChannelIds(recording.reader))).text()

    expect(text.trimEnd().split('\n')).toHaveLength(4)
    expect(text).toContain('""18446744073709551615""')
  })

  it('never reports a finished export before the file is returned', async () => {
    const recording = await openRecording(await buildJsonTelemetryMcap(3))
    const reported: CsvExportProgress[] = []

    await exportChannelsAsCsv(recording, allChannelIds(recording.reader), {
      onProgress: (progress) => reported.push(progress),
    })

    expect(reported.map((progress) => csvExportPercentage(progress))).toEqual([33])
    expect(csvExportPercentage({ messages: 3, expectedMessages: 3, bytes: 90 })).toBeLessThan(100)
  })

  it('builds the file from parts of about 1 MB, so no single step copies the whole CSV', async () => {
    const recording = await openRecording(await buildJsonTelemetryMcap(100_000))
    const textPerBlob: number[] = []
    vi.stubGlobal('Blob', class extends Blob {
      constructor(parts: BlobPart[], options?: BlobPropertyBag) {
        super(parts, options)
        textPerBlob.push(parts.reduce((length, part) => length + (typeof part === 'string' ? part.length : 0), 0))
      }
    })

    const blob = await exportChannelsAsCsv(recording, allChannelIds(recording.reader))

    expect(blob.size).toBeGreaterThan(1.1 * 1024 * 1024)
    expect(Math.max(...textPerBlob)).toBeLessThan(1.1 * 1024 * 1024)
  }, 60_000)

  it('lets the page run while a long export goes on, and hears a cancel', async () => {
    const recording = await openRecording(await buildJsonTelemetryMcap(200_000, 1024 * 1024))
    const channelIds = allChannelIds(recording.reader)
    let timerRan = false
    setTimeout(() => { timerRan = true }, 0)

    await exportChannelsAsCsv(recording, channelIds)
    expect(timerRan).toBe(true)

    const controller = new AbortController()
    setTimeout(() => controller.abort(), 0)
    await expect(exportChannelsAsCsv(recording, channelIds, { signal: controller.signal }))
      .rejects.toMatchObject({ name: 'AbortError' })
  }, 60_000)
})
