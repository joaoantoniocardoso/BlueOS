//! Integration tests for MCAP rewrite (repair data plane).

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::{
    fs::{self, File},
    io::{BufWriter, Read, Write},
    sync::Arc,
};

use mcap::{Compression, MessageStream, Writer, write::WriteOptions};
use tempfile::tempdir;

use blueos_recorder_mcap::{
    RewriteError, SOURCE_READ_BYTES, is_indexed, rewrite, rewrite_from_reader,
};

struct MaxReadTracker<R> {
    inner: R,
    max_read: Arc<AtomicUsize>,
}

impl<R: Read> Read for MaxReadTracker<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.max_read.fetch_max(read, Ordering::Relaxed);
        Ok(read)
    }
}

fn write_chunked_recording(path: &std::path::Path, message_count: usize) -> u64 {
    let file = fs::File::create(path).expect("create");
    let mut writer = Writer::with_options(
        BufWriter::new(file),
        WriteOptions::new().chunk_size(Some(256)),
    )
    .expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    for message_index in 0..message_count {
        let header = mcap::records::MessageHeader {
            channel_id,
            sequence: message_index as u32,
            log_time: message_index as u64,
            publish_time: message_index as u64,
        };
        let payload = vec![0_u8; 1024];
        writer
            .write_to_known_channel(&header, &payload)
            .expect("write");
    }
    writer.finish().expect("finish");
    message_count as u64
}

#[test]
fn rewrite_truncated_file_keeps_complete_messages() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("source.mcap");
    let output = directory.path().join("output.mcap");
    let expected_messages = write_chunked_recording(&source, 8);
    let bytes = fs::read(&source).expect("read");
    let truncate_at = bytes.len() / 2;
    fs::write(&source, &bytes[..truncate_at]).expect("truncate");

    let cancel = AtomicBool::new(false);
    let summary = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect("rewrite");
    assert!(summary.messages > 0);
    assert!(summary.messages < expected_messages);
    assert!(is_indexed(&output));
}

#[test]
fn rewrite_cancelled_leaves_source_unchanged_and_no_output() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("source.mcap");
    let output = directory.path().join("output.mcap");
    write_chunked_recording(&source, 4);
    let original = fs::read(&source).expect("read");

    let cancel = AtomicBool::new(true);
    let error = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect_err("cancelled");
    assert!(matches!(error, RewriteError::Cancelled));
    assert!(!output.exists());
    assert_eq!(fs::read(&source).expect("read"), original);
}

#[test]
fn rewrite_streams_source_without_whole_file_reads() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("large.mcap");
    let output = directory.path().join("output.mcap");
    let message_count = 6_000;
    let file = fs::File::create(&source).expect("create");
    let mut writer = Writer::with_options(
        BufWriter::new(file),
        WriteOptions::new()
            .compression(None)
            .chunk_size(Some(1024 * 1024)),
    )
    .expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    for message_index in 0..message_count {
        let header = mcap::records::MessageHeader {
            channel_id,
            sequence: message_index as u32,
            log_time: message_index as u64,
            publish_time: message_index as u64,
        };
        let payload = vec![0_u8; 1024];
        writer
            .write_to_known_channel(&header, &payload)
            .expect("write");
    }
    writer.finish().expect("finish");
    let total_bytes = fs::metadata(&source).expect("metadata").len();
    assert!(total_bytes > 2 * 1024 * 1024);

    let max_read = Arc::new(AtomicUsize::new(0));
    let reader = File::open(&source).expect("open");
    let tracker = MaxReadTracker {
        inner: reader,
        max_read: Arc::clone(&max_read),
    };
    let mut progress_offsets = Vec::new();
    let cancel = AtomicBool::new(false);
    let summary = rewrite_from_reader(
        tracker,
        total_bytes,
        &output,
        &mut |bytes_read, _total| progress_offsets.push(bytes_read),
        &cancel,
    )
    .expect("rewrite");

    assert!(summary.messages > 0);
    assert!(summary.bytes_read > 0);
    assert!(summary.bytes_read <= total_bytes);
    assert!(
        progress_offsets
            .iter()
            .any(|offset| *offset > 0 && *offset < total_bytes)
    );
    assert!(max_read.load(Ordering::Relaxed) < total_bytes as usize);
}

#[test]
fn rewrite_progress_fires_per_read_chunk_not_per_message() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("many_messages.mcap");
    let output = directory.path().join("output.mcap");
    let message_count = 12_000;
    let file = fs::File::create(&source).expect("create");
    let mut writer = Writer::with_options(
        BufWriter::new(file),
        WriteOptions::new().compression(None).chunk_size(Some(256)),
    )
    .expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    for message_index in 0..message_count {
        let header = mcap::records::MessageHeader {
            channel_id,
            sequence: message_index as u32,
            log_time: message_index as u64,
            publish_time: message_index as u64,
        };
        let payload = vec![0_u8; 512];
        writer
            .write_to_known_channel(&header, &payload)
            .expect("write");
    }
    writer.finish().expect("finish");
    let total_bytes = fs::metadata(&source).expect("metadata").len();
    assert!(total_bytes > SOURCE_READ_BYTES as u64);

    let progress_calls = Arc::new(AtomicUsize::new(0));
    let cancel = AtomicBool::new(false);
    rewrite(
        &source,
        &output,
        &mut |_read, _total| {
            progress_calls.fetch_add(1, Ordering::Relaxed);
        },
        &cancel,
    )
    .expect("rewrite");

    let calls = progress_calls.load(Ordering::Relaxed);
    let chunk_steps = total_bytes.div_ceil(SOURCE_READ_BYTES as u64) as usize;
    assert!(calls <= chunk_steps + 1);
    assert!(calls < message_count);
}

#[test]
fn rewrite_ignores_bytes_appended_after_size_at_start() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("growing.mcap");
    let output = directory.path().join("snapshot.mcap");
    write_chunked_recording(&source, 6);
    let size_at_start = fs::metadata(&source).expect("metadata").len();
    fs::OpenOptions::new()
        .append(true)
        .open(&source)
        .expect("append")
        .write_all(&[0xFF; 4096])
        .expect("grow");

    let cancel = AtomicBool::new(false);
    let summary = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect("rewrite");
    assert!(summary.messages > 0);
    assert!(summary.bytes_read <= size_at_start);
    assert!(is_indexed(&output));
    let indexed_bytes = fs::read(&output).expect("read output");
    let summary_read = mcap::Summary::read(&indexed_bytes)
        .expect("summary read")
        .expect("summary");
    assert!(!summary_read.chunk_indexes.is_empty());
    let message_count = mcap::MessageStream::new(&indexed_bytes)
        .expect("stream")
        .count();
    assert_eq!(message_count, summary.messages as usize);
}

#[test]
fn rewrite_keeps_the_messages_of_the_chunk_its_writer_left_open() {
    let directory = tempdir().expect("tempdir");
    let writing = directory.path().join("writing.mcap");
    let source = directory.path().join("source.mcap");
    let output = directory.path().join("output.mcap");
    let message_count = 200;
    let mut writer = Writer::with_options(
        fs::File::create(&writing).expect("create"),
        WriteOptions::new().compression(Some(Compression::Lz4)),
    )
    .expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    for message_index in 0..message_count {
        let header = mcap::records::MessageHeader {
            channel_id,
            sequence: message_index,
            log_time: message_index.into(),
            publish_time: message_index.into(),
        };
        writer
            .write_to_known_channel(&header, &[0_u8; 1024])
            .expect("write");
    }
    fs::copy(&writing, &source).expect("copy the recording as its writer left it");
    drop(writer);

    let cancel = AtomicBool::new(false);
    let summary = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect("rewrite");

    assert!(summary.messages > 0);
    assert!(summary.messages < u64::from(message_count));
    assert!(is_indexed(&output));
    let recovered = fs::read(&output).expect("read output");
    let sequences: Vec<u32> = MessageStream::new(&recovered)
        .expect("stream")
        .map(|message| message.expect("message").sequence)
        .collect();
    assert_eq!(
        sequences,
        (0..).take(sequences.len()).collect::<Vec<u32>>(),
        "the rewrite keeps every message up to the end of what the writer put on disk, in order"
    );
    assert_eq!(sequences.len() as u64, summary.messages);
}

#[test]
fn rewrite_of_a_recording_that_ends_before_its_first_message_is_an_empty_recording() {
    let directory = tempdir().expect("tempdir");
    let writing = directory.path().join("writing.mcap");
    let source = directory.path().join("source.mcap");
    let output = directory.path().join("output.mcap");
    let mut writer = Writer::with_options(
        fs::File::create(&writing).expect("create"),
        WriteOptions::new().compression(Some(Compression::Lz4)),
    )
    .expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    let header = mcap::records::MessageHeader {
        channel_id,
        sequence: 0,
        log_time: 0,
        publish_time: 0,
    };
    writer
        .write_to_known_channel(&header, b"payload")
        .expect("write");
    fs::copy(&writing, &source).expect("copy the recording as its writer left it");
    drop(writer);

    let cancel = AtomicBool::new(false);
    let summary = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect("rewrite");

    assert_eq!(summary.messages, 0);
    assert!(is_indexed(&output));
    let recovered = fs::read(&output).expect("read output");
    assert_eq!(MessageStream::new(&recovered).expect("stream").count(), 0);
}

#[test]
fn rewrite_of_a_file_that_is_not_an_mcap_fails_and_leaves_no_output() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("source.mcap");
    let output = directory.path().join("output.mcap");
    fs::write(&source, b"not an MCAP recording").expect("write");

    let cancel = AtomicBool::new(false);
    let error =
        rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect_err("not an MCAP");

    assert!(matches!(error, RewriteError::Mcap(_)));
    assert!(!output.exists());
}

#[test]
fn rewrite_produces_indexed_file_with_messages() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("source.mcap");
    let output = directory.path().join("output.mcap");
    write_chunked_recording(&source, 4);
    let cancel = AtomicBool::new(false);
    let summary = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect("rewrite");
    assert!(summary.messages > 0);
    assert!(is_indexed(&output));
    let recovered = fs::read(&output).expect("read output");
    let message_count = MessageStream::new(&recovered).expect("stream").count();
    assert_eq!(message_count, summary.messages as usize);
}
