//! The Disk Usage Service wiring.

use blueos_disk_usage_domain::DiskUsageDomain;
use blueos_service::Service;

use crate::endpoints;

/// The Disk Usage Service
pub struct DiskUsageService;

impl Service for DiskUsageService {
    type Domain = DiskUsageDomain;
    type Context = ();
    type Arguments = ();

    const NAME: &'static str = endpoints::NAME;

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn context(
        service: &blueos_service::ServiceContext<Self::Arguments>,
    ) -> Result<Self::Context, blueos_service::ServiceError> {
        todo!()
    }

    fn build(
        service: &blueos_service::ServiceContext<Self::Arguments>,
        context: &Self::Context,
    ) -> Result<
        blueos_service::ServiceBuilder<Self::Domain, Self::Context>,
        blueos_service::ServiceError,
    > {
        todo!()
    }
}
