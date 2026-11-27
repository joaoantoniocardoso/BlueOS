import { expect, test } from 'bun:test'

import resolveFinalizeResult from './upload_finalize'

test('a clean finalize is a success', () => {
  expect(resolveFinalizeResult()).toEqual({ phase: 'success', statusText: 'Extension installed successfully' })
})

test('a thrown error is a failure', () => {
  expect(resolveFinalizeResult(new Error('boom'))).toEqual({ phase: 'error', message: 'Error: boom' })
})

test('a pull error reported by the tracker is a failure even when the request resolved', () => {
  expect(resolveFinalizeResult(undefined, 'Error: pull failed')).toEqual({ phase: 'error', message: 'Error: pull failed' })
})
