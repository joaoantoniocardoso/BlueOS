//! MCAP rewrite loop over a byte stream.

use core::sync::atomic::{AtomicBool, Ordering};
use std::{
    fs,
    io::{BufWriter, Read},
    path::Path,
};

use mcap::{
    Attachment, McapError, Writer, parse_record,
    records::Record,
    sans_io::linear_reader::{LinearReadEvent, LinearReader, LinearReaderOptions},
};
use tracing::debug;

use super::{RewriteError, RewriteSummary, SOURCE_READ_BYTES, open_chunk_reader::OpenChunkReader};

struct ReadProgress<'a> {
    bytes_consumed: &'a mut u64,
    progress_at_bytes: &'a mut u64,
    total_bytes: u64,
    progress: &'a mut dyn FnMut(u64, u64),
}

struct FinishRewriteOutput<'a> {
    writer: Writer<BufWriter<fs::File>>,
    output: &'a Path,
    cancel: &'a AtomicBool,
    parse_error: Option<McapError>,
    messages: u64,
    progress: &'a mut dyn FnMut(u64, u64),
    bytes_consumed: u64,
    total_bytes: u64,
}

struct RewriteLoop<'a, R: Read> {
    linear: &'a mut LinearReader,
    source: &'a mut OpenChunkReader<R>,
    writer: &'a mut Writer<BufWriter<fs::File>>,
    messages: &'a mut u64,
    bytes_consumed: &'a mut u64,
    progress_at_bytes: &'a mut u64,
    total_bytes: u64,
    progress: &'a mut dyn FnMut(u64, u64),
    parse_error: &'a mut Option<McapError>,
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
    let mut writer = open_rewrite_writer(file)?;
    let mut linear = new_linear_reader();
    let mut parse_error: Option<McapError> = None;

    run_rewrite_events(
        RewriteLoop {
            linear: &mut linear,
            source: &mut source,
            writer: &mut writer,
            messages: &mut messages,
            bytes_consumed: &mut bytes_consumed,
            progress_at_bytes: &mut progress_at_bytes,
            total_bytes,
            progress,
            parse_error: &mut parse_error,
        },
        cancel,
        output,
    )?;

    finish_rewrite_output(FinishRewriteOutput {
        writer,
        output,
        cancel,
        parse_error,
        messages,
        progress,
        bytes_consumed,
        total_bytes,
    })
}

fn open_rewrite_writer(file: fs::File) -> Result<Writer<BufWriter<fs::File>>, RewriteError> {
    Writer::with_options(
        BufWriter::new(file),
        mcap::WriteOptions::new()
            .compression(Some(mcap::Compression::Lz4))
            .emit_message_indexes(true),
    )
    .map_err(RewriteError::Mcap)
}

fn new_linear_reader() -> LinearReader {
    let options = LinearReaderOptions::default()
        .with_skip_end_magic(true)
        .with_validate_chunk_crcs(true);
    LinearReader::new_with_options(options)
}

fn run_rewrite_events<R: Read>(
    rewrite: RewriteLoop<'_, R>,
    cancel: &AtomicBool,
    output: &Path,
) -> Result<(), RewriteError> {
    loop {
        if cancel.load(Ordering::Relaxed) {
            remove_output_on_cancel(output);
            return Err(RewriteError::Cancelled);
        }
        let stop = match rewrite.linear.next_event() {
            None => break,
            Some(Ok(LinearReadEvent::ReadRequest(need))) => {
                let read_progress = ReadProgress {
                    bytes_consumed: rewrite.bytes_consumed,
                    progress_at_bytes: rewrite.progress_at_bytes,
                    total_bytes: rewrite.total_bytes,
                    progress: rewrite.progress,
                };
                if let Err(error) =
                    fill_read_request(rewrite.linear, rewrite.source, need, read_progress)
                {
                    *rewrite.parse_error = Some(error);
                    true
                } else {
                    false
                }
            }
            Some(Ok(LinearReadEvent::Record { opcode, data })) => {
                match parse_record(opcode, data) {
                    Ok(record) => {
                        write_record(rewrite.writer, record.into_owned(), rewrite.messages)?;
                        false
                    }
                    Err(error) => {
                        *rewrite.parse_error = Some(error);
                        true
                    }
                }
            }
            Some(Err(error)) => {
                *rewrite.parse_error = Some(error);
                true
            }
        };
        if stop {
            break;
        }
    }
    Ok(())
}

fn finish_rewrite_output(
    mut context: FinishRewriteOutput<'_>,
) -> Result<RewriteSummary, RewriteError> {
    if context.cancel.load(Ordering::Relaxed) {
        remove_output_on_cancel(context.output);
        return Err(RewriteError::Cancelled);
    }

    if let Err(error) = context.writer.finish() {
        remove_output_after_error(context.output);
        return Err(RewriteError::Mcap(error));
    }

    if let Some(error) = context.parse_error
        && context.messages == 0
        && !matches!(error, McapError::UnexpectedEof)
    {
        remove_output_after_error(context.output);
        return Err(map_fatal_parse_error(error));
    }

    (context.progress)(context.bytes_consumed, context.total_bytes);
    Ok(RewriteSummary {
        messages: context.messages,
        bytes_read: context.bytes_consumed,
    })
}

fn map_fatal_parse_error(error: McapError) -> RewriteError {
    match error {
        McapError::BadMagic => RewriteError::NotMcap,
        other => RewriteError::Mcap(other),
    }
}

fn fill_read_request<R: Read>(
    linear: &mut LinearReader,
    source: &mut R,
    need: usize,
    mut read_progress: ReadProgress<'_>,
) -> Result<(), McapError> {
    if need == 0 {
        return Ok(());
    }
    let mut remaining = need;
    while remaining > 0 {
        if *read_progress.bytes_consumed >= read_progress.total_bytes {
            return Err(McapError::UnexpectedEof);
        }
        let unread = (read_progress.total_bytes - *read_progress.bytes_consumed) as usize;
        let batch = remaining.min(SOURCE_READ_BYTES).min(unread);
        let buffer = linear.insert(batch);
        let written = source.read(buffer)?;
        linear.notify_read(written);
        *read_progress.bytes_consumed += written as u64;
        report_read_progress(&mut read_progress);
        if written == 0 {
            return Err(McapError::UnexpectedEof);
        }
        remaining -= written;
    }
    Ok(())
}

fn report_read_progress(read_progress: &mut ReadProgress<'_>) {
    let chunk = SOURCE_READ_BYTES as u64;
    while *read_progress.bytes_consumed >= *read_progress.progress_at_bytes + chunk {
        *read_progress.progress_at_bytes += chunk;
        (read_progress.progress)(*read_progress.bytes_consumed, read_progress.total_bytes);
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

fn remove_output_on_cancel(output: &Path) {
    if let Err(error) = fs::remove_file(output) {
        debug!(%error, "Failed to remove cancelled rewrite output");
    }
}

fn remove_output_after_error(output: &Path) {
    if let Err(remove_error) = fs::remove_file(output) {
        debug!(%remove_error, "Failed to remove rewrite output after error");
    }
}
