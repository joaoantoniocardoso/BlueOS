//! Sequential MCAP record walk for paged index queries.

use std::io::{Read, Seek, SeekFrom};

use blueos_idl::msg::blueos_recorder_msgs::{
    ChannelMessageCount, ChunkIndexEntry, RecordingIndexResponse,
};

use crate::footer::MCAP_MAGIC;

use super::records::{apply_message_index, parse_chunk_index_entry};

const MAGIC_SIZE: usize = MCAP_MAGIC.len();
const RECORD_HEADER_SIZE: usize = 9;

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

struct RecordDispatch {
    opcode: u8,
    record_start: u64,
    payload_length: u64,
    record_end: u64,
    chunk_limit: u32,
}

pub(crate) struct McapIndexWalker {
    size: u64,
    offset: u64,
    closed: bool,
    chunks: Vec<ChunkIndexEntry>,
    message_counts: Vec<ChannelMessageCount>,
    records_bytes: Vec<u8>,
    current_chunk: Option<usize>,
}

impl McapIndexWalker {
    pub(super) fn new(size: u64, from_offset: u64) -> Self {
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

    pub(super) fn into_index(self) -> RecordingIndexResponse {
        RecordingIndexResponse {
            size: self.size,
            offset: self.offset,
            closed: self.closed,
            chunks: self.chunks,
            message_counts: self.message_counts,
            records: self.records_bytes,
        }
    }

    pub(super) fn validate_magic<R: Read + Seek>(
        &mut self,
        recording: &mut R,
    ) -> Result<(), super::IndexError> {
        recording.seek(SeekFrom::Start(0))?;
        let mut magic = [0_u8; MAGIC_SIZE];
        recording.read_exact(&mut magic)?;
        if magic != MCAP_MAGIC {
            return Err(super::IndexError::InvalidMagic);
        }
        self.offset = MAGIC_SIZE as u64;
        Ok(())
    }

    pub(super) fn walk_record<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        chunk_limit: u32,
    ) -> Result<bool, std::io::Error> {
        let bounds = self.read_record_bounds(recording)?;
        let Some((opcode, record_start, payload_length, record_end)) = bounds else {
            return Ok(false);
        };

        let dispatch = RecordDispatch {
            opcode,
            record_start,
            payload_length,
            record_end,
            chunk_limit,
        };
        self.dispatch_record(recording, dispatch)
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

    fn dispatch_record<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        dispatch: RecordDispatch,
    ) -> Result<bool, std::io::Error> {
        if dispatch.opcode == Opcode::DataEnd as u8 || dispatch.opcode == Opcode::Footer as u8 {
            return self.close_at_record_end(dispatch.record_end);
        }
        if is_summary_metadata_opcode(dispatch.opcode) {
            return self.copy_summary_record(recording, dispatch.record_start, dispatch.record_end);
        }
        if dispatch.opcode == Opcode::Chunk as u8 {
            return self.walk_chunk_record(recording, dispatch);
        }
        if dispatch.opcode == Opcode::MessageIndex as u8 {
            return self.walk_message_index_record(recording, dispatch);
        }
        self.skip_to_record_end(recording, dispatch.record_end)
    }

    fn close_at_record_end(&mut self, record_end: u64) -> Result<bool, std::io::Error> {
        self.offset = record_end;
        self.closed = true;
        Ok(false)
    }

    fn skip_to_record_end<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        record_end: u64,
    ) -> Result<bool, std::io::Error> {
        recording.seek(SeekFrom::Start(record_end))?;
        self.offset = record_end;
        Ok(true)
    }

    fn copy_summary_record<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        record_start: u64,
        record_end: u64,
    ) -> Result<bool, std::io::Error> {
        recording.seek(SeekFrom::Start(record_start))?;
        let mut record_bytes = vec![0_u8; (record_end - record_start) as usize];
        recording.read_exact(&mut record_bytes)?;
        self.records_bytes.extend_from_slice(&record_bytes);
        self.offset = record_end;
        Ok(true)
    }

    fn walk_chunk_record<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        dispatch: RecordDispatch,
    ) -> Result<bool, std::io::Error> {
        if self.chunks.len() >= dispatch.chunk_limit as usize {
            return Ok(false);
        }
        self.ingest_chunk_index_entry(recording, dispatch)
    }

    fn ingest_chunk_index_entry<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        dispatch: RecordDispatch,
    ) -> Result<bool, std::io::Error> {
        let Some(chunk_entry) = parse_chunk_index_entry(
            recording,
            dispatch.record_start,
            dispatch.record_end - dispatch.record_start,
            dispatch.payload_length,
        ) else {
            return Ok(false);
        };
        self.append_chunk_and_seek(recording, chunk_entry, dispatch.record_end)
    }

    fn append_chunk_and_seek<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        chunk_entry: ChunkIndexEntry,
        record_end: u64,
    ) -> Result<bool, std::io::Error> {
        self.chunks.push(chunk_entry);
        self.current_chunk = Some(self.chunks.len() - 1);
        recording.seek(SeekFrom::Start(record_end))?;
        self.offset = record_end;
        Ok(true)
    }

    fn walk_message_index_record<R: Read + Seek>(
        &mut self,
        recording: &mut R,
        dispatch: RecordDispatch,
    ) -> Result<bool, std::io::Error> {
        let applied = self.apply_message_index_for_current_chunk(
            recording,
            dispatch.record_start,
            dispatch.record_end,
            dispatch.payload_length,
        );
        if applied {
            self.skip_to_record_end(recording, dispatch.record_end)
        } else {
            Ok(false)
        }
    }

    fn apply_message_index_for_current_chunk<R: Read>(
        &mut self,
        recording: &mut R,
        record_start: u64,
        record_end: u64,
        payload_length: u64,
    ) -> bool {
        match self.current_chunk {
            None => true,
            Some(chunk_index) => apply_message_index(
                recording,
                &mut self.chunks[chunk_index],
                record_end - record_start,
                payload_length,
                &mut self.message_counts,
            ),
        }
    }
}

fn is_summary_metadata_opcode(opcode: u8) -> bool {
    matches!(
        opcode,
        opcode_value
            if opcode_value == Opcode::Header as u8
                || opcode_value == Opcode::Schema as u8
                || opcode_value == Opcode::Channel as u8
                || opcode_value == Opcode::Metadata as u8
    )
}
