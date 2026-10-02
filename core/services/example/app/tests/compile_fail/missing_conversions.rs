//! A Domain without `impl Conversions` cannot register the example endpoints.

mod common;

use blueos_example_app::endpoints;
use blueos_example_domain::PumpSnapshot;
use blueos_service::ServiceBuilder;

use crate::common::Twin;

fn main() {
    let builder = ServiceBuilder::<Twin>::new(PumpSnapshot::default());
    endpoints::register(builder);
}
