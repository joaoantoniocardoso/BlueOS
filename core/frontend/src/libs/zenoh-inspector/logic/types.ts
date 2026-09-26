/** Framework-agnostic ports of the Zenoh inspector (D-14, D-24). No Vue, DOM or network imports. */

export type TopicSource = 'blueos' | 'rmw_zenoh' | 'ros2dds' | 'raw'

export type Ros2Transport = 'rmw_zenoh' | 'ros2dds'

export type Ros2EntityKind =
  | 'node'
  | 'publisher'
  | 'subscriber'
  | 'service_server'
  | 'service_client'
  | 'action_server'
  | 'action_client'

/** What the key or liveliness token of a ROS 2 entity says about it. */
export interface Ros2Info {
  transport: Ros2Transport
  entityKind: Ros2EntityKind
  /** Fully qualified ROS name, with the leading slash (`/robot1/chatter`). */
  topic: string
  /** ROS form, `pkg/msg/Name` or `pkg/srv/Name`. */
  typeName: string
  /** `RIHS01_<hex>`, rmw_zenoh only. */
  typeHash?: string
  node?: string
  namespace?: string
  domainId?: number
}

export type BlueosKeyKind =
  | 'command'
  | 'query'
  | 'state'
  | 'event'
  | 'jobs'
  | 'settings'
  | 'log'
  | 'http'
  | 'service_liveliness'
  | 'other'

/** A key under `blueos/v1/` split per D-07, D-10 and D-12. */
export interface BlueosKeyInfo {
  service: string
  kind: BlueosKeyKind
  /** Remainder after the kind (`library` in `blueos/v1/recorder/state/library`); empty when none. */
  name: string
}

export interface SampleRecord {
  key: string
  payload: Uint8Array
  /** Full Zenoh encoding string, including any `;<schema>` suffix. */
  encoding: string
  /** Milliseconds since the epoch, from an injected clock. */
  receivedAt: number
  kind: 'put' | 'delete'
}

export interface TopicInfo {
  key: string
  source: TopicSource
  /** From liveliness; undefined when no token was seen. */
  alive?: boolean
  /** Encoding without the schema suffix (`application/cdr`). */
  encoding?: string
  /** Resolved `pkg/msg/Name`, from the resolution chain. */
  schemaName?: string
  blueos?: BlueosKeyInfo
  ros2?: Ros2Info
  lastSample?: SampleRecord
  sampleCount: number
}

export type DecodedPayload =
  | { kind: 'text', value: string }
  | { kind: 'json', value: unknown }
  | { kind: 'cdr', schemaName: string, value: unknown }
  /** No schema or unknown encoding: size and a hex preview of the first bytes. */
  | { kind: 'binary', size: number, preview: string }
  | { kind: 'error', message: string, size: number, preview: string }

/** Schema text lookup (BlueOS IDL, then the ROS 2 and Foxglove catalog). */
export interface SchemaProvider {
  schemaText(schemaName: string): string | undefined
}

/** A view that can render a topic; the highest-priority supported view is the default. */
export interface ViewDescriptor {
  id: string
  label: string
  supports(topic: TopicInfo): boolean
  priority: number
}
