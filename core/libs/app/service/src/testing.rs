//! The Kernel test harness (layer L3): a Service's real `build`, run on the in-process channel backend.
//!
//! Run every test with `#[tokio::test(start_paused = true)]`. The Kernel's [`Clock`] follows the paused tokio clock
//! from [`WALL_CLOCK_AT_START`], so `tokio::time::advance` moves the time the Domain sees, and nothing sleeps.

use core::{marker::PhantomData, time::Duration};
use std::sync::Arc;

use tokio::{task::JoinSet, time::Instant};

use blueos_api::{CommandAck, Message, cdr_encoding, command_key, state_key};
use blueos_comms::{CommsBackend, QueryBody, channel::ChannelBackend};
use blueos_domain::Now;

use crate::{Clock, Kernel, Service, ServiceContext, ServiceError};

/// The wall-clock time the Domain sees when the harness starts: 2026-01-01T00:00:00Z.
pub const WALL_CLOCK_AT_START: Duration = Duration::from_secs(1_767_225_600);

/// How long a client waits for a reply. Time is paused, so a missing reply costs no real time.
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// A running Service `S`, with a client on the same backbone. Dropping it stops the Kernel.
pub struct Harness<S: Service> {
    backend: Arc<dyn CommsBackend>,
    #[expect(
        dead_code,
        reason = "held so that dropping the Harness aborts the Kernel"
    )]
    kernel: JoinSet<()>,
    service: PhantomData<fn() -> S>,
}

/// The injected [`Clock`]: it follows the paused tokio clock, so the time the Domain sees moves only with
/// `tokio::time::advance`. [`Harness`] uses it; a test that runs a [`Kernel`] directly passes it.
pub struct PausedClock {
    started: Instant,
}

impl Clock for PausedClock {
    fn now(&self) -> Now {
        let monotonic = self.started.elapsed();
        Now {
            wall: WALL_CLOCK_AT_START + monotonic,
            monotonic,
        }
    }
}

impl PausedClock {
    /// A clock at [`WALL_CLOCK_AT_START`], with a monotonic time of zero now.
    pub fn start() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl<S: Service> Harness<S> {
    /// Calls `S::build` with `arguments` and starts its Kernel on a fresh channel backend.
    ///
    /// # Errors
    ///
    /// The [`ServiceError`] that `build` or the Kernel's startup returned.
    pub async fn start(arguments: S::Arguments) -> Result<Self, ServiceError> {
        Self::start_on(Arc::new(ChannelBackend::default()), arguments).await
    }

    /// Like [`Harness::start`], on `backend`: for a test that wraps the channel backend, to make the backbone fail
    /// or to record what reaches it.
    ///
    /// # Errors
    ///
    /// The [`ServiceError`] that `build` or the Kernel's startup returned.
    pub async fn start_on(
        backend: Arc<dyn CommsBackend>,
        arguments: S::Arguments,
    ) -> Result<Self, ServiceError> {
        let builder = S::build(&ServiceContext::new(arguments))?;
        let clock = Arc::new(PausedClock::start());
        let kernel = Kernel::start(S::NAME, builder, Arc::clone(&backend), clock).await?;
        let mut tasks = JoinSet::new();
        tasks.spawn(kernel.run());
        Ok(Self {
            backend,
            kernel: tasks,
            service: PhantomData,
        })
    }

    /// The backbone the Service runs on, for anything the harness has no helper for, such as subscribing to an
    /// Event.
    pub fn backend(&self) -> &Arc<dyn CommsBackend> {
        &self.backend
    }

    /// Sends `request` to the Command endpoint `command`, as a client would, and returns the ack.
    ///
    /// # Panics
    ///
    /// When the Service does not reply exactly once with a [`CommandAck`].
    pub async fn send<M: Message>(&self, command: &str, request: &M) -> CommandAck {
        let body = QueryBody::new(
            request.encode().expect("the request encodes"),
            cdr_encoding(M::SCHEMA_NAME),
        );
        let replies = self
            .backend
            .get(&command_key(S::NAME, command), Some(body), REPLY_TIMEOUT)
            .await
            .expect("the command key is valid");
        let [Ok(reply)] = replies.as_slice() else {
            panic!("expected one ack from {command:?}, got {replies:?}");
        };
        CommandAck::decode(&reply.payload().to_bytes()).expect("the reply is a CommandAck")
    }

    /// Reads the State `state`, as a late client would.
    ///
    /// # Panics
    ///
    /// When the Service does not reply exactly once with an `M`.
    pub async fn state<M: Message>(&self, state: &str) -> M {
        let replies = self
            .backend
            .get(&state_key(S::NAME, state), None, REPLY_TIMEOUT)
            .await
            .expect("the state key is valid");
        let [Ok(reply)] = replies.as_slice() else {
            panic!("expected one value of {state:?}, got {replies:?}");
        };
        M::decode(&reply.payload().to_bytes()).expect("the reply is the State's Message")
    }
}
