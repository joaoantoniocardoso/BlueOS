/**
 * Exports selected MCAP channels as one interleaved CSV file in the browser.
 */
import { parse as parseMessageDefinition } from '@foxglove/rosmsg'
import { MessageReader } from '@foxglove/rosmsg2-serialization'

import { sleep, throwIfAborted } from './abort'
import { createProgressGate } from './progress'
import {
  McapChannel, McapIndexedReader, McapMessage,
} from './reader'
import { decodeSchemaDefinition } from './video-track'

export interface Mp4ExportRange {
  startSeconds: number
  endSeconds: number
}

const BASE_COLUMNS = ['log_time', 'topic', 'sequence'] as const

/** How long the export runs before it gives the page a turn, so progress renders and Cancel is heard. */
const YIELD_INTERVAL_MS = 50

/**
 * Lines are joined into string parts of about this many characters, and each part becomes a `Blob` at once, so the
 * final `Blob` joins `Blob`s instead of copying the whole CSV text in one long task.
 */
const PART_CHARACTERS = 1024 * 1024

/** `expectedMessages` comes from the channel message counts, scaled to the range; zero when unknown. */
export interface CsvExportProgress {
  bytes: number
  messages: number
  expectedMessages: number
}

export interface CsvExportOptions {
  range?: Mp4ExportRange
  onProgress?: (progress: CsvExportProgress) => void
  signal?: AbortSignal
}

export interface McapCsvRecording {
  reader: McapIndexedReader
  startTime: bigint
  durationSeconds: number
}

interface DecodedRow {
  logTime: bigint
  topic: string
  sequence: number
  fields: Record<string, string>
}

interface ChannelCursor {
  channelId: number
  topic: string
  positions: number[]
  positionIndex: number
  messages: McapMessage[]
  messageIndex: number
}

function toSeconds(startTime: bigint, logTime: bigint): number {
  return Number(logTime - startTime) / 1e9
}

function csvEscape(value: string): string {
  if (/[",\n\r]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`
  }
  return value
}

function formatCell(value: unknown): string {
  if (value === null || value === undefined) {
    return ''
  }
  if (typeof value === 'boolean') {
    return value ? 'true' : 'false'
  }
  if (typeof value === 'number' || typeof value === 'bigint') {
    return String(value)
  }
  if (typeof value === 'string') {
    return value
  }
  // `JSON.stringify` throws on a 64-bit integer (`uint64[]` inside a list of messages): keep it as a decimal string.
  return JSON.stringify(value, (_key, nested: unknown) => {
    if (typeof nested === 'bigint') {
      return nested.toString()
    }
    return nested
  })
}

function flattenValue(value: unknown, prefix: string, fields: Record<string, string>): void {
  if (value === null || value === undefined) {
    return
  }
  if (value instanceof Uint8Array) {
    const key = prefix === 'data' || prefix.endsWith('.data')
      ? prefix.replace(/\.?data$/, '.data_bytes')
      : `${prefix}_bytes`
    fields[key] = String(value.length)
    return
  }
  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index += 1) {
      flattenValue(value[index], `${prefix}.${index}`, fields)
    }
    return
  }
  if (typeof value === 'object') {
    for (const [key, nested] of Object.entries(value as Record<string, unknown>)) {
      const path = prefix ? `${prefix}.${key}` : key
      if (nested instanceof Uint8Array) {
        // eslint-disable-next-line no-nested-ternary
        const bytesKey = key === 'data' ? prefix ? `${prefix}.data_bytes` : 'data_bytes' : `${path}_bytes`
        fields[bytesKey] = String(nested.length)
      } else if (nested !== null && typeof nested === 'object' && !Array.isArray(nested)) {
        flattenValue(nested, path, fields)
      } else {
        fields[path] = formatCell(nested)
      }
    }
    return
  }
  fields[prefix] = formatCell(value)
}

class ChannelDecoder {
  private readonly topic: string

  private readonly opaque: boolean

  private messageReader: MessageReader | null = null

  constructor(private readonly reader: McapIndexedReader, private readonly channel: McapChannel) {
    this.topic = channel.topic
    this.opaque = channel.messageEncoding === 'octet-stream'
    if (!this.opaque && channel.messageEncoding === 'cdr') {
      const schema = reader.summary.schemas.get(channel.schemaId)
      if (!schema) {
        throw new Error(`Recording is missing the schema for ${channel.topic}.`)
      }
      const text = decodeSchemaDefinition(schema.data)
      this.messageReader = new MessageReader(parseMessageDefinition(text, { ros2: true }))
    }
  }

  decode(message: McapMessage): DecodedRow {
    if (this.opaque) {
      return {
        logTime: message.logTime,
        topic: this.topic,
        sequence: message.sequence,
        fields: { data_bytes: String(message.data.length) },
      }
    }
    const fields: Record<string, string> = {}
    if (this.channel.messageEncoding === 'json') {
      const parsed = JSON.parse(new TextDecoder().decode(message.data)) as unknown
      flattenValue(parsed, '', fields)
    } else if (this.messageReader) {
      flattenValue(this.messageReader.readMessage(message.data), '', fields)
    } else {
      throw new Error(`Unsupported message encoding '${this.channel.messageEncoding}' on ${this.topic}.`)
    }
    return {
      logTime: message.logTime,
      topic: this.topic,
      sequence: message.sequence,
      fields,
    }
  }
}

async function loadCursorChunk(
  cursor: ChannelCursor,
  reader: McapIndexedReader,
  signal?: AbortSignal,
): Promise<boolean> {
  while (cursor.positionIndex < cursor.positions.length) {
    const chunkIndex = cursor.positions[cursor.positionIndex]
    cursor.positionIndex += 1
    // ponytail: every selected channel reads each of its chunks itself, so the work grows with chunks times channels
    // (2 s for a 38 MB recording). Read each chunk once for all selected channels if profiling shows it matters.
    // eslint-disable-next-line no-await-in-loop
    const messages = await reader.readChunkMessages(chunkIndex, cursor.channelId, signal)
    if (messages.length > 0) {
      cursor.messages = messages
      cursor.messageIndex = 0
      return true
    }
  }
  cursor.messages = []
  cursor.messageIndex = 0
  return false
}

async function primeCursor(
  cursor: ChannelCursor,
  reader: McapIndexedReader,
  startLogTime: bigint,
  signal?: AbortSignal,
): Promise<void> {
  while (cursor.messageIndex >= cursor.messages.length) {
    // eslint-disable-next-line no-await-in-loop
    if (!await loadCursorChunk(cursor, reader, signal)) {
      return
    }
  }
  while (cursor.messageIndex < cursor.messages.length && cursor.messages[cursor.messageIndex].logTime < startLogTime) {
    cursor.messageIndex += 1
    if (cursor.messageIndex >= cursor.messages.length) {
      // eslint-disable-next-line no-await-in-loop
      await loadCursorChunk(cursor, reader, signal)
    }
  }
}

async function advanceCursor(
  cursor: ChannelCursor,
  reader: McapIndexedReader,
  signal?: AbortSignal,
): Promise<void> {
  cursor.messageIndex += 1
  if (cursor.messageIndex >= cursor.messages.length) {
    await loadCursorChunk(cursor, reader, signal)
  }
}

function cursorHead(cursor: ChannelCursor): McapMessage | null {
  if (cursor.messageIndex >= cursor.messages.length) {
    return null
  }
  return cursor.messages[cursor.messageIndex]
}

async function buildCursors(
  reader: McapIndexedReader,
  channelIds: number[],
  startLogTime: bigint,
  signal?: AbortSignal,
): Promise<ChannelCursor[]> {
  await reader.loadChunkIndexesUntil(startLogTime, signal)
  const cursors: ChannelCursor[] = []
  for (const channelId of channelIds) {
    const channel = reader.summary.channels.get(channelId)
    if (!channel) {
      continue
    }
    const cursor: ChannelCursor = {
      channelId,
      topic: channel.topic,
      positions: reader.chunkIndexesForChannel(channelId),
      positionIndex: reader.findChunkIndexAtTime(channelId, startLogTime),
      messages: [],
      messageIndex: 0,
    }
    // eslint-disable-next-line no-await-in-loop
    await primeCursor(cursor, reader, startLogTime, signal)
    if (cursorHead(cursor)) {
      cursors.push(cursor)
    }
  }
  return cursors
}

/**
 * Writes each decoded row as a CSV line at once, so nothing but the CSV text is kept. A column gets its position the
 * first time it appears, which puts each topic's fields together; a row written before a column first appeared is
 * shorter than the header, and CSV readers fill the missing cells with empty values.
 */
async function writeRows(
  recording: McapCsvRecording,
  channelIds: number[],
  startLogTime: bigint,
  endLogTime: bigint | null,
  expectedMessages: number,
  onProgress?: (progress: CsvExportProgress) => void,
  signal?: AbortSignal,
): Promise<{ parts: Blob[], fieldColumns: string[], messages: number }> {
  const { reader, startTime } = recording
  const decoders = new Map<number, ChannelDecoder>()
  for (const channelId of channelIds) {
    const channel = reader.summary.channels.get(channelId)
    if (channel) {
      decoders.set(channelId, new ChannelDecoder(reader, channel))
    }
  }

  const cursors = await buildCursors(reader, channelIds, startLogTime, signal)
  const columnPositions = new Map<string, number>()
  const parts: Blob[] = []
  let part = ''
  let bytes = 0
  let messages = 0
  let lastYield = Date.now()
  const shouldReport = createProgressGate()

  for (;;) {
    throwIfAborted(signal, 'The export was cancelled.')
    let nextCursor: ChannelCursor | null = null
    let nextMessage: McapMessage | null = null
    for (const cursor of cursors) {
      const message = cursorHead(cursor)
      if (!message) {
        continue
      }
      if (!nextMessage || message.logTime < nextMessage.logTime) {
        nextCursor = cursor
        nextMessage = message
      }
    }
    if (!nextCursor || !nextMessage) {
      break
    }
    if (endLogTime !== null && nextMessage.logTime > endLogTime) {
      break
    }

    const decoder = decoders.get(nextCursor.channelId)
    if (decoder) {
      const row = decoder.decode(nextMessage)
      const line = rowLine(row, columnPositions, startTime)
      part += line
      if (part.length >= PART_CHARACTERS) {
        parts.push(new Blob([part]))
        part = ''
      }
      messages += 1
      bytes += line.length
      if (onProgress && shouldReport()) {
        onProgress({ messages, expectedMessages, bytes })
      }
    }
    // ponytail: the export still decodes on the main thread and only yields between rows. A Web Worker over
    // libs/mcap/logic (no DOM there) is the upgrade if long exports still feel slow.
    if (Date.now() - lastYield >= YIELD_INTERVAL_MS) {
      // eslint-disable-next-line no-await-in-loop
      await sleep(0, signal)
      lastYield = Date.now()
    }
    // eslint-disable-next-line no-await-in-loop
    await advanceCursor(nextCursor, reader, signal)
  }

  if (part) {
    parts.push(new Blob([part]))
  }
  return { parts, fieldColumns: [...columnPositions.keys()], messages }
}

function rowLine(row: DecodedRow, columnPositions: Map<string, number>, startTime: bigint): string {
  const cells = [String(toSeconds(startTime, row.logTime)), csvEscape(row.topic), String(row.sequence)]
  for (const [column, value] of Object.entries(row.fields)) {
    let position = columnPositions.get(column)
    if (position === undefined) {
      position = columnPositions.size
      columnPositions.set(column, position)
    }
    cells[BASE_COLUMNS.length + position] = csvEscape(value)
  }
  return `${Array.from(cells, (cell) => cell ?? '').join(',')}\n`
}

/** Reads the selected channels over a clip range and returns one interleaved CSV file. */
export async function exportChannelsAsCsv(
  recording: McapCsvRecording,
  channelIds: number[],
  options: CsvExportOptions = {},
): Promise<Blob> {
  if (channelIds.length === 0) {
    throw new Error('Select at least one channel to export.')
  }

  const { onProgress, signal, range } = options
  const startSeconds = Math.max(0, range?.startSeconds ?? 0)
  const endSeconds = Math.min(range?.endSeconds ?? Infinity, recording.durationSeconds)
  const durationSeconds = Math.max(endSeconds - startSeconds, 0)
  const startLogTime = recording.startTime + BigInt(Math.round(startSeconds * 1e9))
  const endLogTime = range && Number.isFinite(range.endSeconds)
    ? recording.startTime + BigInt(Math.round(endSeconds * 1e9))
    : null
  const channelMessages = channelIds.reduce(
    (total, channelId) => total + Number(recording.reader.summary.messageCountByChannel.get(channelId) ?? 0n),
    0,
  )
  const rangeFraction = range && recording.durationSeconds > 0 ? durationSeconds / recording.durationSeconds : 1

  const { parts, fieldColumns, messages } = await writeRows(
    recording,
    channelIds,
    startLogTime,
    endLogTime,
    Math.round(channelMessages * rangeFraction),
    onProgress,
    signal,
  )

  if (messages === 0) {
    throw new Error('The selected part of this recording holds no messages for the chosen channels.')
  }

  const header = `${[...BASE_COLUMNS, ...fieldColumns].join(',')}\n`
  return new Blob([header, ...parts], { type: 'text/csv' })
}
