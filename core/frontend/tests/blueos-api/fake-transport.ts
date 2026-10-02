import type {
  QueryBody, Reply, Sample, Subscription, Transport,
} from '@/libs/blueos-api/transport'

/** A query the test has not answered yet. */
export interface PendingQuery {
  key: string
  body?: QueryBody
  reply: (...replies: Reply[]) => void
  fail: (error: Error) => void
}

interface FakeSubscriber {
  key: string
  onSample: (sample: Sample) => void
  open: boolean
}

function matches(expression: string, key: string): boolean {
  const chunks = expression.split('/').map((chunk) => {
    if (chunk === '*') {
      return '[^/]+'
    }
    return chunk.replace(/[.+?^${}()|[\]\\]/g, '\\$&')
  })
  return new RegExp(`^${chunks.join('/')}$`).test(key)
}

/**
 * An in-memory backbone that the test drives by hand: it publishes samples when told to and answers each query only
 * when the test replies, so no test needs a timer or a sleep. Keys match exactly or with `*` chunks.
 */
export default class FakeTransport implements Transport {
  readonly subscribers: FakeSubscriber[] = []

  /** Runs once a subscriber is declared, before `subscribe` resolves. */
  afterSubscribe?: () => void

  private readonly queries: PendingQuery[] = []

  private readonly waiting: ((query: PendingQuery) => void)[] = []

  async subscribe(key: string, onSample: (sample: Sample) => void): Promise<Subscription> {
    const subscriber = { key, onSample, open: true }
    this.subscribers.push(subscriber)
    this.afterSubscribe?.()
    return {
      close: async () => {
        subscriber.open = false
      },
    }
  }

  get(key: string, body?: QueryBody): Promise<Reply[]> {
    return new Promise((resolve, reject) => {
      const query = {
        key, body, reply: (...replies: Reply[]) => resolve(replies), fail: reject,
      }
      const waiter = this.waiting.shift()
      if (waiter === undefined) {
        this.queries.push(query)
      } else {
        waiter(query)
      }
    })
  }

  /** Resolves with the next query sent, in the order they were sent. */
  nextQuery(): Promise<PendingQuery> {
    const query = this.queries.shift()
    if (query !== undefined) {
      return Promise.resolve(query)
    }
    return new Promise((resolve) => {
      this.waiting.push(resolve)
    })
  }

  /** Delivers `sample` to every open subscriber whose key matches, as the router does. */
  publish(sample: Sample): void {
    for (const subscriber of this.subscribers) {
      if (subscriber.open && matches(subscriber.key, sample.key)) {
        subscriber.onSample(sample)
      }
    }
  }
}
