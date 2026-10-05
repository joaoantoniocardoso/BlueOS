//! Integration tests for MCAP rewrite (repair data plane).

#[path = "support/rewrite_fixtures.rs"]
mod fixtures;

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::{fs, sync::Arc};

use mcap::MessageStream;
use tempfile::tempdir;

use blueos_recorder_mcap::{
    RewriteError, SOURCE_READ_BYTES, is_indexed, rewrite, rewrite_from_reader,
};

use fixtures::*;

#[test]
fn rewrite_truncated_file_keeps_complete_messages() {
    let paths = RewriteTempPaths::new("source.mcap", "output.mcap");
    let expected_messages = write_chunked_recording(&paths.source, 8);
    let bytes = fs::read(&paths.source).expect("read");
    fs::write(&paths.source, &bytes[..bytes.len() / 2]).expect("truncate");

    let cancel = AtomicBool::new(false);
    let summary = rewrite(
        &paths.source,
        &paths.output,
        &mut |_read, _total| {},
        &cancel,
    )
    .expect("rewrite");
    assert!(summary.messages > 0);
    assert!(summary.messages < expected_messages);
    assert!(is_indexed(&paths.output));
}

#[test]
fn rewrite_cancelled_leaves_source_unchanged_and_no_output() {
    let paths = RewriteTempPaths::new("source.mcap", "output.mcap");
    write_chunked_recording(&paths.source, 4);
    let original = fs::read(&paths.source).expect("read");

    let cancel = AtomicBool::new(true);
    let error = rewrite(
        &paths.source,
        &paths.output,
        &mut |_read, _total| {},
        &cancel,
    )
    .expect_err("cancelled");
    assert!(matches!(error, RewriteError::Cancelled));
    assert!(!paths.output.exists());
    assert_eq!(fs::read(&paths.source).expect("read"), original);
}

#[test]
fn rewrite_streams_source_without_whole_file_reads() {
    let paths = RewriteTempPaths::new("large.mcap", "output.mcap");
    let message_count = 6_000;
    write_large_uncompressed_recording(&paths.source, message_count);
    let total_bytes = fs::metadata(&paths.source).expect("metadata").len();
    assert!(total_bytes > 2 * 1024 * 1024);

    let max_read = Arc::new(AtomicUsize::new(0));
    let tracker = MaxReadTracker {
        inner: fs::File::open(&paths.source).expect("open"),
        max_read: Arc::clone(&max_read),
    };
    let mut progress_offsets = Vec::new();
    let cancel = AtomicBool::new(false);
    let summary = rewrite_from_reader(
        tracker,
        total_bytes,
        &paths.output,
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
    let paths = RewriteTempPaths::new("many_messages.mcap", "output.mcap");
    let message_count = 12_000;
    write_many_small_messages(&paths.source, message_count);
    let total_bytes = fs::metadata(&paths.source).expect("metadata").len();
    assert!(total_bytes > SOURCE_READ_BYTES as u64);

    let progress_calls = Arc::new(AtomicUsize::new(0));
    let cancel = AtomicBool::new(false);
    rewrite(
        &paths.source,
        &paths.output,
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
    let paths = RewriteTempPaths::new("growing.mcap", "snapshot.mcap");
    write_chunked_recording(&paths.source, 6);
    let size_at_start = fs::metadata(&paths.source).expect("metadata").len();
    append_growth_bytes(&paths.source, 4096);

    let cancel = AtomicBool::new(false);
    let summary = rewrite(
        &paths.source,
        &paths.output,
        &mut |_read, _total| {},
        &cancel,
    )
    .expect("rewrite");
    assert!(summary.messages > 0);
    assert!(summary.bytes_read <= size_at_start);
    assert!(is_indexed(&paths.output));
    let indexed_bytes = fs::read(&paths.output).expect("read output");
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
    copy_unfinished_lz4_recording(&writing, &source, 200);

    let cancel = AtomicBool::new(false);
    let summary = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect("rewrite");

    assert!(summary.messages > 0);
    assert!(summary.messages < 200);
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
    copy_unfinished_lz4_single_message(&writing, &source);

    let cancel = AtomicBool::new(false);
    let summary = rewrite(&source, &output, &mut |_read, _total| {}, &cancel).expect("rewrite");

    assert_eq!(summary.messages, 0);
    assert!(is_indexed(&output));
    let recovered = fs::read(&output).expect("read output");
    assert_eq!(MessageStream::new(&recovered).expect("stream").count(), 0);
}

#[test]
fn rewrite_of_a_file_that_is_not_an_mcap_fails_and_leaves_no_output() {
    let paths = RewriteTempPaths::new("source.mcap", "output.mcap");
    fs::write(&paths.source, b"not an MCAP recording").expect("write");

    let cancel = AtomicBool::new(false);
    let error = rewrite(
        &paths.source,
        &paths.output,
        &mut |_read, _total| {},
        &cancel,
    )
    .expect_err("not an MCAP");

    assert!(matches!(error, RewriteError::NotMcap));
    assert!(!paths.output.exists());
}

#[test]
fn rewrite_produces_indexed_file_with_messages() {
    let paths = RewriteTempPaths::new("source.mcap", "output.mcap");
    write_chunked_recording(&paths.source, 4);
    let cancel = AtomicBool::new(false);
    let summary = rewrite(
        &paths.source,
        &paths.output,
        &mut |_read, _total| {},
        &cancel,
    )
    .expect("rewrite");
    assert!(summary.messages > 0);
    assert!(is_indexed(&paths.output));
    let recovered = fs::read(&paths.output).expect("read output");
    let message_count = MessageStream::new(&recovered).expect("stream").count();
    assert_eq!(message_count, summary.messages as usize);
}
