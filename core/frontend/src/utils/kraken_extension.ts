const KRAKEN_API_V2_URL = '/kraken/v2.0'

export function validateIdentifier(input: string): (true | string) {
  // Identifiers should be two words separated by a period
  // They can contain lower and uppercase characters, but cannot begin with a number
  if (/\s/.test(input)) {
    return 'This field must not contain whitespace.'
  }
  const name_validator = '[A-Za-z][A-Za-z0-9]+'
  const regex = RegExp(`^${name_validator}\\.${name_validator}`)
  if (regex.test(input)) {
    return true
  }
  return 'This field must contain two words separated by a period. Numbers are allowed after the first character.'
}

export function validateImageName(input: string): (true | string) {
  // See https://github.com/distribution/reference/blob/main/reference.go
  const path_part_regex = /^[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*$/

  if (!input) return 'Name must not be empty'
  if (/\s/.test(input)) return 'Name must not contain whitespace'

  const parts = input.split('/')

  for (const [i, part] of parts.entries()) {
    if (i === 0 && parts.length >= 2) {
      // note the domain is optional; we could just have a path.
      // But if a domain is present, it has looser parsing rules.
      if (!URL.canParse(`http://${part}`)) {
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

export function parsePermissions(label: string | null | undefined): any {
  return JSON.parse(label || '{}')
}

export function extensionUrl(...segments: string[]): string {
  return `${KRAKEN_API_V2_URL}/extension/${segments.join('/')}`
}
