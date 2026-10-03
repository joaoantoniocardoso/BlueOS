//! A Domain whose `Conversions` reads the Goal of the Job type `SetLevel` from another interface type.

mod common;

use blueos_example_api::endpoints::Conversions;
use blueos_example_domain::{AboveMaximum, PumpQuery, PumpRequest, PumpResponse, PumpSnapshot};
use blueos_idl::msg::blueos_example_msgs::{
    LevelRequest, LevelResponse, PumpState, SetLevelFeedback, SetLevelResult,
};
use blueos_jobs::JobId;

use crate::common::Twin;

impl Conversions for Twin {
    type SetLevelError = AboveMaximum;

    fn set_level(_job_id: JobId, _goal: LevelRequest) -> Result<PumpRequest, AboveMaximum> {
        unimplemented!()
    }

    fn set_level_feedback(_snapshot: &PumpSnapshot, _job_id: JobId) -> Option<SetLevelFeedback> {
        None
    }

    fn set_level_result(_snapshot: &PumpSnapshot, _job_id: JobId) -> SetLevelResult {
        SetLevelResult::default()
    }

    fn level(_request: LevelRequest) -> PumpQuery {
        PumpQuery::Level
    }

    fn level_response(_response: PumpResponse) -> Option<LevelResponse> {
        None
    }

    fn pump(_snapshot: &PumpSnapshot) -> PumpState {
        PumpState::default()
    }
}

fn main() {}
