//! A Domain whose `Conversions` reads the Command `Drain` from the wrong Message.

mod common;

use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse, SetLevelRequest};
use blueos_tank_api::endpoints::Conversions;
use blueos_tank_domain::{TankEvent, TankQuery, TankRequest, TankResponse, TankSnapshot};

use crate::common::Twin;

impl Conversions for Twin {
    fn drain(_request: SetLevelRequest) -> TankRequest {
        TankRequest::Drain
    }

    fn level(_request: EmptyRequest) -> TankQuery {
        TankQuery::Level
    }

    fn level_response(_response: TankResponse) -> Option<LevelQueryResponse> {
        None
    }

    fn level_after_fill_response(_response: TankResponse) -> Option<LevelQueryResponse> {
        None
    }

    fn tank(_snapshot: &TankSnapshot) -> LevelQueryResponse {
        LevelQueryResponse::default()
    }

    fn emptied(_event: &TankEvent) -> Option<EmptyRequest> {
        None
    }

    fn level_changed(_event: &TankEvent) -> Option<LevelQueryResponse> {
        None
    }
}

fn main() {}
