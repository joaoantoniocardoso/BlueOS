/* eslint-disable import/no-extraneous-dependencies */
import { SCHEMAS } from '@blueos-idl/schemas'
import { parse } from '@foxglove/rosmsg'
import { MessageReader, MessageWriter } from '@foxglove/rosmsg2-serialization'

import type { MessageForSchema, SchemaName } from './types'

const DECODE_PADDING_BYTES = 256

type ParsedDefinitions = ReturnType<typeof parse>

const definitionCache = new Map<SchemaName, ParsedDefinitions>()
const readerCache = new Map<SchemaName, MessageReader>()
const writerCache = new Map<SchemaName, MessageWriter>()

function orderDefinitionsForSchema(
  schemaName: SchemaName,
  definitions: ParsedDefinitions,
): ParsedDefinitions {
  const rootIndex = definitions.findIndex((definition) => definition.name === schemaName)
  if (rootIndex <= 0) {
    return definitions
  }
  const reordered = [...definitions]
  const [root] = reordered.splice(rootIndex, 1)
  reordered.unshift(root)
  return reordered
}

function getDefinitions(schemaName: SchemaName): ParsedDefinitions {
  let definitions = definitionCache.get(schemaName)
  if (definitions === undefined) {
    const schemaText = SCHEMAS[schemaName]
    if (schemaText === undefined) {
      throw new Error(`Unknown schema: ${schemaName}`)
    }
    definitions = orderDefinitionsForSchema(schemaName, parse(schemaText, { ros2: true }))
    definitionCache.set(schemaName, definitions)
  }
  return definitions
}

function getReader(schemaName: SchemaName): MessageReader {
  let reader = readerCache.get(schemaName)
  if (reader === undefined) {
    reader = new MessageReader(getDefinitions(schemaName))
    readerCache.set(schemaName, reader)
  }
  return reader
}

function getWriter(schemaName: SchemaName): MessageWriter {
  let writer = writerCache.get(schemaName)
  if (writer === undefined) {
    writer = new MessageWriter(getDefinitions(schemaName))
    writerCache.set(schemaName, writer)
  }
  return writer
}

function normalizeDecodedValue(value: unknown): unknown {
  if (typeof value === 'bigint') {
    return Number(value)
  }
  if (Array.isArray(value)) {
    return value.map((entry) => normalizeDecodedValue(entry))
  }
  // MessageReader returns numeric sequences as typed arrays; the generated types declare them as number[].
  if (ArrayBuffer.isView(value) && !(value instanceof DataView)) {
    return Array.from(value as unknown as ArrayLike<unknown>, (entry) => normalizeDecodedValue(entry))
  }
  if (value !== null && typeof value === 'object') {
    const record = value as Record<string, unknown>
    const normalized: Record<string, unknown> = {}
    for (const [key, nested] of Object.entries(record)) {
      normalized[key] = normalizeDecodedValue(nested)
    }
    return normalized
  }
  return value
}

/**
 * D-06: new writers may include trailing fields; @foxglove/rosmsg2-serialization ignores extra bytes.
 * Old writers missing trailing fields need zero padding before decode (see README).
 */
export function decodeCdr<Schema extends SchemaName>(
  schemaName: Schema,
  payload: Uint8Array,
): MessageForSchema<Schema> {
  const reader = getReader(schemaName)
  try {
    return normalizeDecodedValue(reader.readMessage(payload)) as MessageForSchema<Schema>
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    if (!message.includes('Out of bounds') && !message.includes('out of bounds')) {
      throw error
    }
    const padded = new Uint8Array(payload.length + DECODE_PADDING_BYTES)
    padded.set(payload)
    return normalizeDecodedValue(reader.readMessage(padded)) as MessageForSchema<Schema>
  }
}

export function encodeCdr<Schema extends SchemaName>(
  schemaName: Schema,
  message: MessageForSchema<Schema>,
): Uint8Array {
  const writer = getWriter(schemaName)
  return writer.writeMessage(message)
}

export function schemaNameFromEncoding(encoding: string): string | undefined {
  const separator = encoding.indexOf(';')
  if (separator < 0) {
    return undefined
  }
  return encoding.slice(separator + 1)
}
