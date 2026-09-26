/* eslint-disable import/no-extraneous-dependencies */
import { CATALOG_SCHEMAS } from '@blueos-idl/catalog'
import { describe, expect, it } from 'vitest'

import { decodeCdrWithSchema, encodeCdrWithSchema } from '@/libs/blueos-api/cdr'

describe('catalog CDR via decodeCdrWithSchema', () => {
  it('round-trips foxglove_msgs/msg/CompressedVideo', () => {
    const schemaName = 'foxglove_msgs/msg/CompressedVideo'
    const schemaText = CATALOG_SCHEMAS[schemaName]
    expect(schemaText).toBeDefined()
    const message = {
      timestamp: { sec: 1, nanosec: 2 },
      frame_id: 'camera',
      data: new Uint8Array([1, 2, 3]),
      format: 'h264',
    }
    const payload = encodeCdrWithSchema(schemaName, schemaText, message)
    expect(decodeCdrWithSchema(schemaName, schemaText, payload)).toEqual({
      timestamp: { sec: 1, nanosec: 2 },
      frame_id: 'camera',
      data: [1, 2, 3],
      format: 'h264',
    })
  })

  it('round-trips sensor_msgs/msg/Imu', () => {
    const schemaName = 'sensor_msgs/msg/Imu'
    const schemaText = CATALOG_SCHEMAS[schemaName]
    expect(schemaText).toBeDefined()
    const message = {
      header: {
        stamp: { sec: 0, nanosec: 0 },
        frame_id: 'imu',
      },
      orientation: {
        x: 0, y: 0, z: 0, w: 1,
      },
      orientation_covariance: Array.from({ length: 9 }, () => 0),
      angular_velocity: { x: 0, y: 0, z: 0 },
      angular_velocity_covariance: Array.from({ length: 9 }, () => 0),
      linear_acceleration: { x: 0, y: 0, z: 9.8 },
      linear_acceleration_covariance: Array.from({ length: 9 }, () => 0),
    }
    const payload = encodeCdrWithSchema(schemaName, schemaText, message)
    const decoded = decodeCdrWithSchema(schemaName, schemaText, payload)
    expect(decoded.header).toEqual(message.header)
    expect(decoded.orientation).toEqual(message.orientation)
    expect(decoded.linear_acceleration).toEqual(message.linear_acceleration)
  })
})
