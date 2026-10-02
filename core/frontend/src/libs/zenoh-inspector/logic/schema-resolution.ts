import { parseRmwZenohDataKey } from './ros2-names'
import type { SchemaProvider, TopicInfo } from './types'

export const FOXGLOVE_LOG_SCHEMA = 'foxglove_msgs/msg/Log' as const

function blueosLogSchemaName(topic: TopicInfo): string | undefined {
  if (topic.blueos?.kind === 'log') {
    return FOXGLOVE_LOG_SCHEMA
  }
  return undefined
}

export function schemaNameFromEncoding(encoding: string): string | undefined {
  const semicolon = encoding.indexOf(';')
  if (semicolon < 0) {
    return undefined
  }
  const schemaName = encoding.slice(semicolon + 1)
  return schemaName.length > 0 ? schemaName : undefined
}

export function transportTypeName(topic: TopicInfo): string | undefined {
  if (topic.ros2?.typeName) {
    return topic.ros2.typeName
  }
  const fromKey = parseRmwZenohDataKey(topic.key)
  return fromKey?.typeName
}

export function resolveSchemaName(topic: TopicInfo, provider: SchemaProvider): string | undefined {
  const encoding = topic.lastSample?.encoding ?? (topic.encoding ? `${topic.encoding}` : undefined)
  if (encoding) {
    const fromEncoding = schemaNameFromEncoding(encoding)
    if (fromEncoding !== undefined && provider.schemaText(fromEncoding) !== undefined) {
      return fromEncoding
    }
  }

  const fromTransport = transportTypeName(topic)
  if (fromTransport !== undefined && provider.schemaText(fromTransport) !== undefined) {
    return fromTransport
  }

  const fromBlueosLog = blueosLogSchemaName(topic)
  if (fromBlueosLog !== undefined && provider.schemaText(fromBlueosLog) !== undefined) {
    return fromBlueosLog
  }

  return undefined
}
