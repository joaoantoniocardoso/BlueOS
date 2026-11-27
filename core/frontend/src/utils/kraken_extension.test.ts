import { expect, test } from 'bun:test'

import {
  isPermissionsObject, parsePermissions, permissionsBody, userPermissionsBody, validateIdentifier,
} from './kraken_extension'

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

test('parsePermissions returns an object for every label', () => {
  expect(parsePermissions('{"a":1}')).toEqual({ a: 1 })
  for (const bad of ['not json', '{bad', '[]', 'null', '5', '"x"', '', null, undefined]) {
    expect(parsePermissions(bad)).toEqual({})
  }
})

test('isPermissionsObject accepts plain objects only', () => {
  expect(isPermissionsObject({})).toBe(true)
  for (const bad of [[], null, 5, 'x', undefined]) {
    expect(isPermissionsObject(bad)).toBe(false)
  }
})

test('permissionsBody always gives a JSON object string the backend accepts', () => {
  expect(permissionsBody('{"a":1}')).toBe('{"a":1}')
  for (const bad of ['null', '', 'not json', '[]', null, undefined]) {
    expect(permissionsBody(bad)).toBe('{}')
  }
})

test('userPermissionsBody is empty or a JSON object string', () => {
  expect(userPermissionsBody('{"a":1}')).toBe('{"a":1}')
  for (const empty of ['', 'null', 'not json', '[]', null, undefined]) {
    expect(userPermissionsBody(empty)).toBe('')
  }
})
