//! The Kernel test harness (layer L3): a Service's real `context` and `build`, run on the in-process channel backend.
//!
//! Run every test with `#[tokio::test(start_paused = true)]`. The Kernel's [`Clock`] follows the paused tokio clock
//! from [`WALL_CLOCK_AT_START`], so `tokio::time::advance` moves the time the Domain sees, and nothing sleeps.

use core::{marker::PhantomData, time::Duration};
use std::sync::{Arc, Mutex};

use tokio::{task::JoinSet, time::Instant};

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, jobs_key, query_key, settings_key, state_key,
};
use blueos_comms::{CommsBackend, QueryBody, ReplyError, channel::ChannelBackend};
use blueos_domain::{Domain, Effect, Now};
use blueos_idl::msg::blueos_msgs::JobList;

use crate::{Clock, CommandSender, Kernel, Service, ServiceContext, ServiceError, sync};

/// The wall-clock time the Domain sees when the harness starts: 2026-01-01T00:00:00Z.
pub const WALL_CLOCK_AT_START: Duration = Duration::from_secs(1_767_225_600);

/// How long a client waits for a reply. Time is paused, so a missing reply costs no real time.
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// One applied Command's Effects in application order.
type EffectBatch<D> =
    Vec<Effect<<D as Domain>::Tick, <D as Domain>::IoRequest, <D as Domain>::TimerKey>>;

/// Shared storage for [`EffectLog`].
type EffectLogStorage<D> = Arc<Mutex<Vec<EffectBatch<D>>>>;

/// Effects the Kernel recorded for each applied Command, without running IO or timers.
pub struct EffectLog<D: Domain>(EffectLogStorage<D>);

/// A running Service `S`, with a client on the same backbone. Dropping it stops the Kernel.
pub struct Harness<S: Service> {
    backend: Arc<dyn CommsBackend>,
    command_sender: CommandSender<S::Domain>,
    durable_flush: Option<crate::durable_state::DurableWriteFlush>,
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

impl<D: Domain> EffectLog<D> {
    /// Every batch of Effects, one batch per applied Command, in order.
    pub fn batches(&self) -> Vec<EffectBatch<D>> {
        sync::lock_unpoisoned(&self.0).clone()
    }

    /// The Effects from the last applied Command, if any.
    pub fn last_batch(&self) -> Option<EffectBatch<D>> {
        sync::lock_unpoisoned(&self.0).last().cloned()
    }
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
    /// Calls `S::context` and `S::build` with `arguments` and starts its Kernel on a fresh channel backend.
    ///
    /// # Errors
    ///
    /// The [`ServiceError`] that `context`, `build` or the Kernel's startup returned.
    pub async fn start(arguments: S::Arguments) -> Result<Self, ServiceError> {
        Self::start_on(Arc::new(ChannelBackend::default()), arguments).await
    }

    /// Like [`Self::start`], and `change` replaces Context fields after `S::context` and before `S::build`, so the
    /// wiring under test is the wiring that ships: a Port wrapped or faked, or a tunable set.
    ///
    /// # Errors
    ///
    /// The [`ServiceError`] that `context`, `build` or the Kernel's startup returned.
    pub async fn start_with(
        arguments: S::Arguments,
        change: impl FnOnce(&mut S::Context),
    ) -> Result<Self, ServiceError> {
        let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
        Self::start_on_with_effect_log(
            Arc::clone(&backend),
            ServiceContext::new(arguments, backend),
            change,
            None,
        )
        .await
    }

    /// Starts the Service and records every Command's Effects without running IO or timers.
    ///
    /// # Errors
    ///
    /// The [`ServiceError`] that `context`, `build` or the Kernel's startup returned.
    pub async fn start_recording_effects(
        arguments: S::Arguments,
    ) -> Result<(Self, EffectLog<S::Domain>), ServiceError> {
        let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
        Self::start_recording_effects_with_context(ServiceContext::new(
            arguments,
            Arc::clone(&backend),
        ))
        .await
    }

    /// Like [`Self::start_recording_effects`], with a fully built [`ServiceContext`].
    ///
    /// # Errors
    ///
    /// The [`ServiceError`] that `context`, `build` or the Kernel's startup returned.
    pub async fn start_recording_effects_with_context(
        context: ServiceContext<S::Arguments>,
    ) -> Result<(Self, EffectLog<S::Domain>), ServiceError> {
        let log = EffectLog(Arc::new(Mutex::new(Vec::new())));
        let backend = Arc::clone(context.session());
        let harness = Self::start_on_with_effect_log(
            backend,
            context,
            |_context| {},
            Some(Arc::clone(&log.0)),
        )
        .await?;
        Ok((harness, log))
    }

    /// Like [`Harness::start`], on `backend`: for a test that wraps the channel backend, to make the backbone fail
    /// or to record what reaches it.
    ///
    /// # Errors
    ///
    /// The [`ServiceError`] that `context`, `build` or the Kernel's startup returned.
    pub async fn start_on(
        backend: Arc<dyn CommsBackend>,
        arguments: S::Arguments,
    ) -> Result<Self, ServiceError> {
        Self::start_on_with_context(
            Arc::clone(&backend),
            ServiceContext::new(arguments, Arc::clone(&backend)),
        )
        .await
    }

    /// Like [`Harness::start_on`], with a fully built [`ServiceContext`].
    ///
    /// # Errors
    ///
    /// The [`ServiceError`] that `context`, `build` or the Kernel's startup returned.
    pub async fn start_on_with_context(
        backend: Arc<dyn CommsBackend>,
        context: ServiceContext<S::Arguments>,
    ) -> Result<Self, ServiceError> {
        Self::start_on_with_effect_log(backend, context, |_context| {}, None).await
    }

    async fn start_on_with_effect_log(
        backend: Arc<dyn CommsBackend>,
        service: ServiceContext<S::Arguments>,
        change: impl FnOnce(&mut S::Context),
        effect_log: Option<crate::kernel::EffectLogStorage<S::Domain>>,
    ) -> Result<Self, ServiceError> {
        let mut context = S::context(&service)?;
        change(&mut context);
        let builder = S::build(&service, &context)?;
        let clock = Arc::new(PausedClock::start());
        let kernel = Kernel::start_with_effect_log(
            S::NAME,
            builder,
            context,
            Arc::clone(&backend),
            clock,
            effect_log,
        )
        .await?;
        let command_sender = kernel
            .command_sender()
            .expect("the Kernel hands out a CommandSender before it runs");
        let durable_flush = kernel.durable_write_flush();
        let mut tasks = JoinSet::new();
        tasks.spawn(async move {
            kernel.run().await;
        });
        Ok(Self {
            backend,
            command_sender,
            durable_flush,
            kernel: tasks,
            service: PhantomData,
        })
    }

    /// Waits for debounced durable state writes to reach disk.
    pub async fn flush_durable_writes(&self) {
        if let Some(flush) = &self.durable_flush {
            flush.flush().await;
        }
    }

    /// Sends Commands into the service's Inbox without using the backbone.
    pub fn command_sender(&self) -> &CommandSender<S::Domain> {
        &self.command_sender
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

    /// Sends `request` to the Query or IO query endpoint `query`, as a client would, and returns the answer, or the
    /// error reply.
    ///
    /// # Panics
    ///
    /// When the Service does not reply exactly once, or replies with something other than an `R`.
    pub async fn query<Q: Message, R: Message>(
        &self,
        query: &str,
        request: &Q,
    ) -> Result<R, ReplyError> {
        let body = QueryBody::new(
            request.encode().expect("the request encodes"),
            cdr_encoding(Q::SCHEMA_NAME),
        );
        let replies = self
            .backend
            .get(&query_key(S::NAME, query), Some(body), REPLY_TIMEOUT)
            .await
            .expect("the query key is valid");
        let [reply] = replies.as_slice() else {
            panic!("expected one reply from {query:?}, got {replies:?}");
        };
        reply
            .clone()
            .map(|sample| R::decode(&sample.payload().to_bytes()).expect("the reply is an R"))
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

    /// Reads the standard `settings` State, as a late client would.
    ///
    /// # Panics
    ///
    /// When the Service does not reply exactly once with an `M`.
    pub async fn settings<M: Message>(&self) -> M {
        let replies = self
            .backend
            .get(&settings_key(S::NAME), None, REPLY_TIMEOUT)
            .await
            .expect("the settings key is valid");
        let [Ok(reply)] = replies.as_slice() else {
            panic!("expected one settings value, got {replies:?}");
        };
        M::decode(&reply.payload().to_bytes()).expect("the reply is SettingsEnvelope")
    }

    /// Reads the standard `jobs` State, as a late client would.
    ///
    /// # Panics
    ///
    /// When the Service does not reply exactly once with a [`JobList`], as for a Domain without Jobs.
    pub async fn jobs(&self) -> JobList {
        let replies = self
            .backend
            .get(&jobs_key(S::NAME), None, REPLY_TIMEOUT)
            .await
            .expect("the jobs key is valid");
        let [Ok(reply)] = replies.as_slice() else {
            panic!("expected one jobs value, got {replies:?}");
        };
        JobList::decode(&reply.payload().to_bytes()).expect("the reply is a JobList")
    }
}

/// Locks `mutex`, or takes the guarded value if a prior holder panicked (D-29).
pub fn lock_unpoisoned<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    sync::lock_unpoisoned(mutex)
}

/// In-process backbone Session for tests that call [`Service::context`] and [`Service::build`] outside [`Harness`].
pub fn channel_session() -> crate::command_sender::Session {
    Arc::new(ChannelBackend::default())
}
