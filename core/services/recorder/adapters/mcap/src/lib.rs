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
mod writer_handle;

pub use footer::{Footer, MCAP_MAGIC, is_indexed, read_footer_at};
pub use index::{IndexError, walk_index, walk_index_reader};

pub use channel_descriptor::{
    ChannelDescriptor, ChannelRoute, MessageEncoding, SchemaEncoding,
    channel_descriptor_cdr_fallback, channel_descriptor_for_ros2_type,
    channel_descriptor_for_sample,
};
pub use mcap_file::{
    McapError, McapFile, WriteSampleRequest, cached_descriptor, descriptor_for_sample,
    ros2_lane_descriptor,
};
pub use rewrite::{RewriteError, RewriteSummary, SOURCE_READ_BYTES, rewrite, rewrite_from_reader};
pub use writer_handle::McapWriterHandle;
