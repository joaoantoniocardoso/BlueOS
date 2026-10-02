//! Typed timers the Inbox loop polls through a [`DelayQueue`].

use core::time::Duration;
use std::collections::HashMap;

use futures_util::StreamExt;
use tokio_util::time::{DelayQueue, delay_queue::Key};

use blueos_domain::{Domain, Effect};

/// A timer wheel keyed by the Domain's [`Domain::TimerKey`], so a cancel or a re-arm always wins.
pub(crate) struct TimerWheel<D: Domain> {
    queue: DelayQueue<(D::TimerKey, D::Tick)>,
    keys: HashMap<D::TimerKey, Key>,
}

impl<D: Domain> TimerWheel<D> {
    /// An empty wheel.
    pub(crate) fn new() -> Self {
        Self {
            queue: DelayQueue::new(),
            keys: HashMap::new(),
        }
    }

    /// Applies one synchronous timer [`Effect`].
    pub(crate) fn apply(&mut self, effect: Effect<D::Tick, D::IoRequest, D::TimerKey>) {
        match effect {
            Effect::Io(_) => {}
            Effect::Schedule {
                after,
                key,
                command,
            } => self.schedule(after, key, command),
            Effect::Cancel(key) => self.cancel(key),
        }
    }

    /// Whether any timer is armed.
    pub(crate) fn waiting(&self) -> bool {
        !self.keys.is_empty()
    }

    /// Waits until the next timer expires and returns its Tick.
    pub(crate) async fn next_tick(&mut self) -> Option<D::Tick> {
        let expired = self.queue.next().await?;
        let (key, tick) = expired.into_inner();
        self.keys.remove(&key);
        Some(tick)
    }

    fn schedule(&mut self, after: Duration, key: D::TimerKey, command: D::Tick) {
        if let Some(queue_key) = self.keys.remove(&key) {
            self.queue.remove(&queue_key);
        }
        let queue_key = self.queue.insert((key.clone(), command), after);
        self.keys.insert(key, queue_key);
    }

    fn cancel(&mut self, key: D::TimerKey) {
        if let Some(queue_key) = self.keys.remove(&key) {
            self.queue.remove(&queue_key);
        }
    }
}
