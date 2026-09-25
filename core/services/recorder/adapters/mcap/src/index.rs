use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use blueos_idl::msg::blueos_recorder_msgs::{ChannelMessageCount, ChunkIndexEntry, RecordingIndex};
use thiserror::Error;

use crate::footer::MCAP_MAGIC;

const MAGIC_SIZE: usize = MCAP_MAGIC.len();
const RECORD_HEADER_SIZE: usize = 9;
const MIN_LIMIT: u32 = 1;
const MAX_LIMIT: u32 = 20_000;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("Invalid MCAP magic.")]
    InvalidMagic,
    #[error("Index page limit must be between {MIN_LIMIT} and {MAX_LIMIT}.")]
    LimitOutOfRange,
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Opcode {
    Header = 0x01,
    Footer = 0x02,
    Schema = 0x03,
    Channel = 0x04,
    Chunk = 0x06,
    MessageIndex = 0x07,
    Metadata = 0x0C,
    DataEnd = 0x0F,
}

struct EndOfFile;

struct ReadDataStream<'a, R: Read> {
    recording: &'a mut R,
    count: usize,
}

impl<'a, R: Read> ReadDataStream<'a, R> {
    fn read(&mut self, length: usize) -> Result<Vec<u8>, EndOfFile> {
        if length == 0 {
            return Ok(Vec::new());
        }
        let mut buffer = vec![0_u8; length];
        let read = match self.recording.read(&mut buffer) {
            Ok(value) => value,
            Err(_) => return Err(EndOfFile),
        };
        self.count += read;
        if read == 0 || read < length {
            Err(EndOfFile)
        } else {
            Ok(buffer)
        }
    }

    fn read2(&mut self) -> Result<u16, EndOfFile> {
        let bytes = self.read(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read4(&mut self) -> Result<u32, EndOfFile> {
        let bytes = self.read(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read8(&mut self) -> Result<u64, EndOfFile> {
        let bytes = self.read(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

struct McapIndexWalker {
    size: u64,
    offset: u64,
    closed: bool,
    chunks: Vec<ChunkIndexEntry>,
    message_counts: Vec<ChannelMessageCount>,
    records_bytes: Vec<u8>,
    current_chunk: Option<usize>,
}

impl McapIndexWalker {
    fn new(size: u64, from_offset: u64) -> Self {
        Self {
            size,
            offset: from_offset,
            closed: false,
            chunks: Vec::new(),
            message_counts: Vec::new(),
            records_bytes: Vec::new(),
            current_chunk: None,
        }
    }

    fn into_index(self) -> RecordingIndex {
        RecordingIndex {
            size: self.size,
            offset: self.offset,
            closed: self.closed,
            chunks: self.chunks,
            message_counts: self.message_counts,
            records: self.records_bytes,
        }
    }

    fn validate_magic<R: Read + Seek>(&mut self, recording: &mut R) -> Result<(), IndexError> {
        recording.seek(SeekFrom::Start(0))?;
        let mut magic = [0_u8; MAGIC_SIZE];
        recording.read_exact(&mut magic)?;
        if magic != MCAP_MAGIC {
            return Err(IndexError::InvalidMagic);
        }
        self.offset = MAGIC_SIZE as u64;
        Ok(())
    }

    fn read_record_bounds<R: Read + Seek>(
        &self,
        recording: &mut R,
    ) -> Result<Option<(u8, u64, u64, u64)>, std::io::Error> {
        if self.size - self.offset < RECORD_HEADER_SIZE as u64 {
            return Ok(None);
        }
        let record_start = self.offset;
        recording.seek(SeekFrom::Start(record_start))?;
        let mut header = [0_u8; RECORD_HEADER_SIZE];
        recording.read_exact(&mut header)?;
        let payload_length = u64::from_le_bytes([
            header[1], header[2], header[3], header[4], header[5], header[6], header[7], header[8],
        ]);
        // A chunk still being written has `!0` as its length, so the span must not wrap around.
        let Some(record_end) = payload_length
            .checked_add(RECORD_HEADER_SIZE as u64)
            .and_then(|record_span| record_start.checked_add(record_span))
        else {
            return Ok(None);
        };
        if record_end > self.size {
            return Ok(None);
        }
        let opcode = header[0];
        if payload_length == 0 && opcode != Opcode::DataEnd as u8 && opcode != Opcode::Footer as u8
        {
            return Ok(None);
        }
        Ok(Some((opcode, record_start, payload_length, record_end)))
    }

    fn walk_record<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        chunk_limit: u32,
    ) -> Result<bool, std::io::Error> {
        let bounds = self.read_record_bounds(recording)?;
        let Some((opcode, record_start, payload_length, record_end)) = bounds else {
            return Ok(false);
        };

        let mut continue_walk = true;

        if opcode == Opcode::DataEnd as u8 || opcode == Opcode::Footer as u8 {
            self.offset = record_end;
            self.closed = true;
            continue_walk = false;
        } else if matches!(
            opcode,
            x if x == Opcode::Header as u8
                || x == Opcode::Schema as u8
                || x == Opcode::Channel as u8
                || x == Opcode::Metadata as u8
        ) {
            recording.seek(SeekFrom::Start(record_start))?;
            let mut record_bytes = vec![0_u8; (record_end - record_start) as usize];
            recording.read_exact(&mut record_bytes)?;
            self.records_bytes.extend_from_slice(&record_bytes);
            self.offset = record_end;
        } else if opcode == Opcode::Chunk as u8 && self.chunks.len() >= chunk_limit as usize {
            continue_walk = false;
        } else if opcode == Opcode::Chunk as u8 {
            match parse_chunk_index_entry(
                recording,
                record_start,
                record_end - record_start,
                payload_length,
            ) {
                Some(chunk_entry) => {
                    self.chunks.push(chunk_entry);
                    self.current_chunk = Some(self.chunks.len() - 1);
                    recording.seek(SeekFrom::Start(record_end))?;
                    self.offset = record_end;
                }
                None => continue_walk = false,
            }
        } else if opcode == Opcode::MessageIndex as u8 {
            let applied = match self.current_chunk {
                None => true,
                Some(chunk_index) => apply_message_index(
                    recording,
                    &mut self.chunks[chunk_index],
                    record_end - record_start,
                    payload_length,
                    &mut self.message_counts,
                ),
            };
            if applied {
                recording.seek(SeekFrom::Start(record_end))?;
                self.offset = record_end;
            } else {
                continue_walk = false;
            }
        } else {
            recording.seek(SeekFrom::Start(record_end))?;
            self.offset = record_end;
        }

        Ok(continue_walk)
    }
}

fn parse_chunk_index_entry<R: Read>(
    recording: &mut R,
    record_start: u64,
    record_span: u64,
    payload_length: u64,
) -> Option<ChunkIndexEntry> {
    if payload_length < 40 {
        return None;
    }
    let mut stream = ReadDataStream {
        recording,
        count: 0,
    };
    let message_start_time = stream.read8().ok()?;
    let message_end_time = stream.read8().ok()?;
    let uncompressed_size = stream.read8().ok()?;
    if stream.read4().is_err() {
        return None;
    }
    let compression_length = stream.read4().ok()?;
    if payload_length < 40 + compression_length as u64 {
        return None;
    }
    let compression_bytes = stream.read(compression_length as usize).ok()?;
    let compression = String::from_utf8(compression_bytes).ok()?;
    let compressed_size = stream.read8().ok()?;
    if payload_length < stream.count as u64 + compressed_size {
        return None;
    }
    Some(ChunkIndexEntry {
        start_time: message_start_time,
        end_time: message_end_time,
        offset: record_start,
        length: record_span,
        compression,
        compressed_size,
        uncompressed_size,
        channel_ids: Vec::new(),
        message_index_length: 0,
    })
}

fn apply_message_index<R: Read>(
    recording: &mut R,
    current_chunk: &mut ChunkIndexEntry,
    record_span: u64,
    payload_length: u64,
    message_counts: &mut Vec<ChannelMessageCount>,
) -> bool {
    let mut stream = ReadDataStream {
        recording,
        count: 0,
    };
    let channel_id = match stream.read2() {
        Ok(value) => value,
        Err(EndOfFile) => return false,
    };
    let entries_byte_length = match stream.read4() {
        Ok(value) => value,
        Err(EndOfFile) => return false,
    };
    if payload_length < stream.count as u64 {
        return false;
    }
    if !current_chunk.channel_ids.contains(&channel_id) {
        current_chunk.channel_ids.push(channel_id);
    }
    current_chunk.message_index_length += record_span;
    let message_count = entries_byte_length as u64 / 16;
    if let Some(existing) = message_counts
        .iter_mut()
        .find(|entry| entry.channel_id == channel_id)
    {
        existing.count += message_count;
    } else {
        message_counts.push(ChannelMessageCount {
            channel_id,
            count: message_count,
        });
    }
    true
}

pub fn walk_index(path: &Path, from_offset: u64, limit: u32) -> Result<RecordingIndex, IndexError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(IndexError::LimitOutOfRange);
    }
    let size = std::fs::metadata(path)?.len();
    let mut recording = File::open(path)?;
    walk_index_reader(&mut recording, size, from_offset, limit)
}

pub fn walk_index_reader<R: Read + Seek>(
    recording: &mut R,
    size: u64,
    from_offset: u64,
    limit: u32,
) -> Result<RecordingIndex, IndexError> {
    if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        return Err(IndexError::LimitOutOfRange);
    }
    let mut walker = McapIndexWalker::new(size, from_offset);
    if from_offset == 0 {
        walker.validate_magic(recording)?;
    }
    while walker.walk_record(recording, limit)? {}
    Ok(walker.into_index())
}
