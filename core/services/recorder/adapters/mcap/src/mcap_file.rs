//! One open MCAP file owned by the data plane writer actor.

use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::BufWriter,
    path::{Path, PathBuf},
    sync::Arc,
};

use mcap::{
    Compression, McapError as McapCrateError, WriteOptions, Writer, records::MessageHeader,
};
use thiserror::Error;

use blueos_comms::Payload;

use crate::channel_descriptor::ChannelDescriptor;

/// Errors while writing an MCAP file.
#[derive(Debug, Error)]
pub enum McapError {
    /// The writer actor is not running.
    #[error("the MCAP writer stopped")]
    WriterStopped,
    /// Filesystem operation failed.
    #[error("filesystem operation failed")]
    Io(#[source] std::io::Error),
    /// MCAP library error.
    #[error("MCAP write failed")]
    Mcap(#[source] McapCrateError),
    /// A channel descriptor lacked schema text.
    #[error("schema without content")]
    MissingSchemaContent,
}

/// Payload and metadata for one MCAP message write.
pub struct WriteSampleRequest {
    /// Zenoh topic key.
    pub topic: String,
    /// MCAP log time in nanoseconds.
    pub log_time: u64,
    /// MCAP publish time in nanoseconds.
    pub publish_time: u64,
    /// Sample payload (shared until the MCAP write).
    pub payload: Payload,
    /// Channel metadata registered once per topic.
    pub descriptor: Arc<ChannelDescriptor>,
}

/// An open MCAP recording file.
pub struct McapFile {
    path: PathBuf,
    file_name: String,
    writer: Writer<BufWriter<File>>,
    channels: BTreeMap<String, ChannelState>,
    bytes_written: u64,
}

struct ChannelState {
    channel_id: u16,
    sequence: u32,
}

impl McapFile {
    /// Creates a new MCAP file at `path` without truncating an existing file.
    pub fn open(path: PathBuf, file_name: String) -> Result<Self, McapError> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(McapError::Io)?;
        let writer = Writer::with_options(
            BufWriter::new(file),
            WriteOptions::new()
                .compression(Some(Compression::Lz4))
                .emit_message_indexes(true),
        )
        .map_err(McapError::Mcap)?;
        Ok(Self {
            path,
            file_name,
            writer,
            channels: BTreeMap::new(),
            bytes_written: 0,
        })
    }

    /// Basename reported to the Domain.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Path on disk.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Bytes written to the file body so far.
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Writes one queued sample.
    pub fn write_sample_request(&mut self, request: &WriteSampleRequest) -> Result<(), McapError> {
        let channel_id = self.channel_id_for(&request.topic, request.descriptor.as_ref())?;
        let payload = request.payload.to_bytes();
        let payload = payload.as_ref();
        let sequence = self
            .channels
            .get(&request.topic)
            .expect("channel exists")
            .sequence;
        self.writer
            .write_to_known_channel(
                &MessageHeader {
                    channel_id,
                    sequence,
                    log_time: request.log_time,
                    publish_time: request.publish_time,
                },
                payload,
            )
            .map_err(McapError::Mcap)?;
        if let Some(channel) = self.channels.get_mut(&request.topic) {
            channel.sequence += 1;
        }
        self.bytes_written = self.bytes_written.saturating_add(payload.len() as u64);
        Ok(())
    }

    /// Finishes the MCAP file so readers can open it.
    pub fn finish(mut self) -> Result<u64, McapError> {
        self.writer.finish().map_err(McapError::Mcap)?;
        Ok(self.bytes_written)
    }

    fn channel_id_for(
        &mut self,
        topic: &str,
        descriptor: &ChannelDescriptor,
    ) -> Result<u16, McapError> {
        if let Some(state) = self.channels.get(topic) {
            return Ok(state.channel_id);
        }
        let schema_id = match &descriptor.schema {
            Some(schema) => {
                let content = schema
                    .content
                    .as_ref()
                    .ok_or(McapError::MissingSchemaContent)?;
                self.writer
                    .add_schema(
                        &content.name,
                        schema.encoding.as_str(),
                        content.data.as_bytes(),
                    )
                    .map_err(McapError::Mcap)?
            }
            None => 0,
        };
        let channel_id = self
            .writer
            .add_channel(
                schema_id,
                topic,
                descriptor.message_encoding.as_str(),
                &BTreeMap::new(),
            )
            .map_err(McapError::Mcap)?;
        self.channels.insert(
            topic.to_owned(),
            ChannelState {
                channel_id,
                sequence: 0,
            },
        );
        Ok(channel_id)
    }
}

/// Whether a backbone key should be recorded.
pub fn should_record_topic(key: &str) -> bool {
    if key.starts_with("blueos/v1/recorder/") {
        return false;
    }
    if key.starts_with("blueos/v1/services/") {
        return false;
    }
    true
}

/// Picks or builds the channel descriptor for `sample`, reusing `cache` when present.
pub fn descriptor_for_sample(
    topic: &str,
    encoding: &str,
    payload: &blueos_comms::Payload,
    cache: &mut BTreeMap<String, Arc<ChannelDescriptor>>,
) -> Option<Arc<ChannelDescriptor>> {
    if !cache.contains_key(topic) {
        let descriptor =
            crate::channel_descriptor::channel_descriptor_for_sample(topic, encoding, payload)?;
        cache.insert(topic.to_owned(), Arc::new(descriptor));
    }
    cache.get(topic).cloned()
}
