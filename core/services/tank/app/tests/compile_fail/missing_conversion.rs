//! A Domain whose `Conversions` misses the Command `Drain`.

mod common;

use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse};
use blueos_tank_api::endpoints::Conversions;
use blueos_tank_domain::{TankEvent, TankQuery, TankResponse, TankSnapshot};

use crate::common::Twin;

impl Conversions for Twin {
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
