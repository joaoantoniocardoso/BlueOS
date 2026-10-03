//! `Handlers` that miss the IO query `index`.

use blueos_idl::msg::blueos_recorder_msgs::{
    DeleteRecordingGoal, RepairRecordingGoal, SnapshotRecordingGoal,
};
use blueos_jobs::JobId;
use blueos_recorder_app::endpoints::Handlers;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest};
use blueos_service::Refusal;

struct ForgetfulHandlers;

impl Handlers<RecorderDomain> for ForgetfulHandlers {
    fn delete_recording(
        &self,
        _goal: DeleteRecordingGoal,
    ) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }

    fn repair_recording(
        &self,
        _job_id: JobId,
        _goal: RepairRecordingGoal,
    ) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }

    fn snapshot_recording(
        &self,
        _goal: SnapshotRecordingGoal,
    ) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }
}

fn main() {}
