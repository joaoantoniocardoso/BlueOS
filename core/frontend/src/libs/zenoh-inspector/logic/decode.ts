import { ENCODING_APPLICATION_CDR } from '@/libs/blueos-api/keys'

import { resolveSchemaName, schemaNameFromEncoding } from './schema-resolution'
import { encodingBase, payloadIsCdrCandidate } from './topic-classification'
import type {
  CdrCodec, DecodedPayload, SampleRecord, SchemaProvider, TopicInfo,
} from './types'

const TEXT_PLAIN = 'text/plain'
const APPLICATION_JSON = 'application/json'
const ZENOH_BYTES = 'zenoh/bytes'

const BINARY_PREVIEW_BYTES = 64
const DISPLAY_ARRAY_PREVIEW_BYTES = 32
const DISPLAY_ARRAY_MAX_LENGTH = 64

export function bytesToHex(payload: Uint8Array, maxBytes: number): string {
  const length = Math.min(payload.length, maxBytes)
  let hex = ''
  for (let index = 0; index < length; index += 1) {
    hex += payload[index].toString(16).padStart(2, '0')
  }
  return hex
}

function binaryPreview(payload: Uint8Array): DecodedPayload {
  return {
    kind: 'binary',
    size: payload.length,
    preview: bytesToHex(payload, BINARY_PREVIEW_BYTES),
  }
}

function tryParseJson(text: string): unknown | undefined {
  try {
    return JSON.parse(text) as unknown
  } catch {
    return undefined
  }
}

function decodeText(payload: Uint8Array): DecodedPayload {
  const text = new TextDecoder().decode(payload)
  return { kind: 'text', value: text }
}

function decodeCdrPayload(
  resolvedName: string,
  payload: Uint8Array,
  provider: SchemaProvider,
  codec: CdrCodec,
): DecodedPayload {
  const schemaText = provider.schemaText(resolvedName)
  if (!schemaText) {
    return binaryPreview(payload)
  }
  try {
    const value = codec.decode(resolvedName, schemaText, payload)
    return { kind: 'cdr', schemaName: resolvedName, value }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    return {
      kind: 'error',
      message,
      size: payload.length,
      preview: bytesToHex(payload, BINARY_PREVIEW_BYTES),
    }
  }
}

function resolvePayloadSchemaName(
  topic: TopicInfo,
  sample: SampleRecord,
  schemaName: string | undefined,
  provider: SchemaProvider,
): string | undefined {
  const topicForResolution: TopicInfo = {
    ...topic,
    lastSample: sample,
    encoding: encodingBase(sample.encoding),
  }
  return schemaName
    ?? schemaNameFromEncoding(sample.encoding)
    ?? resolveSchemaName(topicForResolution, provider)
}

function tryDecodeCdr(
  base: string,
  resolvedName: string | undefined,
  payload: Uint8Array,
  provider: SchemaProvider,
  codec: CdrCodec,
): DecodedPayload | undefined {
  if (!resolvedName) {
    return undefined
  }
  const ambiguous = base === ENCODING_APPLICATION_CDR || base === ZENOH_BYTES || base === ''
  if (!ambiguous) {
    return undefined
  }
  if (!payloadIsCdrCandidate(base, payload)) {
    return undefined
  }
  return decodeCdrPayload(resolvedName, payload, provider, codec)
}

export function decodePayload(
  topic: TopicInfo,
  schemaName: string | undefined,
  provider: SchemaProvider,
  codec: CdrCodec,
): DecodedPayload {
  const sample = topic.lastSample
  if (!sample) {
    return binaryPreview(new Uint8Array())
  }

  const { payload } = sample
  const base = encodingBase(sample.encoding)
  const resolvedName = resolvePayloadSchemaName(topic, sample, schemaName, provider)

  if (base === TEXT_PLAIN) {
    return decodeText(payload)
  }

  if (base === APPLICATION_JSON) {
    const text = new TextDecoder().decode(payload)
    const parsed = tryParseJson(text)
    if (parsed !== undefined) {
      return { kind: 'json', value: parsed }
    }
    return decodeText(payload)
  }

  const cdr = tryDecodeCdr(base, resolvedName, payload, provider, codec)
  if (cdr) {
    return cdr
  }

  if (base === ZENOH_BYTES || base === '') {
    const text = new TextDecoder().decode(payload)
    const parsed = tryParseJson(text)
    if (parsed !== undefined) {
      return { kind: 'json', value: parsed }
    }
    return binaryPreview(payload)
  }

  return binaryPreview(payload)
}

function isTypedArray(value: unknown): value is ArrayBufferView {
  return ArrayBuffer.isView(value) && !(value instanceof DataView)
}

function summarizeByteLike(value: ArrayBufferView | number[]): { bytes: number, preview: string } {
  const bytes = value instanceof Array ? value.length : value.byteLength
  const uint8 = value instanceof Array
    ? Uint8Array.from(value.slice(0, DISPLAY_ARRAY_PREVIEW_BYTES))
    : new Uint8Array(value.buffer, value.byteOffset, Math.min(value.byteLength, DISPLAY_ARRAY_PREVIEW_BYTES))
  return { bytes, preview: bytesToHex(uint8, DISPLAY_ARRAY_PREVIEW_BYTES) }
}

export function toDisplayValue(value: unknown): unknown {
  if (value === null || value === undefined) {
    return value
  }
  if (typeof value === 'bigint') {
    const asNumber = Number(value)
    return Number.isSafeInteger(asNumber) ? asNumber : value.toString()
  }
  if (typeof value === 'number' || typeof value === 'string' || typeof value === 'boolean') {
    return value
  }
  if (isTypedArray(value)) {
    return summarizeByteLike(value)
  }
  if (Array.isArray(value)) {
    if (value.length > DISPLAY_ARRAY_MAX_LENGTH && value.every((entry) => typeof entry === 'number')) {
      return summarizeByteLike(value as number[])
    }
    return value.map((entry) => toDisplayValue(entry))
  }
  if (typeof value === 'object') {
    const output: Record<string, unknown> = Object.create(null)
    for (const [key, entry] of Object.entries(value as Record<string, unknown>)) {
      output[key] = toDisplayValue(entry)
    }
    return output
  }
  return String(value)
}

function livelinessLabel(topic: TopicInfo): string {
  if (topic.alive === undefined) {
    return 'Unknown'
  }
  return topic.alive ? 'Alive' : 'Dead'
}

export function formatTopicJson(topic: TopicInfo, decoded: DecodedPayload): string {
  let payload: unknown
  switch (decoded.kind) {
    case 'text':
      payload = decoded.value
      break
    case 'json':
      payload = toDisplayValue(decoded.value)
      break
    case 'cdr':
      payload = toDisplayValue(decoded.value)
      break
    case 'binary':
      payload = { size: decoded.size, preview: decoded.preview }
      break
    case 'error':
      payload = {
        error: decoded.message,
        size: decoded.size,
        preview: decoded.preview,
      }
      break
    default:
      payload = null
  }

  const timestamp = topic.lastSample
    ? new Date(topic.lastSample.receivedAt).toISOString()
    : ''

  const formatted = {
    topic: topic.key,
    timestamp,
    liveliness: livelinessLabel(topic),
    source: topic.source,
    schema: (decoded.kind === 'cdr' ? decoded.schemaName : topic.schemaName) ?? 'Unknown',
    payload,
  }

  return JSON.stringify(formatted, null, 2)
}
