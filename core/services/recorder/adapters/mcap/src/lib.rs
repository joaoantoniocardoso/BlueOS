//! MCAP adapter: one [`McapFile`] per open recording, channel descriptors, zero-copy payloads.

#![expect(
    clippy::pub_use,
    reason = "adapter crate exposes channel and file types at the root"
)]

mod channel_descriptor;
mod footer;
mod index;
mod mcap_file;
mod rewrite;
mod summary;
mod writer_handle;
mod writer_metrics;

pub use footer::{Footer, MCAP_MAGIC, is_indexed, read_footer_at};
pub use index::{IndexError, walk_index, walk_index_reader};

pub use channel_descriptor::{
    ChannelDescriptor, ChannelRoute, MessageEncoding, SchemaEncoding,
    channel_descriptor_cdr_fallback, channel_descriptor_for_ros2_type,
    channel_descriptor_for_sample,
};
pub use mcap::write::WriteOptions;
pub use mcap_file::{
    McapError, McapFile, WriteSampleRequest, cached_descriptor, ros2_lane_descriptor,
};
pub use rewrite::{RewriteError, RewriteSummary, SOURCE_READ_BYTES, rewrite, rewrite_from_reader};
pub use summary::{COMPRESSED_VIDEO_SCHEMA, RecordingContents, read_recording_contents};
pub use writer_handle::McapWriterHandle;

// qual:test_helper
/// Default MCAP chunk size used by the data-plane writer ([`WriteOptions::DEFAULT_CHUNK_SIZE`]).
pub const RECORDING_WRITE_CHUNK_SIZE: u64 = WriteOptions::DEFAULT_CHUNK_SIZE;
