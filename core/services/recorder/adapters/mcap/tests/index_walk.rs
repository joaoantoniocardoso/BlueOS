use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use blueos_idl::msg::blueos_recorder_msgs::{ChunkIndexEntry, RecordingIndex};
use blueos_recorder_mcap::footer::{MCAP_MAGIC, read_footer};
use blueos_recorder_mcap::index::{walk_index, walk_index_reader};
use tempfile::tempdir;

/// Start time, end time, compression, compressed size, and (channel id, message count) per message index.
type TestChunk<'a> = (u64, u64, &'a str, u64, Vec<(u16, u32)>);

fn concat_payload(parts: &[&[u8]]) -> Vec<u8> {
    let mut payload = Vec::new();
    for part in parts {
        payload.extend_from_slice(part);
    }
    payload
}

fn prefixed_string(value: &str) -> Vec<u8> {
    let encoded = value.as_bytes();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
    bytes.extend_from_slice(encoded);
    bytes
}

fn mcap_record(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![opcode];
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

fn build_indexed_mcap(summary_offset_start: u64) -> Vec<u8> {
    let header = mcap_record(
        0x01,
        &concat_payload(&[&prefixed_string(""), &prefixed_string("test")]),
    );
    let mut data = Vec::new();
    data.extend_from_slice(&MCAP_MAGIC);
    data.extend_from_slice(&header);
    let summary_start = data.len() as u64;
    let footer_payload = concat_payload(&[
        &summary_start.to_le_bytes(),
        &summary_offset_start.to_le_bytes(),
        &0u32.to_le_bytes(),
    ]);
    data.extend_from_slice(&mcap_record(0x02, &footer_payload));
    data.extend_from_slice(&MCAP_MAGIC);
    data
}

fn build_chunk_payload(
    start_time: u64,
    end_time: u64,
    compression: &str,
    records_size: u64,
    body_fill: &[u8],
) -> Vec<u8> {
    let body = if body_fill.len() == records_size as usize {
        body_fill.to_vec()
    } else {
        vec![0_u8; records_size as usize]
    };
    let mut payload = Vec::new();
    payload.extend_from_slice(&start_time.to_le_bytes());
    payload.extend_from_slice(&end_time.to_le_bytes());
    payload.extend_from_slice(&(records_size * 2).to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(&prefixed_string(compression));
    payload.extend_from_slice(&records_size.to_le_bytes());
    payload.extend_from_slice(&body);
    payload
}

fn build_message_index_payload(channel_id: u16, entry_count: u32) -> Vec<u8> {
    let entries_byte_length = entry_count * 16;
    let mut payload = Vec::new();
    payload.extend_from_slice(&channel_id.to_le_bytes());
    payload.extend_from_slice(&entries_byte_length.to_le_bytes());
    payload.resize(payload.len() + entries_byte_length as usize, 0);
    payload
}

fn build_chunked_mcap(
    chunks: &[TestChunk],
    leading_records: Option<&[Vec<u8>]>,
    trailing: &[u8],
    chunk_body_fill: Option<&[u8]>,
) -> Vec<u8> {
    let mut data = Vec::from(MCAP_MAGIC);
    if let Some(records) = leading_records {
        for record in records {
            data.extend_from_slice(record);
        }
    }
    for (start_time, end_time, compression, records_size, message_indexes) in chunks {
        let body_fill = chunk_body_fill.unwrap_or(&[]);
        let chunk_payload = build_chunk_payload(
            *start_time,
            *end_time,
            compression,
            *records_size,
            body_fill,
        );
        data.extend_from_slice(&mcap_record(0x06, &chunk_payload));
        for (channel_id, entry_count) in message_indexes {
            data.extend_from_slice(&mcap_record(
                0x07,
                &build_message_index_payload(*channel_id, *entry_count),
            ));
        }
    }
    data.extend_from_slice(trailing);
    data
}

fn walk_mcap_index_pages(mcap_path: &Path, page_limit: u32) -> Vec<RecordingIndex> {
    let mut pages = Vec::new();
    let mut offset = 0;
    loop {
        let page = walk_index(mcap_path, offset, page_limit).expect("walk");
        pages.push(page.clone());
        if page.closed || page.offset >= page.size {
            break;
        }
        if page.offset == offset {
            break;
        }
        offset = page.offset;
    }
    pages
}

fn merge_paged_chunk_indexes(
    pages: &[RecordingIndex],
) -> (Vec<ChunkIndexEntry>, HashMap<u16, u64>) {
    let mut chunks = Vec::new();
    let mut message_counts = HashMap::new();
    for page in pages {
        chunks.extend(page.chunks.clone());
        for count in &page.message_counts {
            message_counts
                .entry(count.channel_id)
                .and_modify(|total| *total += count.count)
                .or_insert(count.count);
        }
    }
    (chunks, message_counts)
}

fn message_counts_map(index: &RecordingIndex) -> HashMap<u16, u64> {
    index
        .message_counts
        .iter()
        .map(|entry| (entry.channel_id, entry.count))
        .collect()
}

struct CountingReader<R> {
    inner: R,
    bytes_read: Cell<usize>,
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.bytes_read.set(self.bytes_read.get() + read);
        Ok(read)
    }
}

impl<R: Seek> Seek for CountingReader<R> {
    fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(position)
    }
}

#[test]
fn read_footer_without_summary_offset_section() {
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("indexed.mcap");
    std::fs::write(&mcap_path, build_indexed_mcap(0)).expect("write");

    let footer = read_footer(&mcap_path).expect("read footer");
    assert!(footer.is_some());
    assert!(footer.expect("footer").summary_start > 0);
    assert!(blueos_recorder_mcap::footer::is_indexed(&mcap_path));
}

#[test]
fn walk_index_chunks_and_message_indexes() {
    let schema = mcap_record(
        0x03,
        &concat_payload(&[
            &prefixed_string("schema"),
            &prefixed_string("encoding"),
            b"data",
        ]),
    );
    let channel = mcap_record(
        0x04,
        &concat_payload(&[
            &(1u16).to_le_bytes(),
            &prefixed_string("topic"),
            &prefixed_string("encoding"),
            &1u32.to_le_bytes(),
        ]),
    );
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
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("chunked.mcap");
    std::fs::write(&mcap_path, &mcap_bytes).expect("write");

    let index = walk_index(&mcap_path, 0, 2000).expect("walk");
    assert_eq!(index.size, file_size as u64);
    assert_eq!(index.offset, file_size as u64);
    assert!(!index.closed);
    assert_eq!(index.chunks.len(), 3);
    assert_eq!(index.chunks[0].start_time, 1_000);
    assert_eq!(index.chunks[0].end_time, 2_000);
    assert_eq!(index.chunks[0].compression, "lz4");
    assert_eq!(index.chunks[0].compressed_size, 100);
    assert_eq!(index.chunks[0].uncompressed_size, 200);
    assert_eq!(index.chunks[0].channel_ids, vec![1, 2]);
    let first_message_index_length = (9 + build_message_index_payload(1, 3).len()) as u64
        + (9 + build_message_index_payload(2, 5).len()) as u64;
    assert_eq!(
        index.chunks[0].message_index_length,
        first_message_index_length
    );
    assert_eq!(
        index.chunks[0].offset,
        (MCAP_MAGIC.len() + schema.len() + channel.len()) as u64
    );
    assert_eq!(
        index.chunks[0].length,
        (9 + build_chunk_payload(1_000, 2_000, "lz4", 100, &[]).len()) as u64
    );
    assert_eq!(message_counts_map(&index), HashMap::from([(1, 8), (2, 7)]));
    let mut expected_records = schema;
    expected_records.extend_from_slice(&channel);
    assert_eq!(index.records, expected_records);
}

#[test]
fn walk_index_truncated_record_stops_at_start() {
    let complete = build_chunked_mcap(&[(1_000, 2_000, "lz4", 50, vec![(1, 1)])], None, &[], None);
    let second_chunk = mcap_record(0x06, &build_chunk_payload(3_000, 4_000, "lz4", 60, &[]));
    let truncated = [complete.as_slice(), &second_chunk[..15]].concat();
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("truncated.mcap");
    std::fs::write(&mcap_path, truncated).expect("write");

    let index = walk_index(&mcap_path, 0, 2000).expect("walk");
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

    let index =
        walk_index_reader(&mut std::io::Cursor::new(recording), size, 0, 2000).expect("walk");
    assert_eq!(index.chunks.len(), 1);
    assert_eq!(index.chunks[0].start_time, 1_000);
    assert_eq!(index.offset, complete.len() as u64);
    assert!(!index.closed);
}

#[test]
fn walk_index_resume_from_offset() {
    let mcap_bytes = build_chunked_mcap(
        &[
            (1_000, 2_000, "lz4", 50, vec![(1, 1)]),
            (3_000, 4_000, "lz4", 60, vec![(2, 2)]),
            (5_000, 6_000, "lz4", 70, vec![(1, 3)]),
        ],
        None,
        &[],
        None,
    );
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("resume.mcap");
    std::fs::write(&mcap_path, &mcap_bytes).expect("write");

    let first = walk_index(&mcap_path, 0, 1).expect("walk");
    assert_eq!(first.chunks.len(), 1);
    assert_eq!(first.chunks[0].start_time, 1_000);

    let second = walk_index(&mcap_path, first.offset, 2000).expect("walk");
    assert_eq!(second.chunks.len(), 2);
    assert_eq!(second.chunks[0].start_time, 3_000);
    assert_eq!(second.chunks[1].start_time, 5_000);
    assert_eq!(second.offset, mcap_bytes.len() as u64);
}

#[test]
fn walk_index_limit_caps_chunks() {
    let mcap_bytes = build_chunked_mcap(
        &[
            (1_000, 2_000, "lz4", 40, vec![(1, 1)]),
            (3_000, 4_000, "lz4", 40, vec![(1, 1)]),
            (5_000, 6_000, "lz4", 40, vec![(1, 1)]),
        ],
        None,
        &[],
        None,
    );
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join("limited.mcap");
    std::fs::write(&mcap_path, &mcap_bytes).expect("write");

    let first = walk_index(&mcap_path, 0, 2).expect("walk");
    assert_eq!(first.chunks.len(), 2);
    let second = walk_index(&mcap_path, first.offset, 2).expect("walk");
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

    let single = walk_index(&mcap_path, 0, 20_000).expect("walk");
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

    let index = walk_index(&mcap_path, 0, 2000).expect("walk");
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
    let index = walk_index_reader(&mut counting, file_size, 0, 2000).expect("walk");
    assert_eq!(index.chunks.len(), 1);
    assert!(counting.bytes_read.get() < (file_size / 10) as usize);
}
