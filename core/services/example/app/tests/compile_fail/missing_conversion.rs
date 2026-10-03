//! A Domain whose `Conversions` misses the Goal conversion of the Job type `SetLevel`.

mod common;

use blueos_example_api::endpoints::Conversions;
use blueos_example_domain::{AboveMaximum, PumpQuery, PumpSnapshot};
use blueos_idl::msg::blueos_example_msgs::{
    LevelRequest, LevelResponse, PumpState, SetLevelFeedback, SetLevelResult,
};
use blueos_jobs::JobId;

use crate::common::Twin;

impl Conversions for Twin {
    type SetLevelError = AboveMaximum;

    fn set_level_feedback(_snapshot: &PumpSnapshot, _job_id: JobId) -> Option<SetLevelFeedback> {
        None
    }

    fn set_level_result(_snapshot: &PumpSnapshot, _job_id: JobId) -> SetLevelResult {
        SetLevelResult::default()
    }

    fn level(_request: LevelRequest) -> PumpQuery {
        PumpQuery::Level
    }

    fn level_response(_response: blueos_example_domain::PumpResponse) -> Option<LevelResponse> {
        None
    }

    fn pump(_snapshot: &PumpSnapshot) -> PumpState {
        PumpState::default()
    }
}

fn main() {}
