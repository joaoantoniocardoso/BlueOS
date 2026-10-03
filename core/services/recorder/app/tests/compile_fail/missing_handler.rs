//! `Handlers` that miss the IO query `index`.

use blueos_idl::msg::blueos_recorder_msgs::{
    CancelRepairCommand, DeleteRecordingCommand, RepairRecordingCommand, SnapshotRecordingCommand,
};
use blueos_jobs::JobId;
use blueos_recorder_app::endpoints::Handlers;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest};
use blueos_service::Refusal;

struct ForgetfulHandlers;

impl Handlers<RecorderDomain> for ForgetfulHandlers {
    fn cancel_repair(&self, _request: CancelRepairCommand) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }

    fn delete_recording(
        &self,
        _request: DeleteRecordingCommand,
    ) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }

    fn repair_recording(
        &self,
        _job_id: JobId,
        _request: RepairRecordingCommand,
    ) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }

    fn snapshot_recording(
        &self,
        _request: SnapshotRecordingCommand,
    ) -> Result<RecorderRequest, Refusal> {
        Err(Refusal::from("stub"))
    }
}

fn main() {}
