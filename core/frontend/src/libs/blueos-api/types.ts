import { SCHEMAS } from '@blueos-idl/schemas'

export type SchemaName = keyof typeof SCHEMAS

export type MessageForSchema<Schema extends SchemaName> = Record<string, unknown>

export const COMMAND_ACK_SCHEMA: SchemaName = 'blueos_msgs/msg/CommandAck'
