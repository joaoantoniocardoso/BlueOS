//! MCAP index walk test fixtures.

#![expect(unreachable_pub, reason = "shared index walk test fixtures")]

use core::{cell::Cell, sync::atomic::AtomicBool};
use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use tempfile::tempdir;

use blueos_idl::msg::blueos_recorder_msgs::{ChunkIndexEntry, RecordingIndexResponse};
use blueos_recorder_mcap::{MCAP_MAGIC, walk_index};

/// Start time, end time, compression, compressed size, and (channel id, message count) per message index.
pub type TestChunk<'a> = (u64, u64, &'a str, u64, Vec<(u16, u32)>);

pub struct CountingReader<R> {
    pub inner: R,
    pub bytes_read: Cell<usize>,
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

pub fn walk_mcap_index_pages(mcap_path: &Path, page_limit: u32) -> Vec<RecordingIndexResponse> {
    let mut pages = Vec::new();
    let mut offset = 0;
    loop {
        let page =
            walk_index(mcap_path, offset, page_limit, &AtomicBool::new(false)).expect("walk");
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

pub fn merge_paged_chunk_indexes(
    pages: &[RecordingIndexResponse],
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

pub fn write_temp_mcap(name: &str, bytes: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempdir().expect("tempdir");
    let mcap_path = directory.path().join(name);
    std::fs::write(&mcap_path, bytes).expect("write");
    (directory, mcap_path)
}

pub fn assert_three_chunk_index(
    index: &RecordingIndexResponse,
    schema: &[u8],
    channel: &[u8],
    file_size: usize,
) {
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
    assert_eq!(message_counts_map(index), HashMap::from([(1, 8), (2, 7)]));
    let mut expected_records = schema.to_vec();
    expected_records.extend_from_slice(channel);
    assert_eq!(index.records, expected_records);
}

pub fn mcap_bytes_for_resume_offset_walk() -> Vec<u8> {
    build_chunked_mcap(
        &[
            (1_000, 2_000, "lz4", 50, vec![(1, 1)]),
            (3_000, 4_000, "lz4", 60, vec![(2, 2)]),
            (5_000, 6_000, "lz4", 70, vec![(1, 3)]),
        ],
        None,
        &[],
        None,
    )
}

pub fn mcap_bytes_for_limited_chunk_walk() -> Vec<u8> {
    build_chunked_mcap(
        &[
            (1_000, 2_000, "lz4", 40, vec![(1, 1)]),
            (3_000, 4_000, "lz4", 40, vec![(1, 1)]),
            (5_000, 6_000, "lz4", 40, vec![(1, 1)]),
        ],
        None,
        &[],
        None,
    )
}

pub fn build_indexed_mcap(summary_offset_start: u64) -> Vec<u8> {
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

pub fn build_chunked_mcap(
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

pub fn schema_and_channel_records() -> (Vec<u8>, Vec<u8>) {
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
    (schema, channel)
}

pub fn build_chunk_payload(
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

pub fn build_message_index_payload(channel_id: u16, entry_count: u32) -> Vec<u8> {
    let entries_byte_length = entry_count * 16;
    let mut payload = Vec::new();
    payload.extend_from_slice(&channel_id.to_le_bytes());
    payload.extend_from_slice(&entries_byte_length.to_le_bytes());
    payload.resize(payload.len() + entries_byte_length as usize, 0);
    payload
}

pub fn message_counts_map(index: &RecordingIndexResponse) -> HashMap<u16, u64> {
    index
        .message_counts
        .iter()
        .map(|entry| (entry.channel_id, entry.count))
        .collect()
}

pub fn mcap_record(opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![opcode];
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

pub fn concat_payload(parts: &[&[u8]]) -> Vec<u8> {
    let mut payload = Vec::new();
    for part in parts {
        payload.extend_from_slice(part);
    }
    payload
}

pub fn prefixed_string(value: &str) -> Vec<u8> {
    let encoded = value.as_bytes();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
    bytes.extend_from_slice(encoded);
    bytes
}
