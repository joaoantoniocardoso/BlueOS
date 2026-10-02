/** Framework-agnostic ports of the Zenoh inspector (D-14, D-24). No Vue, DOM or network imports. */

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

/** Fields returned when parsing an rmw_zenoh data key (no transport metadata). */
export interface RmwZenohDataKeyFields {
  domainId: number
  topic: string
  typeName: string
  typeHash: string
}
