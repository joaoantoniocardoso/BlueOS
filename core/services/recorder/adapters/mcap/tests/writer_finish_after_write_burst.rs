//! Regression: draining write batches must not drop a queued `Finish`.

use std::sync::Arc;

use bytes::Bytes;
use tempfile::tempdir;

use blueos_comms::Payload;
use blueos_recorder_mcap::{ChannelDescriptor, McapWriterHandle, MessageEncoding};

#[tokio::test(flavor = "multi_thread")]
async fn finish_returns_bytes_for_every_queued_sample() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("recorder_test.mcap");
    let writer = McapWriterHandle::spawn_with_queue_capacity(64);
    writer
        .open(path.clone(), "recorder_test.mcap".into())
        .await
        .expect("open");

    let descriptor = Arc::new(ChannelDescriptor {
        topic: "test/topic".into(),
        schema: None,
        message_encoding: MessageEncoding::OctetStream,
    });
    const SAMPLE_COUNT: u64 = 32;
    const PAYLOAD: &[u8] = b"sample-bytes";
    for _ in 0..SAMPLE_COUNT {
        writer.try_write_sample(
            "test/topic".into(),
            0,
            0,
            Payload::new(Bytes::from_static(PAYLOAD)),
            Arc::clone(&descriptor),
        );
    }

    let bytes = writer.finish().await.expect("finish must not be dropped");
    assert_eq!(bytes, SAMPLE_COUNT * PAYLOAD.len() as u64);
}
