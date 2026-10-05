//! The writer queue holds at most its byte budget of samples, and counts each sample it drops.

use std::sync::Arc;

use bytes::Bytes;
use tempfile::tempdir;

use blueos_comms::Payload;
use blueos_recorder_mcap::{
    ChannelDescriptor, ChannelRoute, McapWriterHandle, MessageEncoding, WriteSampleRequest,
};

#[tokio::test]
async fn a_sample_past_the_queue_byte_budget_is_dropped_and_counted() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("recorder_test.mcap");
    let writer = McapWriterHandle::spawn_with_queue_bytes(4 * 1024);
    writer
        .open(path, "recorder_test.mcap".into())
        .await
        .expect("open");
    let descriptor = Arc::new(ChannelDescriptor {
        topic: "test/topic".into(),
        schema: None,
        message_encoding: MessageEncoding::OctetStream,
    });

    for _ in 0..8 {
        writer.try_write_sample(WriteSampleRequest {
            topic: "test/topic".into(),
            route: ChannelRoute::for_topic("test/topic"),
            log_time: 0,
            publish_time: 0,
            payload: Payload::new(Bytes::from(vec![0_u8; 1024])),
            descriptor: Arc::clone(&descriptor),
        });
    }

    assert_eq!(writer.take_dropped_samples(), 4);
    assert_eq!(writer.finish().await.expect("finish"), 4 * 1024);
}
