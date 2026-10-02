//! The tank Service: its Domain, name and version, and its endpoints, all registered by the generated `register`.

use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError};
use blueos_tank_domain::{Tank, TankSnapshot};

use crate::{cli::TankArguments, endpoints, handlers::TankHandlers};

/// The tank Service.
pub struct TankService;

impl Service for TankService {
    type Domain = Tank;
    type Context = ();
    type Arguments = TankArguments;

    const NAME: &'static str = endpoints::NAME;
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn build(
        context: &ServiceContext<TankArguments>,
    ) -> Result<ServiceBuilder<Tank>, ServiceError> {
        let handlers = TankHandlers {
            sensor_level: context.arguments().sensor_level,
        };
        Ok(endpoints::register(
            ServiceBuilder::new(TankSnapshot::default()),
            handlers,
        ))
    }
}
