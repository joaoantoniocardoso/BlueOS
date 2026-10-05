//! MCAP index walk adapter tests.

#[path = "support/index_walk_fixtures.rs"]
mod fixtures;

use core::{cell::Cell, sync::atomic::AtomicBool};
use std::{collections::HashSet, fs::File};

use tempfile::tempdir;

use blueos_recorder_mcap::{IndexError, MCAP_MAGIC, read_footer_at, walk_index, walk_index_reader};

use fixtures::*;

#[test]
fn read_footer_without_summary_offset_section() {
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("indexed.mcap");
    std::fs::write(&mcap_path, build_indexed_mcap(0)).expect("write");

    let size = std::fs::metadata(&mcap_path).expect("metadata").len();
    let footer = read_footer_at(&mcap_path, size).expect("read footer");
    assert!(footer.is_some());
    assert!(footer.expect("footer").summary_start > 0);
}

#[test]
fn walk_index_reports_size_seen_at_walk_start() {
    let mcap_bytes =
        build_chunked_mcap(&[(1_000, 2_000, "lz4", 50, vec![(1, 1)])], None, &[], None);
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("grow.mcap");
    std::fs::write(&mcap_path, &mcap_bytes).expect("write");
    let size_at_start = mcap_bytes.len() as u64;
    std::fs::write(&mcap_path, [mcap_bytes, vec![0_u8; 4096]].concat()).expect("grow");

    let mut recording = File::open(&mcap_path).expect("open");
    let index = walk_index_reader(
        &mut recording,
        size_at_start,
        0,
        2000,
        &AtomicBool::new(false),
    )
    .expect("walk");
    assert_eq!(index.size, size_at_start);
}

#[test]
fn walk_index_chunks_and_message_indexes() {
    let (schema, channel) = schema_and_channel_records();
    let mcap_bytes = build_chunked_mcap(
        &[
            (1_000, 2_000, "lz4", 100, vec![(1, 3), (2, 5)]),
            (3_000, 4_000, "zstd", 200, vec![(2, 2), (1, 1)]),
            (5_000, 6_000, "lz4", 150, vec![(1, 4)]),
        ],
        Some(&[schema.clone(), channel.clone()]),
        &[],
        None,
    );
    let file_size = mcap_bytes.len();
    let (_directory, mcap_path) = write_temp_mcap("chunked.mcap", &mcap_bytes);
    let index = walk_index(&mcap_path, 0, 2000, &AtomicBool::new(false)).expect("walk");
    assert_three_chunk_index(&index, &schema, &channel, file_size);
}

#[test]
fn walk_index_truncated_record_stops_at_start() {
    let complete = build_chunked_mcap(&[(1_000, 2_000, "lz4", 50, vec![(1, 1)])], None, &[], None);
    let second_chunk = mcap_record(0x06, &build_chunk_payload(3_000, 4_000, "lz4", 60, &[]));
    let truncated = [complete.as_slice(), &second_chunk[..15]].concat();
    let (_directory, mcap_path) = write_temp_mcap("truncated.mcap", &truncated);

    let index = walk_index(&mcap_path, 0, 2000, &AtomicBool::new(false)).expect("walk");
    assert_eq!(index.chunks.len(), 1);
    assert_eq!(index.offset, complete.len() as u64);
    assert!(!index.closed);
}

#[test]
fn walk_index_stops_at_a_chunk_still_being_written() {
    let complete = build_chunked_mcap(&[(1_000, 2_000, "lz4", 50, vec![(1, 1)])], None, &[], None);
    // The mcap writer puts `!0` in the record length and zeroes in the header until the chunk is flushed.
    let mut open_chunk = vec![0x06];
    open_chunk.extend_from_slice(&u64::MAX.to_le_bytes());
    open_chunk.extend_from_slice(&[0; 40]);
    let recording = [complete.as_slice(), &open_chunk].concat();
    let size = recording.len() as u64;

    let index = walk_index_reader(
        &mut std::io::Cursor::new(recording),
        size,
        0,
        2000,
        &AtomicBool::new(false),
    )
    .expect("walk");
    assert_eq!(index.chunks.len(), 1);
    assert_eq!(index.chunks[0].start_time, 1_000);
    assert_eq!(index.offset, complete.len() as u64);
    assert!(!index.closed);
}

#[test]
fn walk_index_resume_from_offset() {
    let mcap_bytes = mcap_bytes_for_resume_offset_walk();
    let (_directory, mcap_path) = write_temp_mcap("resume.mcap", &mcap_bytes);

    let first = walk_index(&mcap_path, 0, 1, &AtomicBool::new(false)).expect("walk");
    assert_eq!(first.chunks.len(), 1);
    assert_eq!(first.chunks[0].start_time, 1_000);

    let second = walk_index(&mcap_path, first.offset, 2000, &AtomicBool::new(false)).expect("walk");
    assert_eq!(second.chunks.len(), 2);
    assert_eq!(second.chunks[0].start_time, 3_000);
    assert_eq!(second.chunks[1].start_time, 5_000);
    assert_eq!(second.offset, mcap_bytes.len() as u64);
}

#[test]
fn walk_index_limit_caps_chunks() {
    let mcap_bytes = mcap_bytes_for_limited_chunk_walk();
    let (_directory, mcap_path) = write_temp_mcap("limited.mcap", &mcap_bytes);

    let first = walk_index(&mcap_path, 0, 2, &AtomicBool::new(false)).expect("walk");
    assert_eq!(first.chunks.len(), 2);
    let second = walk_index(&mcap_path, first.offset, 2, &AtomicBool::new(false)).expect("walk");
    assert_eq!(second.chunks.len(), 1);
    assert_eq!(second.offset, mcap_bytes.len() as u64);
}

#[test]
fn walk_index_paged_matches_single_walk() {
    let chunks: Vec<TestChunk> = (0..7)
        .map(|index| {
            (
                index as u64 * 1_000,
                index as u64 * 1_000 + 500,
                "lz4",
                50 + index as u64,
                vec![
                    (1, index as u32 + 1),
                    (2, index as u32 + 2),
                    (3, index as u32 + 3),
                ],
            )
        })
        .collect();
    let mcap_bytes = build_chunked_mcap(&chunks, None, &[], None);
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("paged.mcap");
    std::fs::write(&mcap_path, &mcap_bytes).expect("write");

    let single = walk_index(&mcap_path, 0, 20_000, &AtomicBool::new(false)).expect("walk");
    let pages = walk_mcap_index_pages(&mcap_path, 2);
    let (paged_chunks, paged_message_counts) = merge_paged_chunk_indexes(&pages);

    assert_eq!(paged_chunks, single.chunks);
    assert_eq!(paged_message_counts, message_counts_map(&single));

    let single_offsets: Vec<u64> = single.chunks.iter().map(|chunk| chunk.offset).collect();
    let paged_offsets: Vec<u64> = paged_chunks.iter().map(|chunk| chunk.offset).collect();
    assert_eq!(paged_offsets, single_offsets);
    assert_eq!(
        paged_offsets.len(),
        HashSet::<u64>::from_iter(paged_offsets.iter().copied()).len()
    );

    for page in &pages {
        if let Some(last_chunk) = page.chunks.last() {
            assert_eq!(last_chunk.channel_ids, vec![1, 2, 3]);
            assert!(last_chunk.message_index_length > 0);
        }
    }
}

#[test]
fn walk_index_data_end_sets_closed() {
    let mcap_bytes = build_chunked_mcap(
        &[(1_000, 2_000, "lz4", 50, vec![(1, 1)])],
        None,
        &mcap_record(0x0F, &[]),
        None,
    );
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("finalized.mcap");
    std::fs::write(&mcap_path, &mcap_bytes).expect("write");

    let index = walk_index(&mcap_path, 0, 2000, &AtomicBool::new(false)).expect("walk");
    assert!(index.closed);
    assert_eq!(index.offset, mcap_bytes.len() as u64);
}

#[test]
fn walk_index_skips_chunk_bodies() {
    let large_body = vec![b'x'; 2 * 1024 * 1024];
    let mcap_bytes = build_chunked_mcap(
        &[(1_000, 2_000, "lz4", large_body.len() as u64, vec![(1, 1)])],
        None,
        &[],
        Some(&large_body),
    );
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("large-chunk.mcap");
    std::fs::write(&mcap_path, &mcap_bytes).expect("write");

    let file = File::open(&mcap_path).expect("open");
    let file_size = file.metadata().expect("metadata").len();
    let mut counting = CountingReader {
        inner: file,
        bytes_read: Cell::new(0),
    };
    let index = walk_index_reader(&mut counting, file_size, 0, 2000, &AtomicBool::new(false))
        .expect("walk");
    assert_eq!(index.chunks.len(), 1);
    assert!(counting.bytes_read.get() < (file_size / 10) as usize);
}

#[test]
fn walk_index_with_cancel_flag_already_set_returns_cancelled() {
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("sample.mcap");
    std::fs::write(&mcap_path, MCAP_MAGIC).expect("write");
    let cancel = AtomicBool::new(true);
    let error = walk_index(&mcap_path, 0, 2000, &cancel).expect_err("walk");
    assert!(matches!(error, IndexError::Cancelled));
}
