//! Message conversions for `example-minimal`: one function per manifest endpoint (D-26).

#![no_std]

pub mod endpoints;

use blueos_example_domain::{MAX_LEVEL, Pump, PumpQuery, PumpRequest, PumpResponse, PumpSnapshot};
use blueos_idl::msg::blueos_example_msgs::{
    EmptyRequest, LevelQueryResponse, PumpState, PumpStateSelfTestPhase, SetLevelRequest,
};

use crate::endpoints::Conversions;

impl Conversions for Pump {
    fn set_level(request: SetLevelRequest) -> PumpRequest {
        PumpRequest::SetLevel(request.level)
    }

    fn level(_request: EmptyRequest) -> PumpQuery {
        PumpQuery::Level
    }

    fn level_response(response: PumpResponse) -> Option<LevelQueryResponse> {
        let PumpResponse::Level(level) = response;
        Some(LevelQueryResponse {
            level,
            max_level: MAX_LEVEL,
        })
    }

    fn pump(snapshot: &PumpSnapshot) -> PumpState {
        PumpState {
            level: snapshot.level,
            max_level: MAX_LEVEL,
            self_test_phase: PumpStateSelfTestPhase::Idle,
            self_test_active: false,
        }
    }
}
