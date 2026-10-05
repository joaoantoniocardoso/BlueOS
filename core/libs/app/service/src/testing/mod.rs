//! The Kernel test harness (layer L3): a Service's real `context` and `build`, run on the in-process channel backend.
//!
//! Run every test with `#[tokio::test(start_paused = true)]`. The Kernel's [`Clock`] follows the paused tokio clock
//! from [`WALL_CLOCK_AT_START`], so `tokio::time::advance` moves the time the Domain sees, and nothing sleeps.

mod client;
mod start;

use core::{marker::PhantomData, time::Duration};
use std::sync::{Arc, Mutex};

use tokio::{task::JoinSet, time::Instant};

use blueos_comms::{CommsBackend, CommsError, channel::ChannelBackend};
use blueos_domain::{Domain, Effect, Now};
use blueos_idl::Error as IdlError;

use crate::{Clock, CommandSender, Service, ServiceContext, ServiceError, ShutdownHandle, sync};

/// The wall-clock time the Domain sees when the harness starts: 2026-01-01T00:00:00Z.
pub const WALL_CLOCK_AT_START: Duration = Duration::from_secs(1_767_225_600);

/// How long a client waits for a reply. Time is paused, so a missing reply costs no real time.
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// One applied Command's Effects in application order.
type EffectBatch<D> =
    Vec<Effect<<D as Domain>::Tick, <D as Domain>::IoRequest, <D as Domain>::TimerKey>>;

/// Shared storage for [`EffectLog`].
type EffectLogStorage<D> = Arc<Mutex<Vec<EffectBatch<D>>>>;

/// A harness client operation failed before the Service replied as expected.
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    /// The Goal or request body failed CDR encoding.
    #[error("the Message does not encode: {0}")]
    Encode(#[from] IdlError),
    /// The in-process backbone rejected the get.
    #[error("backbone get on {key}: {source}")]
    Comms {
        /// Query or command key that was queried.
        key: String,
        /// Underlying comms error.
        #[source]
        source: CommsError,
    },
    /// The Service returned zero or multiple replies.
    #[error("expected one reply on {key}, got {count}")]
    ReplyCount {
        /// Backbone key that was queried.
        key: String,
        /// Number of replies returned.
        count: usize,
    },
    /// The command endpoint returned zero or multiple acks.
    #[error("expected one ack from {command}, got {count} replies")]
    CommandAck {
        /// Command endpoint name.
        command: String,
        /// Number of replies returned.
        count: usize,
    },
    /// The payload was not the expected ROS message type.
    #[error("the reply on {key} is not {schema}: {error}")]
    Decode {
        /// Backbone key that was queried.
        key: String,
        /// Expected ROS schema name.
        schema: &'static str,
        /// CDR decode failure.
        error: IdlError,
    },
}

/// Effects the Kernel recorded for each applied Command, without running IO or timers.
pub struct EffectLog<D: Domain>(EffectLogStorage<D>);

/// A running Service `S`, with a client on the same backbone. Dropping it stops the Kernel.
pub struct Harness<S: Service> {
    backend: Arc<dyn CommsBackend>,
    command_sender: CommandSender<S::Domain>,
    durable_flush: Option<crate::durable_state::DurableWriteFlush>,
    shutdown: ShutdownHandle,
    /// Dropping the Harness aborts the Kernel.
    kernel: JoinSet<()>,
    service: PhantomData<fn() -> S>,
    /// The settings path of a Service started without one; removed after the Kernel stops.
    _settings_directory: Option<tempfile::TempDir>,
}

/// The injected [`Clock`]: it follows the paused tokio clock, so the time the Domain sees moves only with
/// `tokio::time::advance`. [`Harness`] uses it; a test that runs a [`Kernel`] directly passes it.
pub struct PausedClock {
    started: Instant,
}

impl<D: Domain> EffectLog<D> {
    /// The Effects from the last applied Command, if any.
    // qual:test_helper
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
    // qual:test_helper
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
    // qual:test_helper
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
        start::start_on_with_effect_log(backend, service, change, effect_log).await
    }

    /// Waits for debounced durable state writes to reach disk.
    // qual:test_helper
    pub async fn flush_durable_writes(&self) {
        if let Some(flush) = &self.durable_flush {
            flush.flush().await;
        }
    }

    /// Requests the graceful shutdown that `SIGTERM` would, and waits for the Kernel to finish it.
    pub async fn shutdown(mut self) {
        self.shutdown.trigger();
        while self.kernel.join_next().await.is_some() {}
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
}

/// Locks `mutex`, or takes the guarded value if a prior holder panicked (D-29).
pub fn lock_unpoisoned<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    sync::lock_unpoisoned(mutex)
}

/// In-process backbone Session for tests that call [`Service::context`] and [`Service::build`] outside [`Harness`].
// qual:test_helper
pub fn channel_session() -> crate::command_sender::Session {
    Arc::new(ChannelBackend::default())
}
