//! MCAP recording adapter: background writer thread, periodic flush, channel registration.

mod channel_descriptor;
mod writer;

pub use channel_descriptor::{
    ChannelDescriptor, MessageEncoding, SchemaEncoding, channel_descriptor_for_sample,
};
pub use writer::{
    DEFAULT_CHUNK_BYTES, DEFAULT_FLUSH_INTERVAL_SECS, McapCompression, McapSession,
    McapWriteConfig, McapWriterHandle,
};

#[cfg(test)]
mod tests {
    use blueos_comms::Payload;
    use bytes::Bytes;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn round_trip_mcap_file() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("test.mcap");
        let config = McapWriteConfig::default();
        let session = McapSession::open(&path, config).expect("open");
        let writer = session.writer();

        let lookup = |_name: &str| Some("string data\n".to_string());
        let descriptor = channel_descriptor_for_sample(
            "test/topic",
            "application/cdr;std_msgs/msg/String",
            &Payload::from_bytes(Bytes::from_static(b"\0\0\0\0")),
            &lookup,
            None,
        )
        .expect("descriptor");

        writer
            .write_message(
                "test/topic",
                1,
                1,
                Payload::from_bytes(Bytes::from_static(b"abcd")),
                Some(descriptor),
            )
            .expect("write");

        session.finish().expect("finish");

        let metadata = std::fs::metadata(&path).expect("metadata");
        assert!(metadata.len() > 100);
    }

    #[test]
    fn drop_without_explicit_finish_leaves_readable_summary() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("drop.mcap");
        let config = McapWriteConfig::default();

        {
            let session = McapSession::open(&path, config).expect("open");
            let writer = session.writer();
            let lookup = |_name: &str| Some("string data\n".to_string());
            let descriptor = channel_descriptor_for_sample(
                "test/topic",
                "application/cdr;std_msgs/msg/String",
                &Payload::from_bytes(Bytes::from_static(b"\0\0\0\0")),
                &lookup,
                None,
            )
            .expect("descriptor");
            writer
                .write_message(
                    "test/topic",
                    1,
                    1,
                    Payload::from_bytes(Bytes::from_static(b"abcd")),
                    Some(descriptor),
                )
                .expect("write");
        }

        let bytes = std::fs::read(&path).expect("read mcap");
        let summary = mcap::read::Summary::read(&bytes)
            .expect("parse summary")
            .expect("mcap summary section");
        assert!(!summary.channels.is_empty());

        let message_count = mcap::MessageStream::new(&bytes)
            .expect("message stream")
            .count();
        assert_eq!(message_count, 1);
    }
}
