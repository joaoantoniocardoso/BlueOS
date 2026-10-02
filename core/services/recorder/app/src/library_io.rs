//! Blocking and async IO for the recording library Block.

use core::sync::atomic::AtomicBool;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
};

use tokio::sync::mpsc;
use tracing::warn;

use blueos_domain::IoError;
use blueos_recorder_capture::RecordingState;
use blueos_recorder_domain::{RecorderIoResult, RecorderObservedFact, RecorderSnapshot};
use blueos_recorder_library::{
    LibraryIoRequest, LibraryIoResult, LibraryRepairOutcome, RepairFailure, ScannedRecording,
};
use blueos_recorder_mcap::{RewriteError, rewrite};
use blueos_recorder_paths::RecordingRelativePath;
use blueos_recorder_storage::{RecordingsFolder, StorageError};

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
        blueos_recorder_domain::RecorderIoRequest::Library(io_request) => {
            Ok(Some(RecorderIoResult::Library(match io_request {
                LibraryIoRequest::Scan => scan(context, snapshot),
                LibraryIoRequest::Delete { path } => delete(context, &path),
                LibraryIoRequest::Repair { .. } | LibraryIoRequest::CancelRepair { .. } => {
                    return Err(IoError::new("repair IO must use the async executor"));
                }
            })))
        }
        RecorderIoRequest::Cameras(_) => {
            unreachable!("cameras IO runs on the async executor")
        }
    }
}

/// Runs repair IO without blocking the async runtime worker.
pub(crate) async fn run_library_repair_io(
    recordings_folder: Arc<RecordingsFolder>,
    progress_sender: mpsc::Sender<RecorderObservedFact>,
    cancel_flags: Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
    path: RecordingRelativePath,
) -> Result<Option<RecorderIoResult>, IoError> {
    let relative = path.as_str().to_string();
    let cancel = RecorderContext::repair_cancel_flag(&cancel_flags, &relative);
    let path_for_result = path.clone();
    let rewrite_result = tokio::task::spawn_blocking(move || {
        let source = match recordings_folder.resolve(&relative) {
            Ok(value) => value,
            Err(StorageError::NotFound) | Err(StorageError::InvalidPath) => {
                return LibraryIoResult::RepairFinished {
                    path: path_for_result,
                    outcome: LibraryRepairOutcome::Failed(RepairFailure::Io),
                };
            }
            Err(StorageError::Io(_)) | Err(StorageError::NameCollision) => {
                return LibraryIoResult::RepairFinished {
                    path: path_for_result,
                    outcome: LibraryRepairOutcome::Failed(RepairFailure::Io),
                };
            }
        };
        let temporary = recordings_folder.recover_temporary_path(&relative);
        let progress_path = path_for_result.clone();
        let rewrite_outcome = rewrite(
            &source,
            &temporary,
            &mut |bytes_read, total_bytes| {
                let _ = progress_sender.try_send(RecorderObservedFact::LibraryRepairProgress(
                    blueos_recorder_library::LibraryRepairProgress {
                        path: progress_path.clone(),
                        bytes_processed: bytes_read,
                        total_bytes,
                    },
                ));
            },
            &cancel,
        );
        match rewrite_outcome {
            Ok(_summary) => {
                match recordings_folder.replace_recording_from_temporary(&temporary, &relative) {
                    Ok(()) => LibraryIoResult::RepairFinished {
                        path: path_for_result,
                        outcome: LibraryRepairOutcome::Succeeded,
                    },
                    Err(_) => LibraryIoResult::RepairFinished {
                        path: path_for_result,
                        outcome: LibraryRepairOutcome::Failed(RepairFailure::Replace),
                    },
                }
            }
            Err(RewriteError::Cancelled) => LibraryIoResult::RepairFinished {
                path: path_for_result,
                outcome: LibraryRepairOutcome::Cancelled,
            },
            Err(RewriteError::Mcap(_)) => LibraryIoResult::RepairFinished {
                path: path_for_result,
                outcome: LibraryRepairOutcome::Failed(RepairFailure::Rewrite),
            },
            Err(RewriteError::Io(_)) => LibraryIoResult::RepairFinished {
                path: path_for_result,
                outcome: LibraryRepairOutcome::Failed(RepairFailure::Io),
            },
        }
    })
    .await
    .map_err(|error| IoError::new(error.to_string()))?;

    RecorderContext::clear_repair_cancel_flag(&cancel_flags, path.as_str());
    Ok(Some(RecorderIoResult::Library(rewrite_result)))
}

/// Handles cancel-repair IO (only sets the cancel flag; rewrite observes it).
pub(crate) fn run_cancel_repair_io(
    cancel_flags: &Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
    path: &RecordingRelativePath,
) -> Result<Option<RecorderIoResult>, IoError> {
    RecorderContext::request_repair_cancel(cancel_flags, path.as_str());
    Ok(None)
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

fn delete(context: &RecorderContext, path: &RecordingRelativePath) -> LibraryIoResult {
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
        path: path.clone(),
        error,
    }
}
