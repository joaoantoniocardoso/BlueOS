//! The tank's custom endpoints: the ones whose mapping is not a plain conversion.

use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse, SetLevelRequest};
use blueos_service::Refusal;
use blueos_tank_domain::{Percent, Tank, TankQuery, TankRequest};

use crate::endpoints::Handlers;

/// The tank's custom endpoints, with the sensor its IO query reads.
pub struct TankHandlers {
    /// The level the sensor reads.
    pub sensor_level: u8,
}

impl Handlers<Tank> for TankHandlers {
    fn set_level(&self, request: SetLevelRequest) -> Result<TankRequest, Refusal> {
        Ok(TankRequest::SetLevel(Percent::new(request.level)?))
    }

    fn level_after_fill(&self, request: SetLevelRequest) -> Result<TankQuery, Refusal> {
        Ok(TankQuery::LevelAfterFill(Percent::new(request.level)?))
    }

    async fn probe(&self, _request: EmptyRequest) -> Result<LevelQueryResponse, Refusal> {
        Ok(LevelQueryResponse {
            level: Percent::new(self.sensor_level)?.get(),
            max_level: 100,
        })
    }
}
