//! Message conversions for `example-minimal`: one function per manifest endpoint.

#![no_std]

pub mod endpoints;

use blueos_example_domain::{
    AboveMaximum, Level, MAX_LEVEL, Pump, PumpQuery, PumpRequest, PumpResponse, PumpSnapshot,
};
use blueos_idl::msg::blueos_example_msgs::{
    LevelRequest, LevelResponse, PumpState, PumpStateSelfTestPhase, SetLevelFeedback, SetLevelGoal,
    SetLevelResult,
};
use blueos_jobs::JobId;

use crate::endpoints::Conversions;

impl Conversions for Pump {
    type SetLevelError = AboveMaximum;

    fn set_level(job_id: JobId, goal: SetLevelGoal) -> Result<PumpRequest, AboveMaximum> {
        Level::new(goal.level).map(|level| PumpRequest::SetLevel { job_id, level })
    }

    fn set_level_feedback(snapshot: &PumpSnapshot, job_id: JobId) -> Option<SetLevelFeedback> {
        snapshot
            .filling
            .filter(|filling| filling.job_id == job_id)
            .map(|_filling| SetLevelFeedback {
                level: snapshot.level,
            })
    }

    fn set_level_result(snapshot: &PumpSnapshot, _job_id: JobId) -> SetLevelResult {
        SetLevelResult {
            level: snapshot.level,
        }
    }

    fn level(_request: LevelRequest) -> PumpQuery {
        PumpQuery::Level
    }

    fn level_response(response: PumpResponse) -> Option<LevelResponse> {
        let PumpResponse::Level(level) = response;
        Some(LevelResponse {
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
