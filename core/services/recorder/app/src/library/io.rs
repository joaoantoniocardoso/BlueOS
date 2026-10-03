//! Blocking IO for the recording library Block.

use std::sync::{Mutex, MutexGuard};

use tracing::warn;

use blueos_domain::IoError;
use blueos_jobs::JobId;
use blueos_recorder_capture::RecordingState;
use blueos_recorder_domain::{RecorderIoRequest, RecorderIoResult, RecorderSnapshot};
use blueos_recorder_library::{LibraryIoRequest, LibraryIoResult, ScannedRecording};
use blueos_recorder_paths::RecordingRelativePath;

use crate::context::RecorderContext;

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Runs library IO on a blocking thread (scan and delete).
pub(crate) fn run_library_io(
    context: &RecorderContext,
    snapshot: &RecorderSnapshot,
    request: blueos_recorder_domain::RecorderIoRequest,
) -> Result<Option<RecorderIoResult>, IoError> {
    match request {
        RecorderIoRequest::Library(io_request) => {
            Ok(Some(RecorderIoResult::Library(match io_request {
                LibraryIoRequest::Scan => scan(context, snapshot),
                LibraryIoRequest::Delete { path, job_id } => delete(context, path, job_id),
            })))
        }
        RecorderIoRequest::Cameras(_) => {
            unreachable!("cameras IO runs on the async executor")
        }
    }
}

fn active_recording_relative_path(snapshot: &RecorderSnapshot) -> Option<String> {
    match &snapshot.capture.recording {
        RecordingState::Active(active) => Some(active.file_name.clone()),
        _ => None,
    }
}

fn scan(context: &RecorderContext, snapshot: &RecorderSnapshot) -> LibraryIoResult {
    let active = active_recording_relative_path(snapshot);
    let folder = &context.recordings_folder;
    let mut footer_cache = lock_unpoisoned(&context.library_footer_cache);
    match folder.scan_library(active.as_deref(), &mut footer_cache) {
        Ok(files) => LibraryIoResult::ScanCompleted {
            recordings: files
                .into_iter()
                .map(|file| ScannedRecording {
                    relative_path: file.relative_path,
                    name: file.name,
                    size_bytes: file.size_bytes,
                    modified_unix_seconds: file.modified_unix_seconds,
                    indexed: file.indexed,
                })
                .collect(),
        },
        Err(error) => {
            warn!(%error, "Library scan failed");
            LibraryIoResult::ScanFailed
        }
    }
}

fn delete(
    context: &RecorderContext,
    path: RecordingRelativePath,
    job_id: JobId,
) -> LibraryIoResult {
    let relative = path.as_str();
    let error = context
        .recordings_folder
        .delete_recording(relative)
        .err()
        .map(|error| {
            warn!(%error, path = %relative, "Failed to delete recording");
            IoError::new(error.to_string())
        });
    LibraryIoResult::DeleteFinished {
        path,
        job_id,
        error,
    }
}
