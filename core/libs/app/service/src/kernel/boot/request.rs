//! Inputs collected before the Kernel wires endpoints.

use std::sync::Arc;

use blueos_comms::CommsBackend;
use blueos_domain::Domain;

use crate::{builder::ServiceBuilder, clock::Clock};

#[cfg(feature = "testing")]
use super::super::types::EffectLogStorage;

/// Everything needed to start a [`super::super::types::Kernel`] from a finished `build`.
pub(crate) struct KernelBootRequest<D: Domain, Context> {
    pub service: &'static str,
    pub builder: ServiceBuilder<D, Context>,
    pub context: Context,
    pub backend: Arc<dyn CommsBackend>,
    pub clock: Arc<dyn Clock>,
    #[cfg(feature = "testing")]
    pub effect_log: Option<EffectLogStorage<D>>,
}
