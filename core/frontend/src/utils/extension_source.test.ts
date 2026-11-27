import { expect, test } from 'bun:test'

import {
  entryCandidates,
  formatCandidate,
  isInstalledCandidate,
  isSourceSwitch,
  needsSourceChoice,
  parseSourceConflict,
} from './extension_source'

const other = { manifest: 'm2', manifest_name: 'Second', docker: 'evil/ext', name: 'Ext', tags: ['2.0.0', '1.0.0'] }
const entry = { identifier: 'org.ext', name: 'Ext', docker: 'org/ext', conflicts: [other] }

test('needsSourceChoice is true only for a non-empty conflicts list', () => {
  expect(needsSourceChoice(entry)).toBe(true)
  expect(needsSourceChoice({ ...entry, conflicts: [] })).toBe(false)
  expect(needsSourceChoice({ ...entry, conflicts: undefined })).toBe(false)
  expect(needsSourceChoice(undefined)).toBe(false)
})

test('entryCandidates lists the default repo first, then the conflicts', () => {
  const list = entryCandidates({ ...entry, manifest: 'm1', manifest_name: 'First' })
  expect(list.map((c) => c.docker)).toEqual(['org/ext', 'evil/ext'])
  expect(list[0].manifest).toBe('m1')
  expect(entryCandidates({ ...entry, conflicts: [] })).toEqual([])
})

test('parseSourceConflict reads only the 409 extension_source_conflict body', () => {
  const detail = { code: 'extension_source_conflict', identifier: 'org.ext', installed_docker: 'org/ext', candidates: [other] }
  expect(parseSourceConflict({ response: { status: 409, data: { detail } } })).toEqual(detail)
  expect(parseSourceConflict({ response: { status: 409, data: { detail: { code: 'x' } } } })).toBeUndefined()
  expect(parseSourceConflict({ response: { status: 500, data: { detail } } })).toBeUndefined()
  expect(parseSourceConflict(new Error('boom'))).toBeUndefined()
  expect(parseSourceConflict(undefined)).toBeUndefined()
})

test('installed badge and switch warning follow the installed docker', () => {
  expect(isInstalledCandidate(other, 'evil/ext')).toBe(true)
  expect(isInstalledCandidate(other, 'org/ext')).toBe(false)
  expect(isInstalledCandidate(other, undefined)).toBe(false)
  expect(isSourceSwitch(other, 'org/ext')).toBe(true)
  expect(isSourceSwitch(other, 'evil/ext')).toBe(false)
  expect(isSourceSwitch(other, undefined)).toBe(false)
})

test('formatCandidate shows repo, manifest name and tags', () => {
  expect(formatCandidate(other)).toEqual({ repo: 'evil/ext', source: 'Second', tags: '2.0.0, 1.0.0' })
  expect(formatCandidate({ docker: 'a/b', name: 'B', tags: [] })).toEqual({ repo: 'a/b', source: 'B', tags: '-' })
})

test('with the installed repo as a conflict, the default row carries the winner id and is a switch', () => {
  const winner = { ...entry, manifest: 'm1', manifest_name: 'First' }
  const rows = entryCandidates(winner)
  expect(rows[0]).toMatchObject({ manifest: 'm1', manifest_name: 'First', docker: 'org/ext' })
  expect(isInstalledCandidate(rows[1], 'evil/ext')).toBe(true)
  expect(isSourceSwitch(rows[0], 'evil/ext')).toBe(true)
  expect(formatCandidate(rows[0]).source).toBe('First')
})
