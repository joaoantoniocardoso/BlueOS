//! Custom Recorder endpoints: path validation before the Domain sees a Command.

use blueos_idl::msg::blueos_recorder_msgs::{
    CancelRepairCommand, DeleteRecordingCommand, RepairRecordingCommand,
};
use blueos_recorder_domain::{RecorderDomain, RecorderRequest};
use blueos_recorder_paths::{RecordingRelativePath, recording_path_refusal};
use blueos_service::Refusal;

use crate::endpoints::Handlers;

/// Custom endpoint handlers for the Recorder Service.
pub(crate) struct RecorderHandlers;

impl Handlers<RecorderDomain> for RecorderHandlers {
    fn cancel_repair(&self, request: CancelRepairCommand) -> Result<RecorderRequest, Refusal> {
        let path = RecordingRelativePath::parse(&request.path)
            .map_err(|error| Refusal::from(recording_path_refusal(error)))?;
        Ok(RecorderRequest::CancelRepair { path })
    }

    fn delete_recording(
        &self,
        request: DeleteRecordingCommand,
    ) -> Result<RecorderRequest, Refusal> {
        let path = RecordingRelativePath::parse(&request.path)
            .map_err(|error| Refusal::from(recording_path_refusal(error)))?;
        Ok(RecorderRequest::DeleteRecording { path })
    }

    fn repair_recording(
        &self,
        request: RepairRecordingCommand,
    ) -> Result<RecorderRequest, Refusal> {
        let path = RecordingRelativePath::parse(&request.path)
            .map_err(|error| Refusal::from(recording_path_refusal(error)))?;
        Ok(RecorderRequest::RepairRecording { path })
    }
}
