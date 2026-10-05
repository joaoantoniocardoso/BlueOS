//! Inbox sender access during startup.

use tokio::sync::mpsc;

use blueos_domain::Domain;

use crate::{inbox::Delivery, service::ServiceError};

use super::super::types::Kernel;

pub(crate) fn inbox_sender<D: Domain, Context>(
    kernel: &Kernel<D, Context>,
) -> Result<&mpsc::Sender<Delivery<D>>, ServiceError> {
    kernel
        .inbox_sender
        .as_ref()
        .ok_or_else(|| ServiceError::Build("the inbox sender exists during startup".into()))
}
