//! The conversions between the tank's Messages and its Domain: one function per endpoint, in the generated
//! [`endpoints::Conversions`] trait (D-26).

#![no_std]

pub mod endpoints;

use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse};
use blueos_tank_domain::{
    Percent, Tank, TankEvent, TankQuery, TankRequest, TankResponse, TankSnapshot,
};

use crate::endpoints::Conversions;

impl Conversions for Tank {
    fn drain(_request: EmptyRequest) -> TankRequest {
        TankRequest::Drain
    }

    fn level(_request: EmptyRequest) -> TankQuery {
        TankQuery::Level
    }

    fn level_response(response: TankResponse) -> Option<LevelQueryResponse> {
        let TankResponse::Level(level) = response;
        Some(level_message(level))
    }

    fn level_after_fill_response(response: TankResponse) -> Option<LevelQueryResponse> {
        Self::level_response(response)
    }

    fn tank(snapshot: &TankSnapshot) -> LevelQueryResponse {
        level_message(snapshot.level)
    }

    fn emptied(event: &TankEvent) -> Option<EmptyRequest> {
        matches!(event, TankEvent::Emptied).then(EmptyRequest::default)
    }

    fn level_changed(event: &TankEvent) -> Option<LevelQueryResponse> {
        match *event {
            TankEvent::LevelChanged(level) => Some(level_message(level)),
            TankEvent::Emptied => None,
        }
    }
}

fn level_message(level: Percent) -> LevelQueryResponse {
    LevelQueryResponse {
        level: level.get(),
        max_level: 100,
    }
}
