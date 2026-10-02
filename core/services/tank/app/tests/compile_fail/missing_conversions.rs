//! A Domain without `impl Conversions` cannot register the tank's endpoints.

mod common;

use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse, SetLevelRequest};
use blueos_service::{Refusal, ServiceBuilder};
use blueos_tank_app::endpoints::{self, Handlers};
use blueos_tank_domain::{TankQuery, TankRequest, TankSnapshot};

use crate::common::Twin;

struct TwinHandlers;

impl Handlers<Twin> for TwinHandlers {
    fn set_level(&self, _request: SetLevelRequest) -> Result<TankRequest, Refusal> {
        Ok(TankRequest::Drain)
    }

    fn level_after_fill(&self, _request: SetLevelRequest) -> Result<TankQuery, Refusal> {
        Ok(TankQuery::Level)
    }

    async fn probe(&self, _request: EmptyRequest) -> Result<LevelQueryResponse, Refusal> {
        Ok(LevelQueryResponse::default())
    }
}

fn main() {
    let builder = ServiceBuilder::<Twin>::new(TankSnapshot::default());
    endpoints::register(builder, TwinHandlers);
}
