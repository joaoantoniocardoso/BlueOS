use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;
use std::num::{NonZeroU64, NonZeroUsize};
use std::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
    mpsc::{self, SyncSender, TrySendError},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use blueos_comms::Payload;
use clap::ValueEnum;
use mcap::{Compression, Writer, write::WriteOptions};
use tracing::{error, info, instrument};

use crate::channel_descriptor::ChannelDescriptor;

const NO_SCHEMA_ID: u16 = 0;
const IO_BUFFER_BYTES: NonZeroUsize = NonZeroUsize::new(4 * 1024 * 1024).unwrap();
pub const DEFAULT_CHUNK_BYTES: NonZeroU64 = NonZeroU64::new(10 * 1024 * 1024).unwrap();
pub const DEFAULT_FLUSH_INTERVAL_SECS: NonZeroU64 = NonZeroU64::new(30).unwrap();
const WRITER_QUEUE_CAPACITY: NonZeroUsize = NonZeroUsize::new(4096).unwrap();
const DROP_LOG_EVERY: u64 = 256;

#[derive(Clone, Copy, Debug)]
pub struct McapWriteConfig {
    pub compression: McapCompression,
    pub chunk_size: NonZeroU64,
    pub flush_interval_secs: NonZeroU64,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum McapCompression {
    None,
    #[default]
    Lz4,
    Zstd,
}

pub struct McapSession {
    writer: Arc<McapWriterHandle>,
    writer_thread: Option<JoinHandle<Result<()>>>,
    flush_interval: Duration,
    last_flush: Instant,
    finished: bool,
}

pub struct McapWriterHandle {
    sender: SyncSender<WriterCommand>,
    known_topics: Mutex<HashSet<Arc<str>>>,
    dropped_writes: AtomicU64,
}

enum WriterCommand {
    Write {
        topic: Arc<str>,
        log_time: u64,
        publish_time: u64,
        payload: Payload,
        new_channel: Option<ChannelDescriptor>,
    },
    Flush,
    Finish,
}

struct Channel {
    channel_id: u16,
    sequence: u32,
}

impl Default for McapWriteConfig {
    fn default() -> Self {
        Self {
            compression: McapCompression::Lz4,
            chunk_size: DEFAULT_CHUNK_BYTES,
            flush_interval_secs: DEFAULT_FLUSH_INTERVAL_SECS,
        }
    }
}

impl McapWriteConfig {
    fn open_writer(path: &Path, config: Self) -> Result<Writer<BufWriter<File>>> {
        let file = File::create(path).context("Failed to create MCAP file")?;
        let io_buffer_bytes = IO_BUFFER_BYTES.get().max(config.chunk_size.get() as usize);
        Writer::with_options(
            BufWriter::with_capacity(io_buffer_bytes, file),
            config.into(),
        )
        .context("Failed to create MCAP writer")
    }
}

impl From<McapWriteConfig> for WriteOptions {
    fn from(value: McapWriteConfig) -> Self {
        let compression = match value.compression {
            McapCompression::None => None,
            McapCompression::Lz4 => Some(Compression::Lz4),
            McapCompression::Zstd => Some(Compression::Zstd),
        };

        WriteOptions::new()
            .compression(compression)
            .chunk_size(Some(value.chunk_size.into()))
            .compression_threads(if matches!(value.compression, McapCompression::None) {
                0
            } else {
                2
            })
            .emit_message_indexes(true)
    }
}

impl McapSession {
    #[instrument(skip_all, fields(path = %path.display()))]
    pub fn open(path: &Path, config: McapWriteConfig) -> Result<Self> {
        info!(
            compression = ?config.compression,
            chunk_bytes = config.chunk_size,
            flush_interval_secs = config.flush_interval_secs,
            "Opening MCAP file"
        );
        let writer = McapWriteConfig::open_writer(path, config)?;
        let (sender, receiver) = mpsc::sync_channel(WRITER_QUEUE_CAPACITY.into());
        let writer_thread = thread::Builder::new()
            .name("mcap-writer".into())
            .spawn(move || writer_loop(receiver, writer))
            .context("Failed to spawn MCAP writer thread")?;

        Ok(Self {
            writer: Arc::new(McapWriterHandle {
                sender,
                known_topics: Mutex::new(HashSet::new()),
                dropped_writes: AtomicU64::new(0),
            }),
            writer_thread: Some(writer_thread),
            flush_interval: Duration::from_secs(config.flush_interval_secs.get()),
            last_flush: Instant::now(),
            finished: false,
        })
    }

    pub fn writer(&self) -> Arc<McapWriterHandle> {
        self.writer.clone()
    }

    pub fn flush_if_due(&mut self) -> Result<()> {
        if self.last_flush.elapsed() < self.flush_interval {
            return Ok(());
        }
        self.writer.flush()?;
        self.last_flush = Instant::now();
        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.finish_inner()?;
        Ok(())
    }

    fn finish_inner(&mut self) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        self.writer.flush()?;
        self.writer.send_finish()?;
        if let Some(handle) = self.writer_thread.take() {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => return Err(error),
                Err(_) => return Err(anyhow!("MCAP writer thread panicked")),
            }
        }
        self.finished = true;
        Ok(())
    }
}

impl Drop for McapSession {
    fn drop(&mut self) {
        if let Err(error) = self.finish_inner() {
            error!(%error, "Failed to finish MCAP session on drop");
        }
    }
}

impl McapWriterHandle {
    pub fn write_message(
        &self,
        topic: &str,
        log_time: u64,
        publish_time: u64,
        payload: Payload,
        new_channel: Option<ChannelDescriptor>,
    ) -> Result<()> {
        let known = self
            .known_topics
            .lock()
            .expect("mcap known_topics poisoned");
        let (topic, new_channel) = if let Some(existing) = known.get(topic) {
            (existing.clone(), None)
        } else {
            drop(known);
            let Some(descriptor) = new_channel else {
                return Ok(());
            };
            if descriptor.topic != topic {
                return Err(anyhow!(
                    "Channel descriptor topic mismatch: {}",
                    descriptor.topic
                ));
            }
            let topic_arc: Arc<str> = Arc::from(topic);
            self.known_topics
                .lock()
                .expect("mcap known_topics poisoned")
                .insert(topic_arc.clone());
            (topic_arc, Some(descriptor))
        };

        let command = WriterCommand::Write {
            topic,
            log_time,
            publish_time,
            payload,
            new_channel,
        };
        match self.sender.try_send(command) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => {
                let dropped = self.dropped_writes.fetch_add(1, Ordering::Relaxed) + 1;
                if dropped == 1 || dropped.is_multiple_of(DROP_LOG_EVERY) {
                    error!(
                        dropped,
                        "MCAP writer queue full, dropping message (backpressure)"
                    );
                }
                Ok(())
            }
            Err(TrySendError::Disconnected(_)) => Err(anyhow!("MCAP writer thread stopped")),
        }
    }

    pub fn flush(&self) -> Result<()> {
        self.sender
            .send(WriterCommand::Flush)
            .map_err(|_| anyhow!("MCAP writer thread stopped"))
    }

    fn send_finish(&self) -> Result<()> {
        self.sender
            .send(WriterCommand::Finish)
            .map_err(|_| anyhow!("MCAP writer thread stopped"))
    }
}

impl Channel {
    fn new(channel_id: u16) -> Self {
        Self {
            channel_id,
            sequence: 0,
        }
    }
}

fn writer_loop(
    receiver: mpsc::Receiver<WriterCommand>,
    mut writer: Writer<BufWriter<File>>,
) -> Result<()> {
    let mut channels = HashMap::<Arc<str>, Channel>::new();

    for command in receiver {
        match command {
            WriterCommand::Write {
                topic,
                log_time,
                publish_time,
                payload,
                new_channel,
            } => {
                if let Some(descriptor) = new_channel {
                    register_channel(&mut writer, &mut channels, descriptor)?;
                }

                let channel = channels
                    .get_mut(&topic)
                    .ok_or_else(|| anyhow!("Channel not registered for topic {topic}"))?;

                let header = mcap::records::MessageHeader {
                    channel_id: channel.channel_id,
                    sequence: channel.sequence,
                    log_time,
                    publish_time,
                };

                let data = payload_bytes(&payload);
                if let Err(error) = writer.write_to_known_channel(&header, data.as_ref()) {
                    error!(%error, topic = %topic, "Failed to write message to MCAP channel");
                } else {
                    channel.sequence += 1;
                }
            }
            WriterCommand::Flush => {
                if let Err(error) = writer.flush() {
                    error!(%error, "Failed to flush MCAP writer");
                }
            }
            WriterCommand::Finish => {
                writer.finish().context("Failed to finish MCAP writer")?;
                break;
            }
        }
    }

    Ok(())
}

fn payload_bytes(payload: &Payload) -> Cow<'_, [u8]> {
    Cow::Owned(payload.to_vec())
}

fn register_channel(
    writer: &mut Writer<BufWriter<File>>,
    channels: &mut HashMap<Arc<str>, Channel>,
    descriptor: ChannelDescriptor,
) -> Result<()> {
    if channels.contains_key(descriptor.topic.as_str()) {
        return Err(anyhow!("Channel already registered"));
    }

    let schema_id = match &descriptor.schema {
        Some(schema) => match &schema.content {
            Some(content) => writer
                .add_schema(
                    &content.name,
                    schema.encoding.as_str(),
                    content.data.as_bytes(),
                )
                .context("Failed to add MCAP schema")?,
            None => NO_SCHEMA_ID,
        },
        None => NO_SCHEMA_ID,
    };

    let channel_id = writer
        .add_channel(
            schema_id,
            &descriptor.topic,
            descriptor.message_encoding.as_str(),
            &BTreeMap::new(),
        )
        .context("Failed to add MCAP channel")?;

    info!(topic = %descriptor.topic, "Adding channel");
    channels.insert(Arc::from(descriptor.topic), Channel::new(channel_id));
    Ok(())
}
