import type { MessageBySchema } from '@blueos-idl/messages'

export type SchemaName = keyof MessageBySchema

export type MessageForSchema<Schema extends SchemaName> = MessageBySchema[Schema]

export const COMMAND_ACK_SCHEMA = 'blueos_msgs/msg/CommandAck' as const

export const LOG_SCHEMA: SchemaName = 'foxglove_msgs/msg/Log'

export const PERMISSION_ANSWER_SCHEMA = 'blueos_msgs/msg/PermissionAnswer' as const

export const SERVICE_INFO_SCHEMA = 'blueos_msgs/msg/ServiceInfo' as const
