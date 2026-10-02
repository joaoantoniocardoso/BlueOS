//! Background thread that owns [`McapFile`] and performs blocking IO.

use core::sync::atomic::{AtomicU64, Ordering};
use std::{
    path::PathBuf,
    sync::Arc,
    thread::{self, JoinHandle},
};

use tokio::sync::{mpsc, oneshot};
use tracing::warn;

use blueos_comms::Payload;

use crate::{
    channel_descriptor::ChannelDescriptor,
    mcap_file::{McapError, McapFile, WriteSampleRequest},
};

const DEFAULT_WRITER_QUEUE_CAPACITY: usize = 4096;

/// Commands handled on the writer thread.
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

/// Handle to the MCAP writer thread (one open file at a time).
pub struct McapWriterHandle {
    command_sender: mpsc::Sender<WriterCommand>,
    bytes_written: Arc<AtomicU64>,
    dropped_samples: Arc<AtomicU64>,
    #[expect(dead_code, reason = "detached writer thread; join is not used on Drop")]
    join: JoinHandle<()>,
}

impl Drop for McapWriterHandle {
    fn drop(&mut self) {
        let _ = self.command_sender.try_send(WriterCommand::Shutdown);
    }
}

impl McapWriterHandle {
    /// Starts the writer thread with the default queue capacity.
    pub fn spawn() -> Self {
        Self::spawn_with_queue_capacity(DEFAULT_WRITER_QUEUE_CAPACITY)
    }

    /// Starts the writer thread with a bounded command queue (for tests and tuning).
    pub fn spawn_with_queue_capacity(capacity: usize) -> Self {
        let (command_sender, command_receiver) = mpsc::channel(capacity);
        let bytes_written = Arc::new(AtomicU64::new(0));
        let dropped_samples = Arc::new(AtomicU64::new(0));
        let bytes_for_thread = Arc::clone(&bytes_written);
        let join = thread::spawn(move || writer_loop(command_receiver, bytes_for_thread));
        Self {
            command_sender,
            bytes_written,
            dropped_samples,
            join,
        }
    }

    /// Opens a new file on the writer thread.
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

    /// Queues a sample write; drops the sample when the queue is full.
    pub fn try_write_sample(
        &self,
        topic: String,
        log_time: u64,
        publish_time: u64,
        payload: Payload,
        descriptor: Arc<ChannelDescriptor>,
    ) {
        let command = WriterCommand::Write(WriteSampleRequest {
            topic,
            log_time,
            publish_time,
            payload,
            descriptor,
        });
        if self.command_sender.try_send(command).is_err() {
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

    /// Bytes written to the open file (updated on the writer thread).
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written.load(Ordering::Relaxed)
    }

    /// Takes the number of samples dropped since the last call.
    pub fn take_dropped_samples(&self) -> u64 {
        self.dropped_samples.swap(0, Ordering::Relaxed)
    }
}

fn writer_loop(mut command_receiver: mpsc::Receiver<WriterCommand>, bytes_written: Arc<AtomicU64>) {
    let mut open: Option<McapFile> = None;
    loop {
        let command = match command_receiver.blocking_recv() {
            Some(command) => command,
            None => break,
        };
        match command {
            WriterCommand::Open {
                path,
                file_name,
                reply,
            } => {
                let _ = finish_open_file(&mut open, "before opening a new MCAP file");
                bytes_written.store(0, Ordering::Relaxed);
                let result = McapFile::open(path, file_name).map(|file| {
                    open = Some(file);
                });
                let _ = reply.send(result);
            }
            WriterCommand::Write(request) => {
                let Some(file) = open.as_mut() else {
                    continue;
                };
                match file.write_sample_request(&request) {
                    Ok(()) => {
                        bytes_written.store(file.bytes_written(), Ordering::Relaxed);
                    }
                    Err(error) => {
                        warn!(%error, topic = %request.topic, "failed to write MCAP sample");
                    }
                }
            }
            WriterCommand::Finish { reply } => {
                let result = finish_open_file(&mut open, "on Finish command");
                let _ = reply.send(result);
            }
            WriterCommand::Shutdown => break,
        }
    }
    let _ = finish_open_file(&mut open, "on writer shutdown");
}

fn finish_open_file(open: &mut Option<McapFile>, context: &str) -> Result<u64, McapError> {
    let Some(file) = open.take() else {
        return Ok(0);
    };
    match file.finish() {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            warn!(%error, context, "failed to finish MCAP file");
            Err(error)
        }
    }
}
