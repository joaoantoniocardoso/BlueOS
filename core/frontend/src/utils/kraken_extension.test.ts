import { expect, test } from 'bun:test'

import {
  applyPastedConfig, isPermissionsObject, parsePermissions, permissionsBody, userPermissionsBody, validateIdentifier,
  validateImageName,
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

test('validateImageName rejects what the backend rejects', () => {
  const bad_names = [
    '', 'host/Img', 'repo:1.0', 'a//b', 'a b', 'my_host/img', 'user@host/img', '[::1]:5000/img', 'host:/img',
  ]
  for (const bad of bad_names) {
    expect(validateImageName(bad)).not.toBe(true)
  }
})

test('validateImageName accepts what the backend accepts', () => {
  for (const good of ['nginx', 'a/b', 'localhost:5000/team/img', 'Registry.Example.com/a/b.c_d__e-f']) {
    expect(validateImageName(good)).toBe(true)
  }
})

const pasted = JSON.stringify({
  identifier: 'other.ext', name: 'Other', docker: 'other/img', tag: '2.0', permissions: { a: 1 },
})
const installed = () => ({
  identifier: 'ab.cd', name: 'Mine', docker: 'mine/img', tag: '1.0', permissions: '{}',
})

test('applyPastedConfig fills every field when creating', () => {
  const extension = installed()
  const result = applyPastedConfig(extension, pasted, false)
  expect(result).toEqual({ permissions: { a: 1 } })
  expect(extension).toEqual({
    identifier: 'other.ext', name: 'Other', docker: 'other/img', tag: '2.0', permissions: '{"a":1}',
  })
})

test('applyPastedConfig keeps the locked identifier and docker when editing', () => {
  const extension = installed()
  const result = applyPastedConfig(extension, pasted, true)
  expect(result).toEqual({ permissions: { a: 1 } })
  expect(extension.identifier).toBe('ab.cd')
  expect(extension.docker).toBe('mine/img')
  expect(extension.tag).toBe('2.0')
  expect(extension.name).toBe('Other')
})

test('applyPastedConfig ignores text that is not an extension config', () => {
  const extension = installed()
  for (const text of ['hello', '{"identifier":"a.b"}', '[]', 'null']) {
    expect(applyPastedConfig(extension, text, false)).toBeNull()
  }
  expect(extension).toEqual(installed())
})
