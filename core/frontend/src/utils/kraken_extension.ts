const KRAKEN_API_V2_URL = '/kraken/v2.0'

export function validateIdentifier(input: string): (true | string) {
  // Identifiers should be two words separated by a period
  // They can contain lower and uppercase characters, but cannot begin with a number
  if (/\s/.test(input)) {
    return 'This field must not contain whitespace.'
  }
  const name_validator = '[A-Za-z][A-Za-z0-9]+'
  const regex = RegExp(`^${name_validator}\\.${name_validator}$`)
  if (regex.test(input)) {
    return true
  }
  return 'This field must contain two words separated by a period. Numbers are allowed after the first character.'
}

export function validateImageName(input: string): (true | string) {
  // See https://github.com/distribution/reference/blob/main/reference.go
  const path_part_regex = /^[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*$/
  // Same grammar as the backend check, so a valid UI input is never rejected there
  const domain_regex = /^[A-Za-z0-9]([A-Za-z0-9.-]*[A-Za-z0-9])?(:[0-9]+)?$/

  if (!input) return 'Name must not be empty'
  if (/\s/.test(input)) return 'Name must not contain whitespace'

  const parts = input.split('/')

  for (const [i, part] of parts.entries()) {
    if (i === 0 && parts.length >= 2) {
      // note the domain is optional; we could just have a path.
      // But if a domain is present, it has looser parsing rules.
      if (!domain_regex.test(part)) {
        return 'Name has an invalid domain'
      }
    } else if (i === parts.length - 1 && part.includes(':')) {
      return 'Name must not contain a tag'
    } else if (!path_part_regex.test(part)) {
      return 'Name is invalid'
    }
  }

  return true
}

export function isPermissionsObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

// The catalog has labels that are null or invalid JSON. The backend only accepts a JSON object.
export function parsePermissions(label: string | null | undefined): Record<string, unknown> {
  try {
    const parsed = JSON.parse(label || '{}')
    return isPermissionsObject(parsed) ? parsed : {}
  } catch {
    return {}
  }
}

export function permissionsBody(label: string | null | undefined): string {
  return JSON.stringify(parsePermissions(label))
}

export function userPermissionsBody(label: string | null | undefined): string {
  const parsed = parsePermissions(label)
  return Object.keys(parsed).length > 0 ? JSON.stringify(parsed) : ''
}

export function extensionUrl(...segments: string[]): string {
  return `${KRAKEN_API_V2_URL}/extension/${segments.map(encodeURIComponent).join('/')}`
}

export interface PastedExtensionFields {
  identifier: string
  name: string
  docker: string
  tag: string
  permissions?: string
}

// Copies the fields of a pasted extension config into `extension`.
// Returns the parsed permissions to show in the editor, or null when the text is not an extension config.
export function applyPastedConfig(
  extension: PastedExtensionFields,
  text: string,
): { permissions: unknown } | null {
  try {
    const config = JSON.parse(text)
    if (!(config.identifier && config.name && config.docker && config.tag)) {
      return null
    }
    extension.identifier = config.identifier
    extension.name = config.name
    extension.docker = config.docker
    extension.tag = config.tag

    if (!config.permissions) {
      return { permissions: undefined }
    }
    extension.permissions = typeof config.permissions === 'string'
      ? config.permissions
      : JSON.stringify(config.permissions)
    return { permissions: JSON.parse(extension.permissions) }
  } catch {
    return null
  }
}
