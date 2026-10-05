//! Service metadata, shutdown, settings, and IO executors.

use std::path::PathBuf;

use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::EndpointInfo;

use crate::{
    service::{Service, ServiceContext},
    shutdown::{ShutdownHandle, new_shutdown_channel},
};

use super::super::types::{ServiceBuilder, ServiceMetadata};

impl<D: Domain, Context> ServiceBuilder<D, Context> {
    /// Takes the version, build label and capabilities `info` publishes from `S`, and the folder the settings and the
    /// durable state live in from `service` (D-25). The Kernel's entry points call it after `build`, so `build`
    /// never passes them; a test that starts a [`Kernel`](crate::Kernel) by hand calls it too.
    pub fn for_service<S: Service<Domain = D>>(
        mut self,
        service: &ServiceContext<S::Arguments>,
    ) -> Self {
        self.metadata = ServiceMetadata {
            version: S::VERSION,
            build: S::BUILD,
            capabilities: S::CAPABILITIES,
        };
        self.settings_folder = service.settings_path().map(PathBuf::from);
        self
    }

    /// Endpoints from the Service manifest, listed in `ServiceInfo` on the `info` query. The generated `register`
    /// sets this; the Kernel adds the standard endpoints separately.
    // qual:test_helper
    pub fn manifest_endpoints(mut self, manifest_endpoints: Vec<EndpointInfo>) -> Self {
        self.manifest_endpoints = manifest_endpoints;
        self
    }

    /// Domain Command dispatched through the Inbox once startup finishes, in registration order.
    ///
    /// Use this instead of querying the service's own command keys at startup: those queryables are not served until
    /// after the initial States are published and the liveliness token is declared.
    pub fn on_start(mut self, request: D::Request) -> Self {
        self.startup_commands
            .push(blueos_domain::Command::Request(request));
        self
    }

    /// Domain Command dispatched on `SIGINT`, `SIGTERM`, or [`ShutdownHandle::trigger`].
    // qual:api
    pub fn on_shutdown(mut self, request: D::Request) -> Self {
        self.shutdown_request = Some(request);
        self
    }

    /// Handle for requesting graceful shutdown in tests (no real signals).
    pub fn shutdown_handle(&mut self) -> ShutdownHandle {
        if let Some(sender) = &self.shutdown_sender {
            return ShutdownHandle::new(sender.clone());
        }
        let (handle, receiver) = new_shutdown_channel();
        self.shutdown_sender = Some(handle.sender());
        self.shutdown_receiver = Some(receiver);
        handle
    }
}
