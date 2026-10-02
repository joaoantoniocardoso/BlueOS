//! A Domain whose `Conversions` reads the Command `SetLevel` from the wrong Message.

mod common;

use blueos_example_api::endpoints::Conversions;
use blueos_example_domain::{PumpQuery, PumpRequest, PumpResponse, PumpSnapshot};
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse, PumpState};

use crate::common::Twin;

impl Conversions for Twin {
    fn set_level(_request: EmptyRequest) -> PumpRequest {
        PumpRequest::SetLevel(0)
    }

    fn level(_request: EmptyRequest) -> PumpQuery {
        PumpQuery::Level
    }

    fn level_response(_response: PumpResponse) -> Option<LevelQueryResponse> {
        None
    }

    fn pump(_snapshot: &PumpSnapshot) -> PumpState {
        PumpState::default()
    }
}

fn main() {}
