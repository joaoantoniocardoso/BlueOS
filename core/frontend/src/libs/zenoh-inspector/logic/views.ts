import type { TopicInfo, ViewDescriptor } from './types'

const COMPRESSED_VIDEO = 'foxglove_msgs/msg/CompressedVideo'

export const jsonView: ViewDescriptor = {
  id: 'json',
  label: 'JSON',
  priority: 0,
  supports(): boolean {
    return true
  },
}

export const videoView: ViewDescriptor = {
  id: 'video',
  label: 'Video',
  priority: 100,
  supports(topic: TopicInfo): boolean {
    if (topic.key.startsWith('video/')) {
      return true
    }
    if (topic.schemaName === COMPRESSED_VIDEO) {
      return true
    }
    if (topic.ros2?.typeName === COMPRESSED_VIDEO) {
      return true
    }
    return false
  },
}

export const defaultViewRegistry: ViewDescriptor[] = [jsonView, videoView]

export function availableViews(topic: TopicInfo, registry: ViewDescriptor[]): ViewDescriptor[] {
  return registry.filter((descriptor) => descriptor.supports(topic))
}

export function defaultView(topic: TopicInfo, registry: ViewDescriptor[]): ViewDescriptor {
  const supported = availableViews(topic, registry)
  if (supported.length === 0) {
    return jsonView
  }
  return supported.reduce((best, candidate) => {
    if (candidate.priority > best.priority) {
      return candidate
    }
    return best
  })
}
