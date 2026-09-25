use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use blueos_api::cdr_encoding;
use blueos_comms::Payload;
use blueos_idl::Message;
use blueos_idl::msg::blueos_recorder_msgs::{RecordingIndex, RecordingIndexRequest};
use blueos_recorder_mcap::index::{IndexError, walk_index};
use blueos_recorder_mcap::rewrite::{RewriteError, rewrite};
use blueos_recorder_policy::{
    LibraryCommand, RecorderCommand, RecordingOperationKind, ScannedRecording,
};
use blueos_recorder_storage::{RecordingsFolder, StorageError, StoredRecording};
use bytes::Bytes;
use tokio::sync::mpsc;
use tracing::{error, info, instrument};

pub struct LibraryIoContext {
    recorder_path: PathBuf,
    folder: Arc<Mutex<RecordingsFolder>>,
    cancel_flags: Mutex<BTreeMap<String, Arc<AtomicBool>>>,
    progress_sender: mpsc::Sender<RecorderCommand>,
}

impl LibraryIoContext {
    pub fn new(
        recorder_path: PathBuf,
        progress_sender: mpsc::Sender<RecorderCommand>,
    ) -> Result<Self, std::io::Error> {
        let folder = RecordingsFolder::new(recorder_path.clone())?;
        Ok(Self {
            recorder_path,
            folder: Arc::new(Mutex::new(folder)),
            cancel_flags: Mutex::new(BTreeMap::new()),
            progress_sender,
        })
    }

    pub fn folder(&self) -> Arc<Mutex<RecordingsFolder>> {
        self.folder.clone()
    }

    pub fn discard_leftovers(&self) {
        let folder = self.folder.lock().expect("folder lock");
        for (name, size) in folder.discard_leftovers() {
            info!(
                name = %name,
                size_bytes = size,
                "Discarded leftover repair temporary file"
            );
        }
    }

    #[instrument(skip_all, name = "library_io_handle")]
    pub async fn handle(
        &self,
        request: blueos_recorder_policy::LibraryIo,
        active_session: Option<String>,
    ) -> RecorderCommand {
        match request {
            blueos_recorder_policy::LibraryIo::Scan => self.scan(active_session).await,
            blueos_recorder_policy::LibraryIo::Repair { path } => self.repair(path).await,
            blueos_recorder_policy::LibraryIo::CancelRepair { path } => self.cancel_repair(path),
            blueos_recorder_policy::LibraryIo::Delete { path } => self.delete(path).await,
            blueos_recorder_policy::LibraryIo::Snapshot { path, output_path } => {
                self.snapshot(path, output_path).await
            }
        }
    }

    async fn scan(&self, active_session: Option<String>) -> RecorderCommand {
        let folder = self.folder.clone();
        let active = active_session;
        let result = tokio::task::spawn_blocking(move || {
            let mut folder = folder.lock().expect("folder lock");
            folder.scan(active.as_deref())
        })
        .await;
        let recordings = match result {
            Ok(Ok(recordings)) => recordings,
            Ok(Err(error)) => {
                error!(%error, "Recording library scan failed");
                return RecorderCommand::IoFailed;
            }
            Err(error) => {
                error!(%error, "Recording library scan task join failed");
                return RecorderCommand::IoFailed;
            }
        };
        RecorderCommand::Library(LibraryCommand::ScanCompleted {
            recordings: recordings.into_iter().map(stored_to_scanned).collect(),
            now_unix_seconds: now_unix_seconds(),
        })
    }

    async fn repair(&self, path: String) -> RecorderCommand {
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel_flags
            .lock()
            .expect("cancel lock")
            .insert(path.clone(), cancel.clone());
        let (source, temporary) = {
            let folder = self.folder.lock().expect("folder lock");
            let source = match folder.resolve(&path) {
                Ok(value) => value,
                Err(error) => {
                    self.cancel_flags.lock().expect("cancel lock").remove(&path);
                    return operation_finished(
                        RecordingOperationKind::Repair,
                        path,
                        String::new(),
                        false,
                        false,
                        error.to_string(),
                    );
                }
            };
            (source, folder.temporary_output(&path))
        };
        let progress_sender = self.progress_sender.clone();
        let progress_path = path.clone();
        let cancel_for_rewrite = cancel.clone();
        let rewrite_result = tokio::task::spawn_blocking(move || {
            let mut last_report = std::time::Instant::now();
            rewrite(
                &source,
                &temporary,
                &mut |read, total| {
                    if last_report.elapsed() < Duration::from_secs(1) {
                        return;
                    }
                    last_report = std::time::Instant::now();
                    if progress_sender
                        .blocking_send(RecorderCommand::Library(LibraryCommand::RepairProgress {
                            path: progress_path.clone(),
                            bytes_processed: read,
                            total_bytes: total,
                            now_unix_seconds: now_unix_seconds(),
                        }))
                        .is_err()
                    {
                        // Recorder is shutting down; cancel stops the rewrite loop.
                        cancel_for_rewrite.store(true, Ordering::Relaxed);
                    }
                },
                &cancel_for_rewrite,
            )
        })
        .await;
        self.cancel_flags.lock().expect("cancel lock").remove(&path);
        let temporary_path = {
            let folder = self.folder.lock().expect("folder lock");
            folder.temporary_output(&path)
        };
        match rewrite_result {
            Ok(Ok(_summary)) => {
                let folder = self.folder.clone();
                let path_for_replace = path.clone();
                let replace_result = tokio::task::spawn_blocking(move || {
                    let folder = folder.lock().expect("folder lock");
                    folder.replace(&temporary_path, &path_for_replace)
                })
                .await;
                match replace_result {
                    Ok(Ok(())) => operation_finished(
                        RecordingOperationKind::Repair,
                        path,
                        String::new(),
                        true,
                        false,
                        String::new(),
                    ),
                    Ok(Err(error)) => operation_finished(
                        RecordingOperationKind::Repair,
                        path,
                        String::new(),
                        false,
                        false,
                        error.to_string(),
                    ),
                    Err(error) => {
                        error!(%error, "Repair replace task join failed");
                        operation_finished(
                            RecordingOperationKind::Repair,
                            path,
                            String::new(),
                            false,
                            false,
                            error.to_string(),
                        )
                    }
                }
            }
            Ok(Err(RewriteError::Cancelled)) => operation_finished(
                RecordingOperationKind::Repair,
                path,
                String::new(),
                false,
                true,
                String::new(),
            ),
            Ok(Err(error)) => operation_finished(
                RecordingOperationKind::Repair,
                path,
                String::new(),
                false,
                false,
                error.to_string(),
            ),
            Err(error) => {
                error!(%error, "Repair rewrite task join failed");
                operation_finished(
                    RecordingOperationKind::Repair,
                    path,
                    String::new(),
                    false,
                    false,
                    error.to_string(),
                )
            }
        }
    }

    fn cancel_repair(&self, path: String) -> RecorderCommand {
        if let Some(cancel) = self.cancel_flags.lock().expect("cancel lock").get(&path) {
            cancel.store(true, Ordering::Relaxed);
        }
        RecorderCommand::Ack
    }

    async fn delete(&self, path: String) -> RecorderCommand {
        let folder = self.folder.clone();
        let relative_path = path.clone();
        let result = tokio::task::spawn_blocking(move || {
            let folder = folder.lock().expect("folder lock");
            folder.delete(&relative_path)
        })
        .await;
        match result {
            Ok(Ok(())) => operation_finished(
                RecordingOperationKind::Delete,
                path,
                String::new(),
                true,
                false,
                String::new(),
            ),
            Ok(Err(StorageError::Path(error))) => operation_finished(
                RecordingOperationKind::Delete,
                path,
                String::new(),
                false,
                false,
                error.to_string(),
            ),
            Ok(Err(StorageError::Io(error))) => operation_finished(
                RecordingOperationKind::Delete,
                path,
                String::new(),
                false,
                false,
                error.to_string(),
            ),
            Err(error) => {
                error!(%error, "Delete task join failed");
                operation_finished(
                    RecordingOperationKind::Delete,
                    path,
                    String::new(),
                    false,
                    false,
                    error.to_string(),
                )
            }
        }
    }

    async fn snapshot(&self, path: String, output_path: String) -> RecorderCommand {
        let cancel = Arc::new(AtomicBool::new(false));
        let (source, output) = {
            let folder = self.folder.lock().expect("folder lock");
            let source = match folder.resolve(&path) {
                Ok(value) => value,
                Err(error) => {
                    return operation_finished(
                        RecordingOperationKind::Snapshot,
                        path,
                        output_path,
                        false,
                        false,
                        error.to_string(),
                    );
                }
            };
            let output = self.recorder_path.join(&output_path);
            (source, output)
        };
        let rewrite_result = tokio::task::spawn_blocking(move || {
            rewrite(&source, &output, &mut |_read, _total| {}, &cancel)
        })
        .await;
        match rewrite_result {
            Ok(Ok(_summary)) => operation_finished(
                RecordingOperationKind::Snapshot,
                path,
                output_path,
                true,
                false,
                String::new(),
            ),
            Ok(Err(RewriteError::Cancelled)) => operation_finished(
                RecordingOperationKind::Snapshot,
                path,
                output_path,
                false,
                true,
                String::new(),
            ),
            Ok(Err(error)) => operation_finished(
                RecordingOperationKind::Snapshot,
                path,
                output_path,
                false,
                false,
                error.to_string(),
            ),
            Err(error) => {
                error!(%error, "Snapshot rewrite task join failed");
                operation_finished(
                    RecordingOperationKind::Snapshot,
                    path,
                    output_path,
                    false,
                    false,
                    error.to_string(),
                )
            }
        }
    }
}

fn operation_finished(
    operation: RecordingOperationKind,
    path: String,
    output_path: String,
    succeeded: bool,
    cancelled: bool,
    error: String,
) -> RecorderCommand {
    RecorderCommand::Library(LibraryCommand::OperationFinished {
        operation,
        path,
        output_path,
        succeeded,
        cancelled,
        error,
    })
}

fn stored_to_scanned(recording: StoredRecording) -> ScannedRecording {
    ScannedRecording {
        relative_path: recording.relative_path,
        name: recording.name,
        size_bytes: recording.size_bytes,
        modified_unix_seconds: (recording.modified_unix_nanos / 1_000_000_000) as i64,
        indexed: recording.indexed,
    }
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[instrument(skip_all, name = "library_io_index_query")]
pub async fn run_index_query(
    folder: Arc<Mutex<RecordingsFolder>>,
    payload: &[u8],
) -> Result<(Payload, String), String> {
    let request = RecordingIndexRequest::decode(payload).map_err(|error| error.to_string())?;
    let relative_path = request.path.clone();
    let from_offset = request.from_offset;
    let limit = request.limit;
    let path = {
        let folder = folder.lock().map_err(|error| error.to_string())?;
        folder
            .resolve(&relative_path)
            .map_err(|error| error.to_string())?
    };
    let index = tokio::task::spawn_blocking(move || walk_index(&path, from_offset, limit))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error: IndexError| error.to_string())?;
    let bytes = index.encode().map_err(|error| error.to_string())?;
    Ok((
        Payload::from_bytes(Bytes::from(bytes)),
        cdr_encoding(RecordingIndex::SCHEMA_NAME),
    ))
}
