import type { TopicGroup, TopicInfo } from './types'

export function topicPrimaryLabel(topic: TopicInfo): string {
  return topic.ros2?.topic ?? topic.key
}

function groupSectionTitle(group: TopicGroup): string {
  switch (group.source) {
    case 'blueos':
      return 'BlueOS'
    case 'rmw_zenoh':
      return 'ROS 2 rmw_zenoh'
    case 'ros2dds':
      return 'ROS 2 ros2dds'
    default:
      return 'Other'
  }
}

export function serviceAliveFromBlueosGroup(group: TopicGroup & { source: 'blueos' }): boolean | undefined {
  const token = group.topics.find((topic) => topic.blueos?.kind === 'service_liveliness')
  return token?.alive
}

export function visibleTopicsInGroup(group: TopicGroup): TopicInfo[] {
  if (group.source === 'blueos') {
    return group.topics.filter((topic) => topic.blueos?.kind !== 'service_liveliness')
  }
  return group.topics
}

export type TopicListRow =
  | { kind: 'heading', rowKey: string, title: string }
  | { kind: 'service', rowKey: string, service: string, alive: boolean | undefined }
  | { kind: 'topic', rowKey: string, topic: TopicInfo }

export function flattenTopicGroups(groups: TopicGroup[]): TopicListRow[] {
  const rows: TopicListRow[] = []
  let blueosHeadingAdded = false
  for (const group of groups) {
    if (group.source === 'blueos') {
      if (!blueosHeadingAdded) {
        rows.push({ kind: 'heading', rowKey: 'heading-blueos', title: 'BlueOS' })
        blueosHeadingAdded = true
      }
      rows.push({
        kind: 'service',
        rowKey: `service-${group.service}`,
        service: group.service,
        alive: serviceAliveFromBlueosGroup(group),
      })
      for (const topic of visibleTopicsInGroup(group)) {
        rows.push({ kind: 'topic', rowKey: topic.key, topic })
      }
    } else {
      rows.push({
        kind: 'heading',
        rowKey: `heading-${group.source}`,
        title: groupSectionTitle(group),
      })
      for (const topic of group.topics) {
        rows.push({ kind: 'topic', rowKey: topic.key, topic })
      }
    }
  }
  return rows
}

export function topicCountInGroups(groups: TopicGroup[]): number {
  let count = 0
  for (const group of groups) {
    count += visibleTopicsInGroup(group).length
  }
  return count
}
