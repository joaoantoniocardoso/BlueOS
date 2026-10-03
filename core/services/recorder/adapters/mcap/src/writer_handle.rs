//! Async actor that owns [`McapFile`] and runs blocking IO on the runtime's blocking pool.

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::{path::PathBuf, sync::Arc};

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tracing::warn;

use blueos_comms::Payload;

use crate::{
    channel_descriptor::{ChannelDescriptor, ChannelRoute},
    mcap_file::{McapError, McapFile, WriteSampleRequest},
};

const DEFAULT_WRITER_QUEUE_CAPACITY: usize = 4096;
const DEFAULT_WRITER_QUEUE_BYTES: usize = 32 * 1024 * 1024;

/// Commands handled by the writer actor.
enum WriterCommand {
    Open {
        path: PathBuf,
        file_name: String,
        reply: oneshot::Sender<Result<(), McapError>>,
    },
    Write(WriteSampleRequest),
    Finish {
        reply: oneshot::Sender<Result<u64, McapError>>,
    },
    Shutdown,
}

/// Handle to the MCAP writer actor (one open file at a time).
pub struct McapWriterHandle {
    command_sender: mpsc::Sender<WriterCommand>,
    bytes_written: Arc<AtomicU64>,
    dropped_samples: Arc<AtomicU64>,
    /// Payload bytes of the samples queued and not written yet.
    queued_bytes: Arc<AtomicUsize>,
    /// The most payload bytes the queue holds; a sample past it is dropped.
    queue_bytes: usize,
    #[expect(dead_code, reason = "writer actor; join is not used on Drop")]
    actor: JoinHandle<()>,
}

impl Drop for McapWriterHandle {
    fn drop(&mut self) {
        let _ = self.command_sender.try_send(WriterCommand::Shutdown);
    }
}

impl McapWriterHandle {
    /// Starts the writer actor with the default queue byte budget.
    pub fn spawn() -> Self {
        Self::spawn_with_queue_bytes(DEFAULT_WRITER_QUEUE_BYTES)
    }

    /// Starts the writer actor with a queue that holds at most `queue_bytes` of sample payloads (for tests and
    /// tuning).
    pub fn spawn_with_queue_bytes(queue_bytes: usize) -> Self {
        let (command_sender, command_receiver) = mpsc::channel(DEFAULT_WRITER_QUEUE_CAPACITY);
        let bytes_written = Arc::new(AtomicU64::new(0));
        let dropped_samples = Arc::new(AtomicU64::new(0));
        let queued_bytes = Arc::new(AtomicUsize::new(0));
        let actor = tokio::spawn(writer_actor(
            command_receiver,
            Arc::clone(&bytes_written),
            Arc::clone(&queued_bytes),
        ));
        Self {
            command_sender,
            bytes_written,
            dropped_samples,
            queued_bytes,
            queue_bytes,
            actor,
        }
    }

    /// Opens a new file on the writer actor.
    pub async fn open(&self, path: PathBuf, file_name: String) -> Result<(), McapError> {
        let (reply_sender, reply_receiver) = oneshot::channel();
        self.command_sender
            .send(WriterCommand::Open {
                path,
                file_name,
                reply: reply_sender,
            })
            .await
            .map_err(|_| McapError::WriterStopped)?;
        reply_receiver.await.map_err(|_| McapError::WriterStopped)?
    }

    /// Queues a sample write; drops the sample when the queue is full or the sample would take it past its byte
    /// budget.
    pub fn try_write_sample(
        &self,
        topic: String,
        route: ChannelRoute,
        log_time: u64,
        publish_time: u64,
        payload: Payload,
        descriptor: Arc<ChannelDescriptor>,
    ) {
        let payload_bytes = payload.size_bytes();
        let queued_bytes = self
            .queued_bytes
            .fetch_add(payload_bytes, Ordering::Relaxed);
        if queued_bytes.saturating_add(payload_bytes) > self.queue_bytes {
            self.queued_bytes
                .fetch_sub(payload_bytes, Ordering::Relaxed);
            self.dropped_samples.fetch_add(1, Ordering::Relaxed);
            return;
        }
        let command = WriterCommand::Write(WriteSampleRequest {
            topic,
            route,
            log_time,
            publish_time,
            payload,
            descriptor,
        });
        if self.command_sender.try_send(command).is_err() {
            self.queued_bytes
                .fetch_sub(payload_bytes, Ordering::Relaxed);
            self.dropped_samples.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Finishes the current file and returns bytes written.
    pub async fn finish(&self) -> Result<u64, McapError> {
        let (reply_sender, reply_receiver) = oneshot::channel();
        self.command_sender
            .send(WriterCommand::Finish {
                reply: reply_sender,
            })
            .await
            .map_err(|_| McapError::WriterStopped)?;
        reply_receiver.await.map_err(|_| McapError::WriterStopped)?
    }

    /// Bytes written to the open file (updated after each write batch).
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written.load(Ordering::Relaxed)
    }

    /// Takes the number of samples dropped since the last call.
    pub fn take_dropped_samples(&self) -> u64 {
        self.dropped_samples.swap(0, Ordering::Relaxed)
    }
}

async fn writer_actor(
    mut command_receiver: mpsc::Receiver<WriterCommand>,
    bytes_written: Arc<AtomicU64>,
    queued_bytes: Arc<AtomicUsize>,
) {
    let mut open: Option<McapFile> = None;
    let mut pending: Option<WriterCommand> = None;
    loop {
        let command = match pending.take() {
            Some(command) => command,
            None => match command_receiver.recv().await {
                Some(command) => command,
                None => break,
            },
        };
        match command {
            WriterCommand::Open {
                path,
                file_name,
                reply,
            } => {
                let previous = open.take();
                let open_result = tokio::task::spawn_blocking(move || {
                    if let Some(file) = previous {
                        let _ = finish_file(file, "before opening a new MCAP file");
                    }
                    McapFile::open(path, file_name)
                })
                .await;
                let open_result = match open_result {
                    Ok(result) => result,
                    Err(join_error) => {
                        warn!(%join_error, "MCAP open task failed");
                        let _ = reply.send(Err(McapError::WriterStopped));
                        continue;
                    }
                };
                match open_result {
                    Ok(file) => {
                        bytes_written.store(0, Ordering::Relaxed);
                        open = Some(file);
                        let _ = reply.send(Ok(()));
                    }
                    Err(error) => {
                        let _ = reply.send(Err(error));
                    }
                }
            }
            WriterCommand::Write(first) => {
                let mut batch = vec![first];
                loop {
                    match command_receiver.try_recv() {
                        Ok(WriterCommand::Write(request)) => batch.push(request),
                        Ok(other) => {
                            pending = Some(other);
                            break;
                        }
                        Err(_) => break,
                    }
                }
                let batch_bytes: usize = batch
                    .iter()
                    .map(|request| request.payload.size_bytes())
                    .sum();
                let Some(mut file) = open.take() else {
                    queued_bytes.fetch_sub(batch_bytes, Ordering::Relaxed);
                    continue;
                };
                let write_result = tokio::task::spawn_blocking(move || {
                    for request in batch {
                        match file.write_sample_request(&request) {
                            Ok(()) => {}
                            Err(error) => {
                                warn!(%error, topic = %request.topic, "failed to write MCAP sample");
                            }
                        }
                    }
                    let bytes = file.bytes_written();
                    (file, bytes)
                })
                .await;
                queued_bytes.fetch_sub(batch_bytes, Ordering::Relaxed);
                match write_result {
                    Ok((written_file, bytes)) => {
                        open = Some(written_file);
                        bytes_written.store(bytes, Ordering::Relaxed);
                    }
                    Err(join_error) => {
                        warn!(%join_error, "MCAP write task failed");
                    }
                }
            }
            WriterCommand::Finish { reply } => {
                let taken = open.take();
                let result = tokio::task::spawn_blocking(move || {
                    taken.map_or(Ok(0), |file| finish_file(file, "on Finish command"))
                })
                .await;
                let result = match result {
                    Ok(result) => result,
                    Err(join_error) => {
                        warn!(%join_error, "MCAP finish task failed");
                        Err(McapError::WriterStopped)
                    }
                };
                let _ = reply.send(result);
            }
            WriterCommand::Shutdown => break,
        }
    }
    if let Some(file) = open.take() {
        let _ = tokio::task::spawn_blocking(move || finish_file(file, "on writer shutdown")).await;
    }
}

fn finish_file(file: McapFile, context: &str) -> Result<u64, McapError> {
    match file.finish() {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            warn!(%error, context, "failed to finish MCAP file");
            Err(error)
        }
    }
}
