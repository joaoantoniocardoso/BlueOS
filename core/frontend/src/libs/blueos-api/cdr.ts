/* eslint-disable import/no-extraneous-dependencies */
import { SCHEMAS } from '@blueos-idl/schemas'
import type { MessageDefinition, MessageDefinitionField } from '@foxglove/message-definition'
import { parse } from '@foxglove/rosmsg'
import { MessageReader, MessageWriter } from '@foxglove/rosmsg2-serialization'

import type { MessageForSchema, SchemaName } from './types'

type ParsedDefinitions = ReturnType<typeof parse>

const definitionCache = new Map<SchemaName, ParsedDefinitions>()
const readerCache = new Map<string, MessageReader>()
const writerCache = new Map<SchemaName, MessageWriter>()

function getDefinitions(schemaName: SchemaName): ParsedDefinitions {
  let definitions = definitionCache.get(schemaName)
  if (definitions === undefined) {
    const schemaText = SCHEMAS[schemaName]
    if (schemaText === undefined) {
      throw new Error(`Unknown schema: ${schemaName}`)
    }
    definitions = parse(schemaText, { ros2: true })
    definitionCache.set(schemaName, definitions)
  }
  return definitions
}

function dataFields(definition: MessageDefinition): MessageDefinitionField[] {
  return definition.definitions.filter((field) => field.isConstant !== true)
}

function definitionsMap(definitions: ParsedDefinitions): Map<string, MessageDefinitionField[]> {
  return new Map(definitions.map((definition) => [definition.name ?? '', definition.definitions]))
}

// The generated schema text puts the root definition first, as MessageReader and MessageWriter expect.
function truncatedDefinitions(definitions: ParsedDefinitions, fieldCount: number): ParsedDefinitions {
  const [root, ...dependencies] = definitions
  const constants = root.definitions.filter((field) => field.isConstant === true)
  const fields = dataFields(root).slice(0, fieldCount)
  const truncatedRoot: MessageDefinition = {
    ...root,
    definitions: [...constants, ...fields],
  }
  return [truncatedRoot, ...dependencies]
}

function readerCacheKey(schemaName: SchemaName, fieldCount: number): string {
  return `${schemaName}:${fieldCount}`
}

function getReaderForFieldCount(schemaName: SchemaName, fieldCount: number): MessageReader {
  const cacheKey = readerCacheKey(schemaName, fieldCount)
  let reader = readerCache.get(cacheKey)
  if (reader === undefined) {
    const definitions = getDefinitions(schemaName)
    const fullCount = dataFields(definitions[0]).length
    const definitionsForReader = fieldCount >= fullCount
      ? definitions
      : truncatedDefinitions(definitions, fieldCount)
    reader = new MessageReader(definitionsForReader)
    readerCache.set(cacheKey, reader)
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

function ros2TimeDefault(): { sec: number; nanosec: number } {
  return { sec: 0, nanosec: 0 }
}

function fieldDefault(
  field: MessageDefinitionField,
  definitionsByName: Map<string, MessageDefinitionField[]>,
): unknown {
  if (field.isArray === true) {
    if (field.arrayLength !== undefined) {
      return Array.from({ length: field.arrayLength }, () => fieldDefault(
        { ...field, isArray: false, arrayLength: undefined },
        definitionsByName,
      ))
    }
    return []
  }
  if (field.isComplex === true) {
    const nestedFields = definitionsByName.get(field.type)
    if (nestedFields === undefined) {
      throw new Error(`Unrecognized complex type ${field.type}`)
    }
    return messageDefaults(nestedFields, definitionsByName)
  }
  if (field.type === 'bool') {
    return false
  }
  if (field.type === 'string') {
    return ''
  }
  if (field.type === 'time' || field.type === 'duration') {
    return ros2TimeDefault()
  }
  return 0
}

function messageDefaults(
  fields: MessageDefinitionField[],
  definitionsByName: Map<string, MessageDefinitionField[]>,
): Record<string, unknown> {
  const message: Record<string, unknown> = {}
  for (const field of fields) {
    if (field.isConstant === true) {
      continue
    }
    message[field.name] = fieldDefault(field, definitionsByName)
  }
  return message
}

function isOutOfBoundsDecodeError(error: unknown): boolean {
  if (error instanceof RangeError) {
    return true
  }
  const message = error instanceof Error ? error.message : String(error)
  return /out of bounds|outside the bounds/i.test(message)
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
 * Old writers missing trailing top-level fields are decoded with progressively shorter readers, then
 * missing fields are filled from ROS 2 defaults. Nested message fields are not partially defaulted.
 */
export function decodeCdr<Schema extends SchemaName>(
  schemaName: Schema,
  payload: Uint8Array,
): MessageForSchema<Schema> {
  const definitions = getDefinitions(schemaName)
  const rootFields = dataFields(definitions[0])
  const definitionsByName = definitionsMap(definitions)
  let lastBoundsError: unknown

  for (let fieldCount = rootFields.length; fieldCount >= 1; fieldCount -= 1) {
    try {
      const reader = getReaderForFieldCount(schemaName, fieldCount)
      const decoded = normalizeDecodedValue(reader.readMessage(payload)) as Record<string, unknown>
      if (fieldCount < rootFields.length) {
        for (let index = fieldCount; index < rootFields.length; index += 1) {
          const field = rootFields[index]
          decoded[field.name] = fieldDefault(field, definitionsByName)
        }
      }
      return decoded as MessageForSchema<Schema>
    } catch (error) {
      if (!isOutOfBoundsDecodeError(error)) {
        throw error
      }
      lastBoundsError = error
    }
  }

  if (lastBoundsError !== undefined) {
    throw lastBoundsError
  }
  throw new Error(`Failed to decode CDR for schema ${schemaName}`)
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
