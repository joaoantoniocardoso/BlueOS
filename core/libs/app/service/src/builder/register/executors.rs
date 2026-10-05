//! IO executors and settings registration.

use core::{error::Error, future::Future};
use std::sync::Arc;

use blueos_domain::{Domain, IoError};
use blueos_settings::SettingsSchema;

use crate::settings::register_settings;

use super::super::types::ServiceBuilder;

impl<D: Domain, Context> ServiceBuilder<D, Context> {
    /// The executor for every [`Effect::Io`]. It returns an optional IO result Command, or an [`IoError`] the Kernel
    /// turns into [`Domain::io_failed`].
    pub fn io<F, Fut>(mut self, executor: F) -> Self
    where
        F: Fn(&Context, &D::Snapshot, D::IoRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Option<D::IoResult>, IoError>> + Send + 'static,
    {
        self.io.r#async = Some(Arc::new(move |context, snapshot, request| {
            Box::pin(executor(context, snapshot, request))
        }));
        self
    }

    /// The executor for IO the Domain marks with [`Domain::io_runs_on_blocking_thread`]. The Kernel runs it with
    /// [`tokio::task::spawn_blocking`], with the same ordering and result reporting as [`.io`](Self::io).
    pub fn blocking_io<F>(mut self, executor: F) -> Self
    where
        F: Fn(&Context, &D::Snapshot, D::IoRequest) -> Result<Option<D::IoResult>, IoError>
            + Send
            + Sync
            + 'static,
    {
        self.io.blocking = Some(Arc::new(executor));
        self
    }

    /// Registers Python-compatible settings (D-11): the Kernel loads once at startup, owns `UpdateSettings`, and
    /// persists after each successful update.
    pub fn settings<S>(
        mut self,
        into_snapshot: impl Fn(&mut D::Snapshot, S) + Send + Sync + 'static,
        from_snapshot: impl Fn(&D::Snapshot) -> S + Send + Sync + 'static,
        into_request: impl Fn(
            blueos_idl::msg::blueos_msgs::SettingsEnvelope,
        ) -> Result<D::Request, Box<dyn Error + Send + Sync>>
        + Send
        + Sync
        + 'static,
    ) -> Self
    where
        S: SettingsSchema + Send + Sync + 'static,
    {
        self.settings = Some(register_settings(
            into_snapshot,
            from_snapshot,
            into_request,
        ));
        self
    }
}
