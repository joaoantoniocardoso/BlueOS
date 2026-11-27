import { expect, test } from 'bun:test'

import { validateIdentifier } from './kraken_extension'

test('validateIdentifier rejects characters that break the extension URL', () => {
  for (const bad of ['ab.cd/evil', 'ab.cd?x', 'ab.cd#f', 'ab.cd..', 'ab.cd.ef', 'ab.cd%2F', 'ab cd.ef', 'ab.', '.cd']) {
    expect(validateIdentifier(bad)).not.toBe(true)
  }
})

test('validateIdentifier accepts two words', () => {
  for (const good of ['ab.cd', 'Blue.Robotics1', 'a1.b2']) {
    expect(validateIdentifier(good)).toBe(true)
  }
})
