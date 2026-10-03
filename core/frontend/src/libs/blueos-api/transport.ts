/** A value on the backbone: a published sample, or the answer to a query. */
export interface Sample {
  key: string
  payload: Uint8Array
  encoding: string
}

/** One answer to a query: a Sample, or the error a queryable replied with. */
export type Reply =
  | { kind: 'sample', sample: Sample }
  | { kind: 'error', payload: Uint8Array, encoding: string }

/** The payload a query carries, such as an encoded Command, and its attachment, such as the Command's Job id. */
export interface QueryBody {
  payload: Uint8Array
  encoding: string
  attachment?: Uint8Array
}

export interface Subscription {
  close(): Promise<void>
}

/**
 * The two backbone operations blueos-api needs. `zenohTransport` implements them with zenoh-ts; tests use an
 * in-memory fake.
 */
export interface Transport {
  /** Resolves once the subscriber is declared, so every later publication on `key` reaches `onSample`. */
  subscribe(key: string, onSample: (sample: Sample) => void): Promise<Subscription>
  /** Resolves with every reply once the query on `key` is complete; no reply at all is an empty list. */
  get(key: string, body?: QueryBody): Promise<Reply[]>
  /**
   * Subscribes to service liveliness on `key`. `onAlive` is true on token put and false on delete.
   */
  subscribeLiveliness(key: string, onAlive: (alive: boolean) => void): Promise<Subscription>
}
