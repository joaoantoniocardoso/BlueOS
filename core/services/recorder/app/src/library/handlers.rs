//! Custom Recorder endpoints: path validation before the Domain sees a Command.

use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::sync::Arc;

use blueos_idl::msg::blueos_recorder_msgs::{
    DeleteRecordingCommand, RecordingIndex, RecordingIndexRequest, RepairRecordingCommand,
    SnapshotRecordingCommand,
};
use blueos_jobs::JobId;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest};
use blueos_recorder_mcap::IndexError;
use blueos_recorder_paths::{RecordingRelativePath, recording_path_refusal};
use blueos_recorder_storage::StorageError;
use blueos_service::Refusal;

use crate::{context::RecorderContext, endpoints::Handlers};

/// Default wall-clock budget for one index walk (`spawn_blocking` included).
pub(crate) const RECORDING_INDEX_WALK_TIMEOUT: Duration = Duration::from_secs(30);

/// Custom endpoint handlers for the Recorder Service.
pub(crate) struct RecorderHandlers {
    context: RecorderContext,
}

impl Handlers<RecorderDomain> for RecorderHandlers {
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
        job_id: JobId,
        request: RepairRecordingCommand,
    ) -> Result<RecorderRequest, Refusal> {
        let path = RecordingRelativePath::parse(&request.path)
            .map_err(|error| Refusal::from(recording_path_refusal(error)))?;
        Ok(RecorderRequest::RepairRecording { job_id, path })
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

/// Runs the `index` IO query outside the Inbox.
///
/// Concurrency: the Kernel serves IO queries one at a time per name (`serve_io_query`); a second
/// concurrent client waits in the queryable queue until the first walk finishes (including after a
/// timeout, once cancellation has drained the blocking task).
pub(crate) async fn run_index_query(
    context: &RecorderContext,
    request: RecordingIndexRequest,
) -> Result<RecordingIndex, Refusal> {
    let relative = RecordingRelativePath::parse(&request.path)
        .map_err(|error| Refusal::from(recording_path_refusal(error)))?;
    let path = context
        .recordings_folder
        .resolve(relative.as_str())
        .map_err(storage_refusal)?;
    let from_offset = request.from_offset;
    let limit = request.limit;
    let walk_timeout = context.index_walk_timeout;
    let walker = Arc::clone(&context.index_walker);
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_for_walk = Arc::clone(&cancel);
    let walk_path = path.clone();
    let mut walk_task = tokio::task::spawn_blocking(move || {
        (walker)(&walk_path, from_offset, limit, &cancel_for_walk)
    });
    match tokio::time::timeout(walk_timeout, &mut walk_task).await {
        Ok(join_result) => join_result
            .map_err(|error| Refusal::from(format!("Recording index walk failed: {error}")))
            .and_then(|walk| walk.map_err(index_refusal)),
        Err(_elapsed) => {
            cancel.store(true, Ordering::Relaxed);
            let join_result = walk_task
                .await
                .map_err(|error| Refusal::from(format!("Recording index walk failed: {error}")))?;
            match join_result {
                Ok(index) => Ok(index),
                Err(IndexError::Cancelled) => Err(Refusal::from("Recording index walk timed out.")),
                Err(error) => Err(index_refusal(error)),
            }
        }
    }
}

fn storage_refusal(error: StorageError) -> Refusal {
    match error {
        StorageError::InvalidPath => Refusal::from("Invalid recording path."),
        StorageError::NotFound => Refusal::from("Recording not found."),
        StorageError::Io(error) => Refusal::from(error.to_string()),
        StorageError::NameCollision => Refusal::from("Invalid recording path."),
    }
}

fn index_refusal(error: IndexError) -> Refusal {
    match error {
        IndexError::Cancelled => Refusal::from("Recording index walk timed out."),
        other => Refusal::from(other.to_string()),
    }
}
