//! A Domain whose `Conversions` misses the Command `SetLevel`.

mod common;

use blueos_example_api::endpoints::Conversions;
use blueos_example_domain::{PumpQuery, PumpSnapshot};
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse, PumpState};

use crate::common::Twin;

impl Conversions for Twin {
    fn level(_request: EmptyRequest) -> PumpQuery {
        PumpQuery::Level
    }

    fn level_response(_response: blueos_example_domain::PumpResponse) -> Option<LevelQueryResponse> {
        None
    }

    fn pump(_snapshot: &PumpSnapshot) -> PumpState {
        PumpState::default()
    }
}

fn main() {}
