//! A value that is not `Handlers` cannot handle the tank's custom endpoints.

use blueos_service::ServiceBuilder;
use blueos_tank_app::endpoints;
use blueos_tank_domain::{Tank, TankSnapshot};

struct NoHandlers;

fn main() {
    let builder = ServiceBuilder::<Tank>::new(TankSnapshot::default());
    endpoints::register(builder, NoHandlers);
}
