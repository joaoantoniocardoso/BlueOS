//! MCAP adapter: one [`McapFile`] per open recording, channel descriptors, zero-copy payloads.

#![expect(
    clippy::pub_use,
    reason = "adapter crate exposes channel and file types at the root"
)]

mod channel_descriptor;
mod mcap_file;
mod writer_handle;

pub use channel_descriptor::{
    ChannelDescriptor, ChannelRoute, MessageEncoding, SchemaEncoding,
    channel_descriptor_cdr_fallback, channel_descriptor_for_ros2_type,
    channel_descriptor_for_sample,
};
pub use mcap_file::{
    McapError, McapFile, WriteSampleRequest, cached_descriptor, descriptor_for_sample,
    ros2_lane_descriptor, should_record_topic,
};
pub use writer_handle::McapWriterHandle;
