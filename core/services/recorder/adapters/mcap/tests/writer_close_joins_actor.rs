//! `close` must join the writer actor after shutdown IO, or readers can see a truncated MCAP.

use std::sync::Arc;

use bytes::Bytes;
use tempfile::tempdir;

use blueos_comms::Payload;
use blueos_recorder_mcap::{ChannelDescriptor, ChannelRoute, McapWriterHandle, MessageEncoding};

#[tokio::test(flavor = "multi_thread")]
async fn close_without_finish_waits_for_writer_actor_shutdown() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("recorder_drop.mcap");
    let writer = McapWriterHandle::spawn_with_queue_capacity(8);
    writer
        .open(path.clone(), "recorder_drop.mcap".into())
        .await
        .expect("open");

    let descriptor = Arc::new(ChannelDescriptor {
        topic: "test/topic".into(),
        schema: None,
        message_encoding: MessageEncoding::OctetStream,
    });
    writer.try_write_sample(
        "test/topic".into(),
        ChannelRoute::for_topic("test/topic"),
        0,
        0,
        Payload::new(Bytes::from_static(b"x")),
        descriptor,
    );

    writer.close().await.expect("close joins the writer actor");

    let bytes = std::fs::read(&path).expect("read");
    mcap::Summary::read(&bytes)
        .expect("read summary")
        .expect("MCAP footer present after close");
}
