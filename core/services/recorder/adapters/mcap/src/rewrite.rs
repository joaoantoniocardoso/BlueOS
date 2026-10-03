//! Rewrites a truncated MCAP into a seekable file (in-process repair).

use core::sync::atomic::{AtomicBool, Ordering};
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read},
    path::Path,
};

use mcap::{
    Attachment, Compression, MAGIC, McapError, WriteOptions, Writer, parse_record,
    records::{Record, op},
    sans_io::linear_reader::{LinearReadEvent, LinearReader, LinearReaderOptions},
};
use thiserror::Error;
use tracing::debug;

/// Maximum bytes read from the source per `read` call (repair progress granularity).
pub const SOURCE_READ_BYTES: usize = 1024 * 1024;
/// Bytes of a record's opcode and length.
const RECORD_PREFIX_BYTES: usize = 1 + 8;
/// Bytes of a chunk header before its compression name: start and end time, uncompressed size and CRC, and the
/// name's length.
const CHUNK_HEADER_BEFORE_COMPRESSION_BYTES: usize = 8 + 8 + 8 + 4 + 4;
/// The record and compressed length the `mcap` writer gives a chunk until it closes it.
const OPEN_CHUNK_LENGTH: u64 = u64::MAX;

/// Why a rewrite stopped before producing a file.
#[derive(Debug, Error)]
pub enum RewriteError {
    /// The caller asked to stop.
    #[error("rewrite cancelled")]
    Cancelled,
    /// The source does not start with the MCAP magic, so no repair can recover it.
    #[error("not an MCAP file")]
    NotMcap,
    /// The MCAP library reported an error.
    #[error("{0}")]
    Mcap(#[from] McapError),
    /// A filesystem operation failed.
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Counts from a successful rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewriteSummary {
    /// Messages copied into the output file.
    pub messages: u64,
    /// Bytes read from the source (the repair progress offset).
    pub bytes_read: u64,
}

/// Passes a recording through, but gives the chunk its writer left open a compressed length that runs to the end
/// of the source. The reader then decompresses the part of that chunk that reached the disk, instead of rejecting
/// a header whose lengths are still [`OPEN_CHUNK_LENGTH`].
struct OpenChunkReader<R> {
    source: R,
    /// Bytes until the next record starts: the start magic at first.
    record_remaining: u64,
    /// Bytes read ahead to inspect a record header, served before the source.
    read_ahead: Vec<u8>,
    /// Set once nothing more needs inspecting: after an open chunk header or the footer.
    passing_through: bool,
}

impl<R: Read> Read for OpenChunkReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.read_ahead.is_empty() && self.record_remaining == 0 && !self.passing_through {
            self.read_record_header()?;
        }
        if !self.read_ahead.is_empty() {
            let served = buffer.len().min(self.read_ahead.len());
            buffer[..served].copy_from_slice(&self.read_ahead[..served]);
            self.read_ahead.drain(..served);
            return Ok(served);
        }
        if self.passing_through {
            return self.source.read(buffer);
        }
        let limit = usize::try_from(self.record_remaining)
            .map_or(buffer.len(), |remaining| remaining.min(buffer.len()));
        let read = self.source.read(&mut buffer[..limit])?;
        self.record_remaining -= read as u64;
        Ok(read)
    }
}

impl<R: Read> OpenChunkReader<R> {
    fn new(source: R) -> Self {
        Self {
            source,
            record_remaining: MAGIC.len() as u64,
            read_ahead: Vec::new(),
            passing_through: false,
        }
    }

    /// Reads the next record's opcode and length ahead, and the chunk header too when the chunk is still open.
    fn read_record_header(&mut self) -> io::Result<()> {
        if !self.read_ahead_to(RECORD_PREFIX_BYTES)? {
            self.passing_through = true;
            return Ok(());
        }
        let opcode = self.read_ahead[0];
        let mut length = [0_u8; 8];
        length.copy_from_slice(&self.read_ahead[1..RECORD_PREFIX_BYTES]);
        let length = u64::from_le_bytes(length);
        if opcode == op::FOOTER {
            self.passing_through = true;
            return Ok(());
        }
        if opcode != op::CHUNK || length != OPEN_CHUNK_LENGTH {
            self.record_remaining = length;
            return Ok(());
        }
        self.passing_through = true;
        let compression_end = RECORD_PREFIX_BYTES + CHUNK_HEADER_BEFORE_COMPRESSION_BYTES;
        if !self.read_ahead_to(compression_end)? {
            return Ok(());
        }
        let mut compression_length = [0_u8; 4];
        compression_length.copy_from_slice(&self.read_ahead[compression_end - 4..compression_end]);
        let compressed_size_start =
            compression_end + u32::from_le_bytes(compression_length) as usize;
        let header_end = compressed_size_start + 8;
        if !self.read_ahead_to(header_end)? {
            return Ok(());
        }
        let header_bytes = (header_end - RECORD_PREFIX_BYTES) as u64;
        self.read_ahead[compressed_size_start..header_end]
            .copy_from_slice(&(OPEN_CHUNK_LENGTH - header_bytes).to_le_bytes());
        Ok(())
    }

    /// Reads ahead until `length` bytes are held; `false` when the source ends first.
    fn read_ahead_to(&mut self, length: usize) -> io::Result<bool> {
        let missing = length.saturating_sub(self.read_ahead.len());
        (&mut self.source)
            .take(missing as u64)
            .read_to_end(&mut self.read_ahead)?;
        Ok(self.read_ahead.len() >= length)
    }
}

/// Rewrites `source` into `output`, reporting the read offset through `progress`.
///
/// It keeps every complete message. A source cut short is not an error, so one cut before its first complete
/// message becomes an empty recording; a source that is not an MCAP recording fails.
pub fn rewrite(
    source: &Path,
    output: &Path,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<RewriteSummary, RewriteError> {
    let source_file = File::open(source)?;
    let total_bytes = source_file.metadata()?.len();
    let buffered = BufReader::with_capacity(SOURCE_READ_BYTES, source_file);
    rewrite_from_reader(buffered, total_bytes, output, progress, cancel)
}

/// Rewrites from an already-open reader (tests and streaming callers).
pub fn rewrite_from_reader<R: Read>(
    source: R,
    total_bytes: u64,
    output: &Path,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<RewriteSummary, RewriteError> {
    let mut source = OpenChunkReader::new(source);
    let mut messages = 0_u64;
    let mut bytes_consumed = 0_u64;
    let mut progress_at_bytes = 0_u64;

    let file = fs::File::create(output)?;
    let mut writer = Writer::with_options(
        BufWriter::new(file),
        WriteOptions::new()
            .compression(Some(Compression::Lz4))
            .emit_message_indexes(true),
    )?;

    let options = LinearReaderOptions::default()
        .with_skip_end_magic(true)
        .with_validate_chunk_crcs(true);
    let mut linear = LinearReader::new_with_options(options);

    let mut parse_error: Option<McapError> = None;
    while let Some(event) = linear.next_event() {
        if cancel.load(Ordering::Relaxed) {
            if let Err(error) = fs::remove_file(output) {
                debug!(%error, "Failed to remove cancelled rewrite output");
            }
            return Err(RewriteError::Cancelled);
        }
        match event {
            Ok(LinearReadEvent::ReadRequest(need)) => {
                if let Err(error) = fill_read_request(
                    &mut linear,
                    &mut source,
                    need,
                    &mut bytes_consumed,
                    &mut progress_at_bytes,
                    total_bytes,
                    progress,
                ) {
                    parse_error = Some(error);
                    break;
                }
            }
            Ok(LinearReadEvent::Record { opcode, data }) => match parse_record(opcode, data) {
                Ok(record) => {
                    write_record(&mut writer, record.into_owned(), &mut messages)?;
                }
                Err(error) => {
                    parse_error = Some(error);
                    break;
                }
            },
            Err(error) => {
                parse_error = Some(error);
                break;
            }
        }
    }

    if cancel.load(Ordering::Relaxed) {
        if let Err(error) = fs::remove_file(output) {
            debug!(%error, "Failed to remove cancelled rewrite output");
        }
        return Err(RewriteError::Cancelled);
    }

    if let Err(error) = writer.finish() {
        if let Err(remove_error) = fs::remove_file(output) {
            debug!(%remove_error, "Failed to remove rewrite output after finish error");
        }
        return Err(RewriteError::Mcap(error));
    }

    if let Some(error) = parse_error
        && messages == 0
        && !matches!(error, McapError::UnexpectedEof)
    {
        if let Err(remove_error) = fs::remove_file(output) {
            debug!(%remove_error, "Failed to remove rewrite output after parse error");
        }
        return Err(match error {
            McapError::BadMagic => RewriteError::NotMcap,
            other => RewriteError::Mcap(other),
        });
    }

    progress(bytes_consumed, total_bytes);
    Ok(RewriteSummary {
        messages,
        bytes_read: bytes_consumed,
    })
}

fn fill_read_request<R: Read>(
    linear: &mut LinearReader,
    source: &mut R,
    need: usize,
    bytes_consumed: &mut u64,
    progress_at_bytes: &mut u64,
    total_bytes: u64,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<(), McapError> {
    if need == 0 {
        return Ok(());
    }
    let mut remaining = need;
    while remaining > 0 {
        if *bytes_consumed >= total_bytes {
            return Err(McapError::UnexpectedEof);
        }
        let unread = (total_bytes - *bytes_consumed) as usize;
        let batch = remaining.min(SOURCE_READ_BYTES).min(unread);
        let buffer = linear.insert(batch);
        let written = source.read(buffer)?;
        linear.notify_read(written);
        *bytes_consumed += written as u64;
        report_read_progress(bytes_consumed, progress_at_bytes, total_bytes, progress);
        if written == 0 {
            return Err(McapError::UnexpectedEof);
        }
        remaining -= written;
    }
    Ok(())
}

fn report_read_progress(
    bytes_consumed: &u64,
    progress_at_bytes: &mut u64,
    total_bytes: u64,
    progress: &mut dyn FnMut(u64, u64),
) {
    let chunk = SOURCE_READ_BYTES as u64;
    while *bytes_consumed >= *progress_at_bytes + chunk {
        *progress_at_bytes += chunk;
        progress(*bytes_consumed, total_bytes);
    }
}

fn write_record(
    writer: &mut Writer<BufWriter<fs::File>>,
    record: Record<'static>,
    messages: &mut u64,
) -> Result<(), RewriteError> {
    match record {
        Record::Schema { header, data } => {
            writer.add_schema_with_id(header.id, &header.name, &header.encoding, &data)?;
        }
        Record::Channel(channel) => {
            writer.add_channel_with_id(
                channel.id,
                channel.schema_id,
                &channel.topic,
                &channel.message_encoding,
                &channel.metadata,
            )?;
        }
        Record::Message { header, data } => {
            writer.write_to_known_channel(&header, &data)?;
            *messages += 1;
        }
        Record::Attachment {
            header,
            data,
            crc: _,
        } => {
            let attachment = Attachment {
                create_time: header.create_time,
                log_time: header.log_time,
                name: header.name,
                media_type: header.media_type,
                data,
            };
            writer.attach(&attachment)?;
        }
        Record::Metadata(metadata) => {
            writer.write_metadata(&metadata)?;
        }
        Record::Header(_)
        | Record::DataEnd(_)
        | Record::Footer(_)
        | Record::MessageIndex(_)
        | Record::ChunkIndex(_)
        | Record::AttachmentIndex(_)
        | Record::Statistics(_)
        | Record::MetadataIndex(_)
        | Record::SummaryOffset(_)
        | Record::Chunk { .. }
        | Record::Unknown { .. } => {}
    }
    Ok(())
}
