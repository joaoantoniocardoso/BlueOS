//! `Handlers` that miss the Goal mapping of the custom Job type `RepairRecording`.

use core::future::Future;

use blueos_idl::msg::blueos_recorder_msgs::{
    DeleteRecordingGoal, RecordingIndexRequest, RecordingIndexResponse, SnapshotRecordingGoal,
};
use blueos_recorder_app::endpoints::Handlers;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest};
use blueos_service::Refusal;

struct ForgetfulHandlers;

impl Handlers<RecorderDomain> for ForgetfulHandlers {
    fn delete_recording(&self, _goal: DeleteRecordingGoal) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }

    fn snapshot_recording(&self, _goal: SnapshotRecordingGoal) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }

    fn index(
        &self,
        _request: RecordingIndexRequest,
    ) -> impl Future<Output = Result<RecordingIndexResponse, Refusal>> + Send {
        async { Err(Refusal::from("stub")) }
    }
}

fn main() {}
