use std::fs::{self, File};
use std::io::BufWriter;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use mcap::parse_record;
use mcap::records::Record;
use mcap::sans_io::linear_reader::{LinearReadEvent, LinearReader, LinearReaderOptions};
use mcap::{Attachment, McapError, Writer};
use thiserror::Error;
use tracing::debug;

use crate::writer::McapWriteConfig;

const SOURCE_READ_BYTES: usize = 1024 * 1024;

#[derive(Debug, Error)]
pub enum RewriteError {
    #[error("rewrite cancelled")]
    Cancelled,
    #[error("{0}")]
    Mcap(#[from] McapError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewriteSummary {
    pub messages: u64,
    pub bytes_read: u64,
}

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

#[doc(hidden)]
pub fn rewrite_from_reader<R: Read>(
    mut source: R,
    total_bytes: u64,
    output: &Path,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<RewriteSummary, RewriteError> {
    let mut messages = 0_u64;
    let mut bytes_consumed = 0_u64;

    let write_config = McapWriteConfig::default();
    let mut writer = McapWriteConfig::open_writer(output, write_config)
        .map_err(|error| RewriteError::Io(std::io::Error::other(error.to_string())))?;

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
                if let Err(error) =
                    fill_read_request(&mut linear, &mut source, need, &mut bytes_consumed)
                {
                    parse_error = Some(error);
                    break;
                }
            }
            Ok(LinearReadEvent::Record { opcode, data }) => match parse_record(opcode, data) {
                Ok(record) => {
                    if write_record(&mut writer, record.into_owned(), &mut messages)? {
                        progress(bytes_consumed, total_bytes);
                    }
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
    {
        if let Err(remove_error) = fs::remove_file(output) {
            debug!(%remove_error, "Failed to remove rewrite output after parse error");
        }
        return Err(RewriteError::Mcap(error));
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
) -> Result<(), McapError> {
    if need == 0 {
        return Ok(());
    }
    let mut remaining = need;
    while remaining > 0 {
        let batch = remaining.min(SOURCE_READ_BYTES);
        let buffer = linear.insert(batch);
        let written = source.read(buffer)?;
        linear.notify_read(written);
        *bytes_consumed += written as u64;
        if written == 0 {
            return Err(McapError::UnexpectedEof);
        }
        remaining -= written;
    }
    Ok(())
}

fn write_record(
    writer: &mut Writer<BufWriter<fs::File>>,
    record: Record<'static>,
    messages: &mut u64,
) -> Result<bool, RewriteError> {
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
            return Ok(true);
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
    Ok(false)
}
