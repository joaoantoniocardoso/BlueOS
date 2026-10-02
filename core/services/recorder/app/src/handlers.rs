//! Custom Recorder endpoints: path validation before the Domain sees a Command.

use core::future::Future;

use blueos_idl::msg::blueos_recorder_msgs::{
    CancelRepairCommand, DeleteRecordingCommand, RecordingIndex, RecordingIndexRequest,
    RepairRecordingCommand, SnapshotRecordingCommand,
};
use blueos_recorder_domain::{RecorderDomain, RecorderRequest};
use blueos_recorder_paths::{RecordingRelativePath, recording_path_refusal};
use blueos_service::Refusal;

use crate::{context::RecorderContext, endpoints::Handlers, index_io::run_index_query};

/// Custom endpoint handlers for the Recorder Service.
pub(crate) struct RecorderHandlers {
    context: RecorderContext,
}

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

    fn snapshot_recording(
        &self,
        request: SnapshotRecordingCommand,
    ) -> Result<RecorderRequest, Refusal> {
        let path = RecordingRelativePath::parse(&request.path)
            .map_err(|error| Refusal::from(recording_path_refusal(error)))?;
        Ok(RecorderRequest::SnapshotRecording { path })
    }

    fn index(
        &self,
        request: RecordingIndexRequest,
    ) -> impl Future<Output = Result<RecordingIndex, Refusal>> + Send {
        let recorder_context = self.context.clone();
        async move { run_index_query(&recorder_context, request).await }
    }
}

impl RecorderHandlers {
    pub(crate) fn new(context: RecorderContext) -> Self {
        Self { context }
    }
}
