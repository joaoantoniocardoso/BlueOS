//! Shared MCAP bytes for rewrite integration tests.

#![expect(unreachable_pub, dead_code, reason = "shared rewrite test fixtures")]

use core::sync::atomic::{AtomicUsize, Ordering};
use std::{
    fs,
    io::{BufWriter, Read, Seek, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use mcap::{Compression, Writer, write::WriteOptions};
use tempfile::TempDir;

pub struct MaxReadTracker<R> {
    pub inner: R,
    pub max_read: Arc<AtomicUsize>,
}

pub struct RewriteTempPaths {
    pub directory: TempDir,
    pub source: PathBuf,
    pub output: PathBuf,
}

impl<R: Read> Read for MaxReadTracker<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.max_read.fetch_max(read, Ordering::Relaxed);
        Ok(read)
    }
}

impl RewriteTempPaths {
    pub fn new(source_name: &str, output_name: &str) -> Self {
        let directory = tempfile::tempdir().expect("tempdir");
        let source = directory.path().join(source_name);
        let output = directory.path().join(output_name);
        Self {
            directory,
            source,
            output,
        }
    }
}

pub fn write_chunked_recording(path: &Path, message_count: usize) -> u64 {
    write_messages_to_recording(
        path,
        message_count,
        WriteOptions::new().chunk_size(Some(256)),
        1024,
    )
}

pub fn write_large_uncompressed_recording(path: &Path, message_count: usize) -> u64 {
    write_messages_to_recording(
        path,
        message_count,
        WriteOptions::new()
            .compression(None)
            .chunk_size(Some(1024 * 1024)),
        1024,
    )
}

pub fn write_many_small_messages(path: &Path, message_count: usize) -> u64 {
    write_messages_to_recording(
        path,
        message_count,
        WriteOptions::new().compression(None).chunk_size(Some(256)),
        512,
    )
}

pub fn write_messages_to_recording(
    path: &Path,
    message_count: usize,
    options: WriteOptions,
    payload_bytes: usize,
) -> u64 {
    let file = fs::File::create(path).expect("create");
    let mut writer = Writer::with_options(BufWriter::new(file), options).expect("writer");
    let channel_id = open_test_json_channel(&mut writer);
    for message_index in 0..message_count {
        write_test_message(
            &mut writer,
            channel_id,
            u32::try_from(message_index).expect("message index"),
            payload_bytes,
        );
    }
    writer.finish().expect("finish");
    message_count as u64
}

pub fn copy_unfinished_lz4_recording(writing_path: &Path, source_path: &Path, message_count: u32) {
    let mut writer = Writer::with_options(
        fs::File::create(writing_path).expect("create"),
        WriteOptions::new().compression(Some(Compression::Lz4)),
    )
    .expect("writer");
    let channel_id = open_test_json_channel(&mut writer);
    for message_index in 0..message_count {
        write_test_message(&mut writer, channel_id, message_index, 1024);
    }
    fs::copy(writing_path, source_path).expect("copy the recording as its writer left it");
    drop(writer);
}

pub fn copy_unfinished_lz4_single_message(writing_path: &Path, source_path: &Path) {
    let mut writer = Writer::with_options(
        fs::File::create(writing_path).expect("create"),
        WriteOptions::new().compression(Some(Compression::Lz4)),
    )
    .expect("writer");
    let channel_id = open_test_json_channel(&mut writer);
    write_test_message(&mut writer, channel_id, 0, b"payload".len());
    fs::copy(writing_path, source_path).expect("copy the recording as its writer left it");
    drop(writer);
}

pub fn append_growth_bytes(path: &Path, growth_bytes: usize) {
    fs::OpenOptions::new()
        .append(true)
        .open(path)
        .expect("append")
        .write_all(&vec![0xFF; growth_bytes])
        .expect("grow");
}

fn open_test_json_channel<W: Write + Seek>(writer: &mut Writer<W>) -> u16 {
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel")
}

fn write_test_message<W: Write + Seek>(
    writer: &mut Writer<W>,
    channel_id: u16,
    message_index: u32,
    payload_bytes: usize,
) {
    let header = mcap::records::MessageHeader {
        channel_id,
        sequence: message_index,
        log_time: u64::from(message_index),
        publish_time: u64::from(message_index),
    };
    let payload = vec![0_u8; payload_bytes];
    writer
        .write_to_known_channel(&header, &payload)
        .expect("write");
}
