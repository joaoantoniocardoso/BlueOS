import { expect, test } from 'bun:test'

import { isIntegerString, isSocketPort } from './pattern_validators'

test('isSocketPort rejects the privileged range unless asked', () => {
  expect(isSocketPort(80)).toBe(false)
  expect(isSocketPort(80, false)).toBe(true)
  expect(isSocketPort(65536)).toBe(false)
})

test('isIntegerString accepts digits only', () => {
  expect(isIntegerString('1234')).toBe(true)
  expect(isIntegerString('12a4')).toBe(false)
})
