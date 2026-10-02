/* eslint-disable max-classes-per-file */
/** A sample on `key` is not CDR of the Message the endpoint declares. */
export class UnexpectedEncodingError extends Error {
  constructor(readonly key: string, readonly expected: string, readonly actual: string) {
    super(`Expected ${expected} on ${key}, got ${actual}`)
    this.name = 'UnexpectedEncodingError'
  }
}

/** The queryable on `key` replied with an error instead of a Message. */
export class QueryFailedError extends Error {
  constructor(readonly key: string, readonly reason: string) {
    super(`Query on ${key} failed: ${reason}`)
    this.name = 'QueryFailedError'
  }
}

/** Nothing answered a query on `key`: no running Service serves it. */
export class NoReplyError extends Error {
  constructor(readonly key: string) {
    super(`No reply on ${key}`)
    this.name = 'NoReplyError'
  }
}
