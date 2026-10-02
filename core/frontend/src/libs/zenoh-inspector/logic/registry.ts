import { parseBlueosKey } from './blueos-keys'
import { parseRmwZenohDataKey, parseRmwZenohToken, parseRos2ddsToken } from './ros2-names'
import type {
  InspectorState, Ros2EntityKind, Ros2Info, SampleRecord, TopicGroup, TopicInfo, TopicSource,
} from './types'

function emptyTopics(): Record<string, TopicInfo> {
  return Object.create(null) as Record<string, TopicInfo>
}

export function createInspectorState(): InspectorState {
  return { topics: emptyTopics() }
}

function encodingBase(encoding: string): string {
  const semicolon = encoding.indexOf(';')
  return semicolon < 0 ? encoding : encoding.slice(0, semicolon)
}

function inferSource(key: string): { source: TopicSource, blueos?: TopicInfo['blueos'], ros2?: Ros2Info } {
  const blueos = parseBlueosKey(key)
  if (blueos) {
    return { source: 'blueos', blueos }
  }

  const rmw = parseRmwZenohDataKey(key)
  if (rmw) {
    return { source: 'rmw_zenoh', ros2: rmw }
  }

  // A ros2dds data key is indistinguishable from any other key; only its publisher token marks it as ROS 2.
  return { source: 'raw' }
}

function ros2ddsDataKeyFromRosTopic(topic: string): string {
  return topic.startsWith('/') ? topic.slice(1) : topic
}

function isPublisherToken(ros2: Ros2Info): boolean {
  return ros2.entityKind === 'publisher'
}

function isRos2ddsServiceOrAction(entityKind: Ros2EntityKind): boolean {
  return entityKind === 'service_server'
    || entityKind === 'service_client'
    || entityKind === 'action_server'
    || entityKind === 'action_client'
}

function nonPublisherEntityKey(ros2: Ros2Info): string {
  const topicSegment = ros2.topic.replace(/\//g, '%')
  const typeSegment = ros2.typeName.replace(/\//g, '%')
  const nodeSegment = ros2.node ?? '_'
  return `@entity/${ros2.transport}/${ros2.entityKind}/${topicSegment}/${nodeSegment}/${typeSegment}`
}

function rmwRos2Matches(dataRos2: Ros2Info, tokenRos2: Ros2Info): boolean {
  return dataRos2.domainId === tokenRos2.domainId
    && dataRos2.topic === tokenRos2.topic
    && dataRos2.typeHash === tokenRos2.typeHash
}

function rmwDataKeysForToken(state: InspectorState, tokenRos2: Ros2Info): string[] {
  const keys: string[] = []
  for (const key of Object.keys(state.topics)) {
    const fromKey = parseRmwZenohDataKey(key)
    if (fromKey && rmwRos2Matches(fromKey, tokenRos2)) {
      keys.push(key)
    }
  }
  return keys
}

function hasRmwDataKeyForToken(state: InspectorState, tokenRos2: Ros2Info): boolean {
  return rmwDataKeysForToken(state, tokenRos2).length > 0
}

function removeTopic(state: InspectorState, key: string): InspectorState {
  if (!state.topics[key]) {
    return state
  }
  const topics = emptyTopics()
  for (const existingKey of Object.keys(state.topics)) {
    if (existingKey !== key) {
      topics[existingKey] = state.topics[existingKey]
    }
  }
  return { topics }
}

function mergeRmwDataWithPublisherToken(dataRos2: Ros2Info, publisherRos2: Ros2Info): Ros2Info {
  return {
    ...dataRos2,
    entityKind: 'publisher',
    node: publisherRos2.node,
    namespace: publisherRos2.namespace,
  }
}

function updatePublisherTokens(
  existing: TopicInfo | undefined,
  tokenKey: string,
  alive: boolean,
): string[] {
  const current = existing?.publisherTokens ?? []
  if (alive) {
    if (current.includes(tokenKey)) {
      return current
    }
    return [...current, tokenKey]
  }
  return current.filter((entry) => entry !== tokenKey)
}

function isHiddenEntityOnlyTopic(state: InspectorState, key: string): boolean {
  const topic = state.topics[key]
  if (!topic?.entityOnly || !topic.ros2 || !isPublisherToken(topic.ros2)) {
    return false
  }
  return topic.ros2.transport === 'rmw_zenoh' && hasRmwDataKeyForToken(state, topic.ros2)
}

function visibleTopicKeys(state: InspectorState): string[] {
  return Object.keys(state.topics).filter((key) => !isHiddenEntityOnlyTopic(state, key))
}

function upsertTopic(state: InspectorState, key: string, patch: Partial<TopicInfo>): InspectorState {
  const existing = state.topics[key]
  const topic: TopicInfo = {
    key,
    sampleCount: 0,
    ...inferSource(key),
    ...existing,
    ...patch,
  }
  const topics = emptyTopics()
  for (const existingKey of Object.keys(state.topics)) {
    topics[existingKey] = state.topics[existingKey]
  }
  topics[key] = topic
  return { topics }
}

function applyPublisherToDataKey(
  state: InspectorState,
  dataKey: string,
  tokenKey: string,
  publisherRos2: Ros2Info,
  alive: boolean,
  dataRos2?: Ros2Info,
): InspectorState {
  const existing = state.topics[dataKey]
  const publisherTokens = updatePublisherTokens(existing, tokenKey, alive)
  const ros2 = dataRos2
    ? mergeRmwDataWithPublisherToken(dataRos2, publisherRos2)
    : { ...publisherRos2, entityKind: 'publisher' as const }
  return upsertTopic(state, dataKey, {
    alive: publisherTokens.length > 0,
    publisherTokens,
    ros2,
    source: publisherRos2.transport,
    schemaName: publisherRos2.typeName,
    entityOnly: false,
  })
}

function applyRmwPublisherLiveliness(
  state: InspectorState,
  tokenKey: string,
  publisherRos2: Ros2Info,
  alive: boolean,
): InspectorState {
  const dataKeys = rmwDataKeysForToken(state, publisherRos2)
  if (dataKeys.length > 0) {
    let next = state
    for (const dataKey of dataKeys) {
      const dataRos2 = parseRmwZenohDataKey(dataKey)
      if (dataRos2) {
        next = applyPublisherToDataKey(next, dataKey, tokenKey, publisherRos2, alive, dataRos2)
      }
    }
    if (next.topics[publisherRos2.topic]?.entityOnly) {
      next = removeTopic(next, publisherRos2.topic)
    }
    return next
  }

  const placeholderKey = publisherRos2.topic
  const existing = state.topics[placeholderKey]
  const publisherTokens = updatePublisherTokens(existing, tokenKey, alive)
  if (publisherTokens.length === 0) {
    return removeTopic(state, placeholderKey)
  }
  return upsertTopic(state, placeholderKey, {
    alive: true,
    publisherTokens,
    ros2: publisherRos2,
    source: 'rmw_zenoh',
    schemaName: publisherRos2.typeName,
    entityOnly: true,
  })
}

function applyRos2ddsPublisherLiveliness(
  state: InspectorState,
  tokenKey: string,
  publisherRos2: Ros2Info,
  alive: boolean,
): InspectorState {
  const dataKey = ros2ddsDataKeyFromRosTopic(publisherRos2.topic)
  const existing = state.topics[dataKey]
  const hasTraffic = (existing?.sampleCount ?? 0) > 0

  if (hasTraffic) {
    return applyPublisherToDataKey(state, dataKey, tokenKey, publisherRos2, alive)
  }

  const publisherTokens = updatePublisherTokens(existing, tokenKey, alive)
  if (publisherTokens.length === 0) {
    return removeTopic(state, dataKey)
  }
  return upsertTopic(state, dataKey, {
    alive: true,
    publisherTokens,
    ros2: { ...publisherRos2, entityKind: 'publisher' },
    source: 'ros2dds',
    schemaName: publisherRos2.typeName,
    entityOnly: true,
  })
}

function applyNonPublisherLiveliness(
  state: InspectorState,
  ros2: Ros2Info,
  alive: boolean,
): InspectorState {
  const entityKey = nonPublisherEntityKey(ros2)
  if (!alive) {
    return removeTopic(state, entityKey)
  }
  return upsertTopic(state, entityKey, {
    alive: true,
    ros2,
    source: ros2.transport,
    schemaName: ros2.typeName,
    entityOnly: true,
  })
}

function joinRmwSampleWithEntity(
  before: InspectorState,
  after: InspectorState,
  sampleKey: string,
): InspectorState {
  const parsed = parseRmwZenohDataKey(sampleKey)
  if (!parsed) {
    return after
  }
  const entity = before.topics[parsed.topic]
  if (!entity?.entityOnly || !entity.ros2 || entity.ros2.entityKind !== 'publisher') {
    return after
  }
  let next = upsertTopic(after, sampleKey, {
    alive: entity.alive,
    publisherTokens: entity.publisherTokens,
    ros2: mergeRmwDataWithPublisherToken(parsed, entity.ros2),
    schemaName: entity.schemaName ?? parsed.typeName,
    entityOnly: false,
  })
  next = removeTopic(next, parsed.topic)
  return next
}

export function applySample(state: InspectorState, sample: SampleRecord): InspectorState {
  const existing = state.topics[sample.key]
  if (sample.kind === 'delete') {
    if (!existing) {
      return state
    }
    return removeTopic(state, sample.key)
  }

  const sampleCount = (existing?.sampleCount ?? 0) + 1
  let next = upsertTopic(state, sample.key, {
    encoding: encodingBase(sample.encoding),
    lastSample: sample,
    sampleCount,
    entityOnly: false,
  })
  next = joinRmwSampleWithEntity(state, next, sample.key)
  return next
}

export function applyRos2Liveliness(
  state: InspectorState,
  token: string,
  alive: boolean,
): InspectorState {
  const rmw = parseRmwZenohToken(token)
  const ros2 = rmw ?? parseRos2ddsToken(token)
  if (!ros2) {
    return state
  }

  if (ros2.transport === 'ros2dds' && isRos2ddsServiceOrAction(ros2.entityKind)) {
    return applyNonPublisherLiveliness(state, ros2, alive)
  }

  if (!isPublisherToken(ros2)) {
    return applyNonPublisherLiveliness(state, ros2, alive)
  }

  if (ros2.transport === 'rmw_zenoh') {
    return applyRmwPublisherLiveliness(state, token, ros2, alive)
  }

  return applyRos2ddsPublisherLiveliness(state, token, ros2, alive)
}

export function applyBlueosServiceLiveliness(
  state: InspectorState,
  service: string,
  alive: boolean,
): InspectorState {
  const key = `blueos/v1/services/${service}`
  return upsertTopic(state, key, {
    alive,
    source: 'blueos',
    blueos: { service, kind: 'service_liveliness', name: '' },
  })
}

export function sortedTopicKeys(state: InspectorState): string[] {
  return visibleTopicKeys(state).sort()
}

export function topicsBySource(state: InspectorState): TopicGroup[] {
  const blueosByService = Object.create(null) as Record<string, TopicInfo[]>
  const rmw: TopicInfo[] = []
  const ros2dds: TopicInfo[] = []
  const raw: TopicInfo[] = []

  for (const key of visibleTopicKeys(state)) {
    const topic = state.topics[key]
    if (topic.source === 'blueos' && topic.blueos) {
      const { service } = topic.blueos
      if (!blueosByService[service]) {
        blueosByService[service] = []
      }
      blueosByService[service].push(topic)
    } else if (topic.source === 'rmw_zenoh') {
      rmw.push(topic)
    } else if (topic.source === 'ros2dds') {
      ros2dds.push(topic)
    } else {
      raw.push(topic)
    }
  }

  const groups: TopicGroup[] = []
  for (const service of Object.keys(blueosByService).sort()) {
    groups.push({
      source: 'blueos',
      service,
      topics: blueosByService[service].sort((left, right) => left.key.localeCompare(right.key)),
    })
  }
  if (rmw.length > 0) {
    groups.push({
      source: 'rmw_zenoh',
      topics: rmw.sort((left, right) => left.key.localeCompare(right.key)),
    })
  }
  if (ros2dds.length > 0) {
    groups.push({
      source: 'ros2dds',
      topics: ros2dds.sort((left, right) => left.key.localeCompare(right.key)),
    })
  }
  if (raw.length > 0) {
    groups.push({
      source: 'raw',
      topics: raw.sort((left, right) => left.key.localeCompare(right.key)),
    })
  }
  return groups
}
