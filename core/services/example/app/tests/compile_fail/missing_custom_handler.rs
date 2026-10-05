//! `Handlers` that miss the Goal mapping of the custom Job type `SetLevel`.

#[path = "custom/endpoints.rs"]
mod endpoints;

use blueos_example_domain::Pump;

use crate::endpoints::Handlers;

struct ForgetfulHandlers;

impl Handlers<Pump> for ForgetfulHandlers {}

fn main() {}
