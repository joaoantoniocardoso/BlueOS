import type { ServiceInfo } from '@blueos-idl/messages'

import { infoQueryKey } from './keys'
import { SERVICE_INFO_SCHEMA } from './types'
import { zenohQueryCdr } from './zenoh-helpers'

/* eslint-disable import/prefer-default-export */
export async function getServiceInfo(service: string, timeoutMs = 30_000): Promise<ServiceInfo> {
  return zenohQueryCdr(infoQueryKey(service), SERVICE_INFO_SCHEMA, { timeoutMs })
}
