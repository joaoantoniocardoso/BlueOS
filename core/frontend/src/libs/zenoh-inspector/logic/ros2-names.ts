import type { RmwZenohDataKeyFields, Ros2EntityKind, Ros2Info } from './types'

const RMW_ZENOH_PREFIX = '@ros2_lv/'

const RMW_ENTITY_KIND: Record<string, Ros2EntityKind> = {
  MP: 'publisher',
  MS: 'subscriber',
  SS: 'service_server',
  SC: 'service_client',
  AS: 'action_server',
  AC: 'action_client',
}

const ROS2DDS_ENTITY_KIND: Record<string, Ros2EntityKind> = {
  MP: 'publisher',
  MS: 'subscriber',
  SS: 'service_server',
  SC: 'service_client',
  AS: 'action_server',
  AC: 'action_client',
}

const DDS_TO_ROS_RE = /^(.+)::(msg|srv|action)::dds_::(.+)_$/

function unmangleRmwZenohSegment(segment: string): string {
  if (segment === '%') {
    return '/'
  }
  const replaced = segment.replace(/%/g, '/')
  return replaced.startsWith('/') ? replaced : `/${replaced}`
}

export function ddsToRosTypeName(dds: string): string | undefined {
  const match = dds.match(DDS_TO_ROS_RE)
  if (!match) {
    return undefined
  }
  return `${match[1]}/${match[2]}/${match[3]}`
}

export function ros2ddsTopicFromDataKey(key: string): string {
  if (key.length === 0) {
    return '/'
  }
  if (key.startsWith('/')) {
    return key
  }
  return `/${key}`
}

export function parseRmwZenohDataKey(key: string): RmwZenohDataKeyFields | undefined {
  const segments = key.split('/')
  if (segments.length < 4) {
    return undefined
  }
  const domainId = Number.parseInt(segments[0], 10)
  if (Number.isNaN(domainId)) {
    return undefined
  }
  const typeHash = segments[segments.length - 1]
  if (!typeHash.startsWith('RIHS01_')) {
    return undefined
  }
  const ddsType = segments[segments.length - 2]
  const typeName = ddsToRosTypeName(ddsType)
  if (!typeName) {
    return undefined
  }
  const topicSegments = segments.slice(1, segments.length - 2)
  if (topicSegments.length === 0) {
    return undefined
  }
  const topic = `/${topicSegments.join('/')}`
  return {
    domainId,
    topic,
    typeName,
    typeHash,
  }
}

export function parseRmwZenohToken(key: string): Ros2Info | undefined {
  if (!key.startsWith(RMW_ZENOH_PREFIX) || key.startsWith('@/')) {
    return undefined
  }

  const parts = key.slice(RMW_ZENOH_PREFIX.length).split('/')
  if (parts.length < 9) {
    return undefined
  }

  const domainId = Number.parseInt(parts[0], 10)
  if (Number.isNaN(domainId)) {
    return undefined
  }

  const kindCode = parts[4]
  if (kindCode === 'NN') {
    return undefined
  }

  const entityKind = RMW_ENTITY_KIND[kindCode]
  if (!entityKind) {
    return undefined
  }

  const tail = parts.slice(8)
  const typeIndex = tail.findIndex((segment) => segment.includes('::'))
  if (typeIndex < 0) {
    return undefined
  }
  const ddsType = tail[typeIndex]
  const typeName = ddsToRosTypeName(ddsType)
  const typeHash = tail[typeIndex + 1]
  if (!typeName || !typeHash?.startsWith('RIHS01_')) {
    return undefined
  }

  const topic = typeIndex === 1
    ? unmangleRmwZenohSegment(tail[0])
    : unmangleRmwZenohSegment(tail.slice(0, typeIndex).join('/'))

  const namespace = unmangleRmwZenohSegment(parts[6])

  return {
    transport: 'rmw_zenoh',
    entityKind,
    domainId,
    namespace,
    node: parts[7],
    topic,
    typeName,
    typeHash,
  }
}

export function parseRos2ddsToken(key: string): Ros2Info | undefined {
  const match = key.match(
    /^@\/([^/]+)\/@ros2_lv\/(MP|MS|SS|SC|AS|AC)\/([^/]+)\/([^/]+)(?:\/([^/]+))?$/,
  )
  if (!match) {
    return undefined
  }

  const entityKind = ROS2DDS_ENTITY_KIND[match[2]]
  if (!entityKind) {
    return undefined
  }

  const path = match[3].replace(/\u00a7/g, '/')
  const topic = ros2ddsTopicFromDataKey(path)
  const typeName = match[4].replace(/\u00a7/g, '/')
  if (!typeName.includes('/') || typeName.split('/').length !== 3) {
    return undefined
  }

  return {
    transport: 'ros2dds',
    entityKind,
    topic,
    typeName,
  }
}

export function ros2ddsLivelinessTokenToDataKey(key: string): string | undefined {
  const match = key.match(/^@\/([^/]+)\/@ros2_lv\/(MP|MS|SS|SC|AS|AC)\/([^/]+)/)
  if (!match) {
    return undefined
  }
  if (!ROS2DDS_ENTITY_KIND[match[2]]) {
    return undefined
  }
  return match[3].replace(/\u00a7/g, '/')
}
