import type { EndpointInfo } from '@blueos-idl/messages'

import { COMMAND_ACK_SCHEMA } from '@/libs/blueos-api/types'

import type { CdrCodec, SchemaProvider } from './types'

/**
 * The schema names a client sends to and reads from `endpoint`, from its kind and interface type: a Job type takes
 * its Goal and acks with a `CommandAck`, a Query of a `.srv` takes its request and answers its response, and a State,
 * an Event or a Query of a `.msg` (a Job type's history) takes nothing.
 */
export function endpointSchemas(endpoint: EndpointInfo): { requestSchema: string, responseSchema: string } {
  switch (endpoint.kind) {
    case 'job':
      return { requestSchema: `${endpoint.interface_type}_Goal`, responseSchema: COMMAND_ACK_SCHEMA }
    case 'query':
      if (endpoint.interface_type.includes('/srv/')) {
        return {
          requestSchema: `${endpoint.interface_type}_Request`,
          responseSchema: `${endpoint.interface_type}_Response`,
        }
      }
      return { requestSchema: '', responseSchema: endpoint.interface_type }
    default:
      return { requestSchema: '', responseSchema: endpoint.interface_type }
  }
}

export function defaultRequestText(
  schemaName: string,
  provider: SchemaProvider,
  codec: CdrCodec,
): string | undefined {
  const schemaText = provider.schemaText(schemaName)
  if (!schemaText) {
    return undefined
  }
  const defaults = codec.defaults(schemaName, schemaText)
  return JSON.stringify(defaults, null, 2)
}

export function parseRequestText(
  text: string,
): { ok: true, value: Record<string, unknown> } | { ok: false, message: string } {
  try {
    const parsed = JSON.parse(text.trim() || '{}') as unknown
    if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return { ok: false, message: 'Request body must be a JSON object' }
    }
    return { ok: true, value: parsed as Record<string, unknown> }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    return { ok: false, message }
  }
}

export function encodeRequest(
  schemaName: string,
  value: Record<string, unknown>,
  provider: SchemaProvider,
  codec: CdrCodec,
): Uint8Array | undefined {
  const schemaText = provider.schemaText(schemaName)
  if (!schemaText) {
    return undefined
  }
  return codec.encode(schemaName, schemaText, value)
}

export function endpointsByKind(
  endpoints: EndpointInfo[],
): Record<string, EndpointInfo[]> {
  const grouped = Object.create(null) as Record<string, EndpointInfo[]>
  for (const endpoint of endpoints) {
    if (!grouped[endpoint.kind]) {
      grouped[endpoint.kind] = []
    }
    grouped[endpoint.kind].push(endpoint)
  }
  for (const kind of Object.keys(grouped)) {
    grouped[kind].sort((left, right) => left.name.localeCompare(right.name))
  }
  return grouped
}
