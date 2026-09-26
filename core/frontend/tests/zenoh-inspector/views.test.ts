import { describe, expect, it } from 'vitest'

import type { TopicInfo } from '@/libs/zenoh-inspector/logic/types'
import { defaultView, defaultViewRegistry, videoView } from '@/libs/zenoh-inspector/logic/views'

describe('view registry', () => {
  it('prefers video for compressed video topics', () => {
    const topic: TopicInfo = {
      key: 'video/front',
      source: 'raw',
      sampleCount: 0,
      schemaName: 'foxglove_msgs/msg/CompressedVideo',
    }
    expect(defaultView(topic, defaultViewRegistry).id).toBe(videoView.id)
  })

  it('falls back to json for generic topics', () => {
    const topic: TopicInfo = {
      key: 'blueos/v1/recorder/state/status',
      source: 'blueos',
      sampleCount: 0,
    }
    expect(defaultView(topic, defaultViewRegistry).id).toBe('json')
  })
})
