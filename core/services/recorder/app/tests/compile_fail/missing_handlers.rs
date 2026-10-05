//! A value that is not `Handlers` cannot handle the recorder's IO query.

use blueos_recorder_app::endpoints;
use blueos_recorder_domain::{RecorderDomain, RecorderSnapshot};
use blueos_service::ServiceBuilder;

struct NoHandlers;

fn main() {
    let builder = ServiceBuilder::<RecorderDomain>::new(RecorderSnapshot::default());
    endpoints::register(builder, NoHandlers);
}
