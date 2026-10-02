import type { Ros2EntityKind, Ros2Info } from './types'

const RMW_ZENOH_PREFIX = '@ros2_lv/'

const RMW_ENTITY_KIND: Record<string, Ros2EntityKind> = {
  MP: 'publisher',
  MS: 'subscriber',
  SS: 'service_server',
  SC: 'service_client',
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
  const path = key.startsWith('/') ? key.slice(1) : key
  return `/${path}`
}

function ros2ddsUnescapeKey(segment: string): string {
  return segment.replace(/\u00a7/g, '/')
}

/** Extracts the Zenoh data key a ros2dds liveliness token advertises (slashes, no leading slash). */
export function ros2ddsLivelinessTokenToDataKey(key: string): string | undefined {
  if (!key.startsWith('@/')) {
    return undefined
  }
  const afterAt = key.slice(2)
  const zidEnd = afterAt.indexOf('/')
  if (zidEnd < 0) {
    return undefined
  }
  const rest = afterAt.slice(zidEnd + 1)
  if (!rest.startsWith('@ros2_lv/')) {
    return undefined
  }
  const segments = rest.slice('@ros2_lv/'.length).split('/')
  if (segments.length < 2) {
    return undefined
  }
  const entityKind = ROS2DDS_ENTITY_KIND[segments[0]]
  if (!entityKind) {
    return undefined
  }
  return ros2ddsUnescapeKey(segments[1])
}

export function parseRmwZenohDataKey(key: string): Ros2Info | undefined {
  const hashMatch = key.match(/\/(RIHS01_[a-f0-9]+)$/)
  if (!hashMatch) {
    return undefined
  }
  const typeHash = hashMatch[1]
  const beforeHash = key.slice(0, -hashMatch[0].length)

  const typeMatch = beforeHash.match(/^(.*)\/([^/]+::(?:msg|srv|action)::dds_::\w+_)$/)
  if (!typeMatch) {
    return undefined
  }

  const typeName = ddsToRosTypeName(typeMatch[2])
  if (!typeName) {
    return undefined
  }

  const domainMatch = typeMatch[1].match(/^(\d+)\/(.*)$/)
  if (!domainMatch) {
    return undefined
  }

  const domainId = Number.parseInt(domainMatch[1], 10)
  if (Number.isNaN(domainId)) {
    return undefined
  }

  const topicPath = domainMatch[2]
  const topic = topicPath.startsWith('/') ? topicPath : `/${topicPath}`

  return {
    transport: 'rmw_zenoh',
    entityKind: 'publisher',
    domainId,
    topic,
    typeName,
    typeHash,
  }
}

export function parseRmwZenohToken(key: string): Ros2Info | undefined {
  if (!key.startsWith(RMW_ZENOH_PREFIX)) {
    return undefined
  }

  const parts = key.slice(RMW_ZENOH_PREFIX.length).split('/')
  if (parts.length < 11) {
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

  const ddsType = parts[9]
  const typeName = ddsToRosTypeName(ddsType)
  const typeHash = parts[10]?.startsWith('RIHS01_') ? parts[10] : undefined
  if (!typeName || !typeHash) {
    return undefined
  }

  const namespace = unmangleRmwZenohSegment(parts[6])
  const topic = unmangleRmwZenohSegment(parts[8])

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
  const topic = path.startsWith('/') ? path : `/${path}`
  const typeName = match[4].replace(/\u00a7/g, '/')

  return {
    transport: 'ros2dds',
    entityKind,
    topic,
    typeName,
  }
}
