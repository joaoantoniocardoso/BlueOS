//! MCAP chunk and message-index record parsing for the index walk.

use std::io::Read;

use blueos_idl::msg::blueos_recorder_msgs::{ChannelMessageCount, ChunkIndexEntry};

use super::stream::ReadDataStream;

/// Chunk record fields before the compression string (start/end time, uncompressed size, uncompressed CRC, name length).
pub(crate) const CHUNK_RECORD_PREFIX_BYTES: u64 = 8 + 8 + 8 + 4 + 4;
/// One message-index entry: log time and offset (`uint64` + `uint64`).
pub(crate) const MESSAGE_INDEX_ENTRY_BYTES: u64 = 16;

struct ChunkHeaderFields {
    message_start_time: u64,
    message_end_time: u64,
    uncompressed_size: u64,
    compression: String,
    compressed_size: u64,
}

pub(crate) fn parse_chunk_index_entry<R: Read>(
    recording: &mut R,
    record_start: u64,
    record_span: u64,
    payload_length: u64,
) -> Option<ChunkIndexEntry> {
    chunk_record_prefix_valid(payload_length)?;
    let mut stream = ReadDataStream {
        recording,
        count: 0,
    };
    let timing = read_chunk_timing_fields(&mut stream)?;
    let compression_length = stream.read4().ok()?;
    chunk_compression_length_valid(payload_length, compression_length)?;
    let compression = read_length_prefixed_utf8(&mut stream, compression_length as usize)?;
    let compressed_size = stream.read8().ok()?;
    chunk_compressed_body_valid(payload_length, stream.count, compressed_size)?;
    Some(chunk_index_entry_from_header(
        record_start,
        record_span,
        ChunkHeaderFields {
            message_start_time: timing.0,
            message_end_time: timing.1,
            uncompressed_size: timing.2,
            compression,
            compressed_size,
        },
    ))
}

fn chunk_record_prefix_valid(payload_length: u64) -> Option<()> {
    (payload_length >= CHUNK_RECORD_PREFIX_BYTES).then_some(())
}

fn chunk_compression_length_valid(payload_length: u64, compression_length: u32) -> Option<()> {
    (payload_length >= CHUNK_RECORD_PREFIX_BYTES + compression_length as u64).then_some(())
}

fn chunk_compressed_body_valid(
    payload_length: u64,
    bytes_read: usize,
    compressed_size: u64,
) -> Option<()> {
    (payload_length >= bytes_read as u64 + compressed_size).then_some(())
}

fn read_chunk_timing_fields<R: Read>(
    stream: &mut ReadDataStream<'_, R>,
) -> Option<(u64, u64, u64)> {
    let message_start_time = stream.read8().ok()?;
    let message_end_time = stream.read8().ok()?;
    let uncompressed_size = stream.read8().ok()?;
    stream.read4().ok()?;
    Some((message_start_time, message_end_time, uncompressed_size))
}

fn read_length_prefixed_utf8<R: Read>(
    stream: &mut ReadDataStream<'_, R>,
    length: usize,
) -> Option<String> {
    let compression_bytes = stream.read(length).ok()?;
    String::from_utf8(compression_bytes).ok()
}

fn chunk_index_entry_from_header(
    record_start: u64,
    record_span: u64,
    header: ChunkHeaderFields,
) -> ChunkIndexEntry {
    ChunkIndexEntry {
        start_time: header.message_start_time,
        end_time: header.message_end_time,
        offset: record_start,
        length: record_span,
        compression: header.compression,
        compressed_size: header.compressed_size,
        uncompressed_size: header.uncompressed_size,
        channel_ids: Vec::new(),
        message_index_length: 0,
    }
}

pub(crate) fn apply_message_index<R: Read>(
    recording: &mut R,
    current_chunk: &mut ChunkIndexEntry,
    record_span: u64,
    payload_length: u64,
    message_counts: &mut Vec<ChannelMessageCount>,
) -> bool {
    let Some((channel_id, entries_byte_length, header_bytes)) =
        read_message_index_header(recording)
    else {
        return false;
    };
    if payload_length < header_bytes as u64 {
        return false;
    }
    apply_message_index_counts(
        current_chunk,
        record_span,
        channel_id,
        entries_byte_length,
        message_counts,
    )
}

fn read_message_index_header<R: Read>(recording: &mut R) -> Option<(u16, u32, usize)> {
    let mut stream = ReadDataStream {
        recording,
        count: 0,
    };
    let channel_id = stream.read2().ok()?;
    let entries_byte_length = stream.read4().ok()?;
    Some((channel_id, entries_byte_length, stream.count))
}

fn apply_message_index_counts(
    current_chunk: &mut ChunkIndexEntry,
    record_span: u64,
    channel_id: u16,
    entries_byte_length: u32,
    message_counts: &mut Vec<ChannelMessageCount>,
) -> bool {
    if !current_chunk.channel_ids.contains(&channel_id) {
        current_chunk.channel_ids.push(channel_id);
    }
    current_chunk.message_index_length += record_span;
    let message_count = entries_byte_length as u64 / MESSAGE_INDEX_ENTRY_BYTES;
    merge_channel_message_count(message_counts, channel_id, message_count);
    true
}

fn merge_channel_message_count(
    message_counts: &mut Vec<ChannelMessageCount>,
    channel_id: u16,
    message_count: u64,
) {
    if let Some(existing) = message_counts
        .iter_mut()
        .find(|entry| entry.channel_id == channel_id)
    {
        existing.count += message_count;
        return;
    }
    message_counts.push(ChannelMessageCount {
        channel_id,
        count: message_count,
    });
}
