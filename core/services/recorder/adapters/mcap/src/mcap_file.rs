//! One open MCAP file owned by the data plane writer actor.

use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::BufWriter,
    path::PathBuf,
    sync::Arc,
};

use mcap::{
    Compression, McapError as McapCrateError, WriteOptions, Writer, records::MessageHeader,
};
use thiserror::Error;
use tracing::info;

use blueos_comms::Payload;

use crate::channel_descriptor::{
    ChannelDescriptor, ChannelRoute, channel_descriptor_cdr_fallback,
    channel_descriptor_for_ros2_type,
};

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
    /// Writer route (supports multiple channels per topic).
    pub route: ChannelRoute,
    /// MCAP log time in nanoseconds.
    pub log_time: u64,
    /// MCAP publish time in nanoseconds.
    pub publish_time: u64,
    /// Sample payload (shared until the MCAP write).
    pub payload: Payload,
    /// Channel metadata registered once per route.
    pub descriptor: Arc<ChannelDescriptor>,
}

/// An open MCAP recording file.
pub struct McapFile {
    /// Basename reported to the Domain.
    pub(crate) file_name: String,
    writer: Writer<BufWriter<File>>,
    channels: BTreeMap<ChannelRoute, ChannelState>,
    /// Bytes written to the file body so far.
    pub(crate) bytes_written: u64,
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
            file_name,
            writer,
            channels: BTreeMap::new(),
            bytes_written: 0,
        })
    }

    /// Writes one queued sample.
    pub fn write_sample_request(&mut self, request: &WriteSampleRequest) -> Result<(), McapError> {
        let channel_id =
            self.channel_id_for(&request.route, &request.topic, request.descriptor.as_ref())?;
        let payload = request.payload.to_bytes();
        let payload = payload.as_ref();
        let sequence = match self.channels.get(&request.route) {
            Some(channel) => channel.sequence,
            None => return Err(McapError::WriterStopped),
        };
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
        if let Some(channel) = self.channels.get_mut(&request.route) {
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
        route: &ChannelRoute,
        topic: &str,
        descriptor: &ChannelDescriptor,
    ) -> Result<u16, McapError> {
        if let Some(state) = self.channels.get(route) {
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
        info!(
            topic,
            type_name = route.type_name.as_deref(),
            file_name = self.file_name,
            "Added a channel to the recording"
        );
        self.channels.insert(
            route.clone(),
            ChannelState {
                channel_id,
                sequence: 0,
            },
        );
        Ok(channel_id)
    }
}

/// Descriptor for a ros2dds schema lane (typed or fallback when schema text is unknown).
pub fn ros2_lane_descriptor(
    topic: &str,
    type_name: Option<&str>,
    cache: &mut BTreeMap<ChannelRoute, Arc<ChannelDescriptor>>,
) -> (ChannelRoute, Arc<ChannelDescriptor>) {
    match type_name {
        Some(type_name) => {
            let route = ChannelRoute::typed(topic, type_name);
            let descriptor = cached_descriptor(cache, &route, || {
                channel_descriptor_for_ros2_type(topic, type_name)
            });
            (route, descriptor)
        }
        None => {
            let route = ChannelRoute::for_topic(topic);
            let descriptor =
                cached_descriptor(cache, &route, || channel_descriptor_cdr_fallback(topic));
            (route, descriptor)
        }
    }
}

/// Returns a cached descriptor for `route`, building it when missing.
pub fn cached_descriptor(
    cache: &mut BTreeMap<ChannelRoute, Arc<ChannelDescriptor>>,
    route: &ChannelRoute,
    build: impl FnOnce() -> ChannelDescriptor,
) -> Arc<ChannelDescriptor> {
    if let Some(descriptor) = cache.get(route) {
        return Arc::clone(descriptor);
    }
    let descriptor = Arc::new(build());
    cache.insert(route.clone(), Arc::clone(&descriptor));
    descriptor
}
