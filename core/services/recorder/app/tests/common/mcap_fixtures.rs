//! MCAP test fixtures.

use std::{fs, path::Path, path::PathBuf};

use mcap::{
    Compression,
    write::{WriteOptions, Writer},
};

pub(crate) fn mcap_channel_topic_names(path: &Path) -> Vec<String> {
    let data = fs::read(path).expect("read mcap");
    let summary = mcap::Summary::read(&data)
        .expect("parse mcap")
        .expect("mcap summary");
    summary
        .channels
        .values()
        .map(|channel| channel.topic.clone())
        .collect()
}

pub(crate) fn write_truncated_mcap(path: &Path) {
    let file = fs::File::create(path).expect("create");
    let mut writer = Writer::with_options(file, WriteOptions::new()).expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    for index in 0..8 {
        let header = mcap::records::MessageHeader {
            channel_id,
            sequence: index as u32,
            log_time: index as u64,
            publish_time: index as u64,
        };
        writer
            .write_to_known_channel(&header, b"payload")
            .expect("write");
    }
    writer.finish().expect("finish");
    let bytes = fs::read(path).expect("read");
    fs::write(path, &bytes[..bytes.len() / 2]).expect("truncate");
}

/// Writes at `path` what a recorder killed right after it opened its first chunk leaves.
pub(crate) fn write_mcap_killed_before_its_first_message(path: &Path) {
    let writing = path.with_extension("writing");
    let file = fs::File::create(&writing).expect("create");
    let mut writer = Writer::with_options(
        file,
        WriteOptions::new().compression(Some(Compression::Lz4)),
    )
    .expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    let header = mcap::records::MessageHeader {
        channel_id,
        sequence: 0,
        log_time: 0,
        publish_time: 0,
    };
    writer
        .write_to_known_channel(&header, b"payload")
        .expect("write");
    fs::copy(&writing, path).expect("copy the recording as its writer left it");
    drop(writer);
    fs::remove_file(&writing).expect("remove the writer's file");
}

pub(crate) fn recorder_mcaps(directory: &Path) -> Vec<PathBuf> {
    fs::read_dir(directory)
        .expect("read dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("recorder_") && name.ends_with(".mcap"))
        })
        .collect()
}

pub(crate) fn recorder_mcap_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths = recorder_mcaps(directory);
    paths.sort();
    paths
}

pub(crate) fn single_recorder_mcap(directory: &Path) -> PathBuf {
    let paths = recorder_mcap_paths(directory);
    assert_eq!(paths.len(), 1);
    paths[0].clone()
}
