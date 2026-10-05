//! Opens durable state and applies any on-disk restore before endpoints are declared.

use std::sync::Arc;

use tokio::sync::Mutex;

use blueos_domain::{Command, Domain};

use crate::{
    builder::{InboxCommand, ServiceBuilder},
    clock::Clock,
    durable_state::DurableStateRegistration,
    service::ServiceError,
};

use super::super::super::types::Kernel;

pub(super) struct DurableBootPrepared<D: Domain> {
    pub startup_commands: Vec<InboxCommand<D>>,
    pub shutdown_request: Mutex<Option<D::Request>>,
    pub durable_registration: Option<DurableStateRegistration<D>>,
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    pub(super) fn prepare_durable_state_at_boot(
        service: &'static str,
        clock: Arc<dyn Clock>,
        mut builder: ServiceBuilder<D, Context>,
    ) -> Result<(ServiceBuilder<D, Context>, DurableBootPrepared<D>), ServiceError> {
        let mut startup_commands = core::mem::take(&mut builder.startup_commands);
        let shutdown_request = Mutex::new(builder.shutdown_request.take());
        let durable_registration = builder.durable.take().map(|declaration| {
            (declaration.open)(
                service.to_owned(),
                builder.settings_folder.clone(),
                declaration.version,
            )
        });
        if let Some(registration) = &durable_registration {
            registration
                .store
                .ensure_directory()
                .map_err(ServiceError::Settings)?;
            if (registration.restore_from_disk)(&mut builder.snapshot, clock.as_ref()) {
                startup_commands.push(Command::Tick(registration.restored_tick.clone()));
            }
        }
        Ok((
            builder,
            DurableBootPrepared {
                startup_commands,
                shutdown_request,
                durable_registration,
            },
        ))
    }
}
