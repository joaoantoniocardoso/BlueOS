//! Kernel constructors.

use std::sync::Arc;

use tokio::sync::mpsc;

use blueos_comms::CommsBackend;
use blueos_domain::Domain;

use crate::{
    builder::ServiceBuilder, clock::Clock, command_sender::CommandSender,
    logging::LogPublisherRuntime, service::ServiceError,
};

use super::{boot::request::KernelBootRequest, types::Kernel};

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    /// Declares every endpoint of `builder` on `backend` and publishes the initial States, so every endpoint answers
    /// once this returns. The Kernel owns `context` from here on and hands it to IO code and Tasks.
    ///
    /// # Errors
    ///
    /// [`ServiceError::DeclareEndpoint`] when the backbone refuses an endpoint.
    pub async fn start(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        context: Context,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ServiceError> {
        Self::boot(KernelBootRequest {
            service,
            builder,
            context,
            backend,
            clock,
            #[cfg(feature = "testing")]
            effect_log: None,
        })
        .await
    }

    async fn boot(request: KernelBootRequest<D, Context>) -> Result<Self, ServiceError> {
        super::boot::run::boot(request).await
    }

    /// Like [`Self::start`], optionally recording Effects without running IO or timers (harness only).
    #[cfg(feature = "testing")]
    // qual:allow(srp, max_parameters=6) reason: "Public harness boot API matches the monolithic Kernel signature"
    pub async fn start_with_effect_log(
        service: &'static str,
        builder: ServiceBuilder<D, Context>,
        context: Context,
        backend: Arc<dyn CommsBackend>,
        clock: Arc<dyn Clock>,
        effect_log: Option<super::EffectLogStorage<D>>,
    ) -> Result<Self, ServiceError> {
        Self::boot(KernelBootRequest {
            service,
            builder,
            context,
            backend,
            clock,
            effect_log,
        })
        .await
    }

    /// Runs the service log publisher flush as the last shutdown step (D-04, D-13).
    pub fn attach_log_publisher(&mut self, runtime: LogPublisherRuntime) {
        self.log_publisher = Some(runtime);
    }

    /// Handle for waiting on debounced durable writes in tests.
    #[cfg(feature = "testing")]
    pub fn durable_write_flush(&self) -> Option<crate::durable_state::DurableWriteFlush> {
        self.durable
            .as_ref()
            .map(|durable| durable.persister.flush_handle())
    }

    /// Hands out a [`CommandSender`] while the Inbox is still open.
    pub fn command_sender(&self) -> Option<CommandSender<D>> {
        self.inbox_sender
            .as_ref()
            .map(|sender| CommandSender::new(mpsc::Sender::clone(sender)))
    }
}
