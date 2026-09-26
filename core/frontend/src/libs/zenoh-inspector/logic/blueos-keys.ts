import { KEY_PREFIX } from '@/libs/blueos-api/keys'

import type { BlueosKeyInfo, BlueosKeyKind } from './types'

const STANDARD_KINDS: BlueosKeyKind[] = ['command', 'query', 'state', 'event', 'jobs', 'settings', 'log']

export function isBlueosKey(key: string): boolean {
  return key.startsWith(`${KEY_PREFIX}/`)
}

export function parseBlueosKey(key: string): BlueosKeyInfo | undefined {
  const prefix = `${KEY_PREFIX}/`
  if (!key.startsWith(prefix)) {
    return undefined
  }

  const remainder = key.slice(prefix.length)
  if (remainder.length === 0) {
    return undefined
  }

  if (remainder.startsWith('services/')) {
    const service = remainder.slice('services/'.length)
    if (service.length === 0 || service.includes('/')) {
      return undefined
    }
    return { service, kind: 'service_liveliness', name: '' }
  }

  const firstSlash = remainder.indexOf('/')
  if (firstSlash < 0) {
    return undefined
  }

  const service = remainder.slice(0, firstSlash)
  const afterService = remainder.slice(firstSlash + 1)
  if (afterService.length === 0) {
    return undefined
  }

  const kindSlash = afterService.indexOf('/')
  const kindSegment = kindSlash < 0 ? afterService : afterService.slice(0, kindSlash)
  const name = kindSlash < 0 ? '' : afterService.slice(kindSlash + 1)

  if (kindSegment === 'http') {
    return { service, kind: 'http', name }
  }

  if (STANDARD_KINDS.includes(kindSegment as BlueosKeyKind)) {
    return { service, kind: kindSegment as BlueosKeyKind, name }
  }

  return { service, kind: 'other', name: afterService }
}
