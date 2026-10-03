//! The `example-minimal` Service: an empty Context, and Domain wiring through the generated
//! [`endpoints::register`].

use blueos_example_domain::{Pump, PumpSnapshot};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError};

use crate::{cli::ExampleArguments, endpoints};

/// The teaching example Service.
pub struct ExampleService;

impl Service for ExampleService {
    type Domain = Pump;
    type Context = ();
    type Arguments = ExampleArguments;

    const NAME: &'static str = endpoints::NAME;
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn context(_service: &ServiceContext<ExampleArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<ExampleArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Pump>, ServiceError> {
        Ok(
            endpoints::register(ServiceBuilder::new(PumpSnapshot::default())).service_metadata(
                Self::VERSION,
                Self::BUILD,
                Self::CAPABILITIES,
            ),
        )
    }
}
