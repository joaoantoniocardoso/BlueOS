use std::fs::{self, File};
use std::io::{BufWriter, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use blueos_recorder_mcap::footer::is_indexed;
use blueos_recorder_mcap::index::walk_index;
use blueos_recorder_mcap::rewrite::{rewrite, rewrite_from_reader};
use blueos_recorder_mcap::{McapSession, McapWriteConfig};
use mcap::{MessageStream, Writer, write::WriteOptions};
use tempfile::tempdir;

fn write_chunked_recording(path: &std::path::Path, message_count: usize) -> u64 {
    write_chunked_recording_with_options(
        path,
        message_count,
        WriteOptions::new().chunk_size(Some(256)),
    )
}

fn write_large_recording(path: &std::path::Path, message_count: usize) -> u64 {
    write_chunked_recording_with_options(
        path,
        message_count,
        WriteOptions::new()
            .compression(None)
            .chunk_size(Some(1024 * 1024)),
    )
}

fn write_chunked_recording_with_options(
    path: &std::path::Path,
    message_count: usize,
    options: WriteOptions,
) -> u64 {
    let file = fs::File::create(path).expect("create");
    let mut writer = Writer::with_options(BufWriter::new(file), options).expect("writer");
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
fn rewrite_truncated_mid_chunk_keeps_complete_chunk_messages() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("source.mcap");
    let output = directory.path().join("output.mcap");
    let expected_messages = write_chunked_recording(&source, 8);
    let bytes = fs::read(&source).expect("read");
    let index = walk_index(&source, 0, 100).expect("index");
    assert!(index.chunks.len() >= 2);
    let truncate_at = index.chunks[1].offset as usize + 10;
    fs::write(&source, &bytes[..truncate_at.min(bytes.len())]).expect("truncate");

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
    let config = McapWriteConfig::default();
    let session = McapSession::open(&source, config).expect("open");
    session.finish().expect("finish");
    let original = fs::read(&source).expect("read");

    let cancel = AtomicBool::new(true);
    let error = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect_err("cancelled");
    assert!(matches!(
        error,
        blueos_recorder_mcap::rewrite::RewriteError::Cancelled
    ));
    assert!(!output.exists());
    assert_eq!(fs::read(&source).expect("read"), original);
}

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

#[test]
fn rewrite_streams_source_without_whole_file_reads() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("large.mcap");
    let output = directory.path().join("output.mcap");
    let message_count = 6_000;
    write_large_recording(&source, message_count);
    let total_bytes = fs::metadata(&source).expect("metadata").len();
    assert!(total_bytes > 2 * 1024 * 1024);

    let max_read = Arc::new(AtomicUsize::new(0));
    let file = File::open(&source).expect("open");
    let tracker = MaxReadTracker {
        inner: file,
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
    assert!(total_bytes - summary.bytes_read <= 16);
    assert!(
        progress_offsets
            .iter()
            .any(|offset| *offset > 0 && *offset < total_bytes)
    );
    assert!(
        progress_offsets
            .windows(2)
            .any(|window| window[1] > window[0]),
        "progress offsets should advance while streaming"
    );
    assert!(max_read.load(Ordering::Relaxed) <= 1024 * 1024);
}

#[test]
fn rewrite_produces_indexed_file_with_messages() {
    let directory = tempdir().expect("tempdir");
    let source = directory.path().join("source.mcap");
    let output = directory.path().join("output.mcap");
    let expected_messages = write_chunked_recording(&source, 5);

    let cancel = AtomicBool::new(false);
    let summary = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect("rewrite");
    assert_eq!(summary.messages, expected_messages);
    assert!(is_indexed(&output));

    let recovered = fs::read(&output).expect("read output");
    let message_count = MessageStream::new(&recovered).expect("stream").count();
    assert_eq!(message_count as u64, expected_messages);
}
