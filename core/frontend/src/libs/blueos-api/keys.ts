// Mirrors `core/libs/api/src/lib.rs` (D-07, D-10, D-12).

export const API_VERSION = 'v1'
export const KEY_PREFIX = 'blueos/v1'

export const ENCODING_APPLICATION_CDR = 'application/cdr'
export const TYPE_HASH_ATTACHMENT_KEY = 'blueos.type_hash'

export function serviceLivelinessKey(service: string): string {
  return `${KEY_PREFIX}/services/${service}`
}

export function serviceInfoKey(service: string): string {
  return `${KEY_PREFIX}/services/${service}/info`
}

export function commandKey(service: string, name: string): string {
  return `${KEY_PREFIX}/${service}/command/${name}`
}

export function stateKey(service: string, name: string): string {
  return `${KEY_PREFIX}/${service}/state/${name}`
}

export function eventKey(service: string, name: string): string {
  return `${KEY_PREFIX}/${service}/event/${name}`
}

export function queryKey(service: string, name: string): string {
  return `${KEY_PREFIX}/${service}/query/${name}`
}

export function jobsKey(service: string): string {
  return `${KEY_PREFIX}/${service}/jobs`
}

export function settingsKey(service: string): string {
  return `${KEY_PREFIX}/${service}/settings`
}

export function logKey(service: string): string {
  return `${KEY_PREFIX}/${service}/log`
}

export function statusStateKey(service: string): string {
  return stateKey(service, 'status')
}

export function infoQueryKey(service: string): string {
  return queryKey(service, 'info')
}

export function cdrEncoding(schemaName: string): string {
  return `${ENCODING_APPLICATION_CDR};${schemaName}`
}
