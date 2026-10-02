import { describe, expect, it } from 'vitest'

import {
  flattenTopicGroups,
  topicCountInGroups,
  topicPrimaryLabel,
  visibleTopicsInGroup,
} from '@/libs/zenoh-inspector/logic/topic-list'
import type { TopicGroup } from '@/libs/zenoh-inspector/logic/types'

describe('topic-list', () => {
  it('labels ROS topics by name and others by key', () => {
    expect(topicPrimaryLabel({
      key: 'robot1/chatter',
      source: 'ros2dds',
      sampleCount: 0,
      ros2: {
        transport: 'ros2dds',
        entityKind: 'publisher',
        topic: '/chatter',
        typeName: 'std_msgs/msg/String',
      },
    })).toBe('/chatter')
    expect(topicPrimaryLabel({
      key: 'blueos/v1/recorder/state/library',
      source: 'blueos',
      sampleCount: 0,
    })).toBe('blueos/v1/recorder/state/library')
  })

  it('hides service liveliness keys from visible topic lists', () => {
    const group: TopicGroup = {
      source: 'blueos',
      service: 'recorder',
      topics: [
        {
          key: 'blueos/v1/services/recorder',
          source: 'blueos',
          sampleCount: 0,
          blueos: { service: 'recorder', kind: 'service_liveliness', name: '' },
        },
        {
          key: 'blueos/v1/recorder/state/library',
          source: 'blueos',
          sampleCount: 0,
          blueos: { service: 'recorder', kind: 'state', name: 'library' },
        },
      ],
    }
    expect(visibleTopicsInGroup(group).map((topic) => topic.key)).toEqual([
      'blueos/v1/recorder/state/library',
    ])
    expect(topicCountInGroups([group])).toBe(1)
  })

  it('flattens groups into headings, service rows, and topics', () => {
    const groups: TopicGroup[] = [
      {
        source: 'blueos',
        service: 'recorder',
        topics: [
          {
            key: 'blueos/v1/services/recorder',
            source: 'blueos',
            sampleCount: 0,
            alive: true,
            blueos: { service: 'recorder', kind: 'service_liveliness', name: '' },
          },
          {
            key: 'blueos/v1/recorder/state/library',
            source: 'blueos',
            sampleCount: 1,
            blueos: { service: 'recorder', kind: 'state', name: 'library' },
          },
        ],
      },
      {
        source: 'raw',
        topics: [{
          key: 'misc/key',
          source: 'raw',
          sampleCount: 0,
        }],
      },
    ]
    const rows = flattenTopicGroups(groups)
    expect(rows.map((row) => row.kind)).toEqual([
      'heading',
      'service',
      'topic',
      'heading',
      'topic',
    ])
    const serviceRow = rows.find((row) => row.kind === 'service')
    expect(serviceRow && serviceRow.kind === 'service' ? serviceRow.alive : undefined).toBe(true)
  })
})
