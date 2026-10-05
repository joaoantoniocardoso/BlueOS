//! The Recorder's `bytes` and `index` IO queries.

use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::sync::Arc;

use blueos_idl::msg::blueos_recorder_msgs::{
    RecordingBytesRequest, RecordingBytesResponse, RecordingIndexRequest, RecordingIndexResponse,
};
use blueos_recorder_domain::RecorderDomain;
use blueos_recorder_mcap::IndexError;
use blueos_recorder_paths::RecordingRelativePath;
use blueos_recorder_storage::{RangeStart, StorageError, read_range};
use blueos_service::Refusal;

use crate::{context::RecorderContext, endpoints::Handlers};

/// Default wall-clock budget for one index walk (`spawn_blocking` included).
pub(crate) const RECORDING_INDEX_WALK_TIMEOUT: Duration = Duration::from_secs(30);

/// The most bytes one `bytes` reply carries, the Recorder's MCAP chunk size. Zenoh fragments it on the way, but
/// the browser receives it as one websocket message shared with every State and Event of the page: at 10 Mbit/s a
/// 1 MiB reply holds that websocket for under a second.
const RECORDING_BYTES_MAX_LENGTH: u32 = 1024 * 1024;

/// Wall-clock budget for one `bytes` read (`spawn_blocking` included).
const RECORDING_BYTES_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// The handlers of the Recorder's IO queries.
pub(crate) struct RecorderHandlers {
    context: RecorderContext,
}

impl Handlers<RecorderDomain> for RecorderHandlers {
    fn bytes(
        &self,
        request: RecordingBytesRequest,
    ) -> impl Future<Output = Result<RecordingBytesResponse, Refusal>> + Send {
        let recorder_context = self.context.clone();
        async move { run_bytes_query(&recorder_context, request).await }
    }

    fn index(
        &self,
        request: RecordingIndexRequest,
    ) -> impl Future<Output = Result<RecordingIndexResponse, Refusal>> + Send {
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
) -> Result<RecordingIndexResponse, Refusal> {
    let relative = RecordingRelativePath::parse(&request.path).map_err(Refusal::from)?;
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
    let mut walk_task =
        tokio::task::spawn_blocking(move || (walker)(&path, from_offset, limit, &cancel_for_walk));
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

/// Runs the `bytes` IO query outside the Inbox, one read at a time like `index`.
async fn run_bytes_query(
    context: &RecorderContext,
    request: RecordingBytesRequest,
) -> Result<RecordingBytesResponse, Refusal> {
    let relative = RecordingRelativePath::parse(&request.path).map_err(Refusal::from)?;
    let path = context
        .recordings_folder
        .resolve(relative.as_str())
        .map_err(storage_refusal)?;
    let length = u64::from(request.length.min(RECORDING_BYTES_MAX_LENGTH));
    let start = if request.from_end {
        RangeStart::FromEnd
    } else {
        RangeStart::Offset(request.offset)
    };
    let read = tokio::task::spawn_blocking(move || read_range(&path, start, length));
    // ponytail: a read past its timeout is not cancelled, so its blocking thread may still run beside the next read.
    // Read in pieces with a cancel flag, as the index walk does, if a disk stalls that long.
    let range = tokio::time::timeout(RECORDING_BYTES_READ_TIMEOUT, read)
        .await
        .map_err(|_elapsed| Refusal::from("Recording read timed out."))?
        .map_err(|error| Refusal::from(format!("Recording read failed: {error}")))?
        .map_err(Refusal::from)?;
    Ok(RecordingBytesResponse {
        size: range.size,
        data: range.data,
    })
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
