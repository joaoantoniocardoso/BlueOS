use std::sync::Arc;

use blueos_idl::Message;
use blueos_idl::msg::blueos_recorder_msgs::RecordingIndexRequest;
use blueos_recorder::library_io::{LibraryIoContext, run_index_query};
use blueos_recorder_library::{
    LibraryCommand, LibrarySnapshot, ScannedRecording, handle_library_command,
};
use blueos_recorder_mcap::index::walk_index;
use blueos_recorder_policy::RecorderCommand;
use blueos_recorder_storage::RecordingsFolder;
use tokio::sync::mpsc;

#[tokio::test]
async fn library_scan_lists_recordings_in_folder() {
    let directory = tempfile::tempdir().expect("tempdir");
    let recording = directory.path().join("recorder_20240102_030405.mcap");
    std::fs::write(&recording, chunked_mcap_without_summary()).expect("write");
    let (progress_sender, _progress_receiver) = mpsc::channel(8);
    let context =
        LibraryIoContext::new(directory.path().to_path_buf(), progress_sender).expect("context");
    let command = context
        .handle(blueos_recorder_policy::LibraryIo::Scan, None)
        .await;
    let RecorderCommand::Library(LibraryCommand::ScanCompleted { recordings, .. }) = command else {
        panic!("expected scan completed");
    };
    assert_eq!(recordings.len(), 1);
    assert_eq!(recordings[0].relative_path, "recorder_20240102_030405.mcap");
}

#[tokio::test]
async fn repairing_truncated_chunk_finishes_ready() {
    let directory = tempfile::tempdir().expect("tempdir");
    let source = directory.path().join("broken.mcap");
    write_truncated_chunked_mcap(&source);
    let (progress_sender, _progress_receiver) = mpsc::channel(8);
    let context =
        LibraryIoContext::new(directory.path().to_path_buf(), progress_sender).expect("context");
    let command = context
        .handle(
            blueos_recorder_policy::LibraryIo::Repair {
                path: "broken.mcap".into(),
            },
            None,
        )
        .await;
    let RecorderCommand::Library(LibraryCommand::OperationFinished {
        operation,
        succeeded,
        ..
    }) = command
    else {
        panic!("expected repair operation finished, got {command:?}");
    };
    assert_eq!(
        operation,
        blueos_recorder_policy::RecordingOperationKind::Repair
    );
    assert!(succeeded);
    let output = directory.path().join("broken.mcap");
    assert!(blueos_recorder_mcap::footer::is_indexed(&output));
}

#[tokio::test]
async fn delete_active_recording_is_rejected_by_policy() {
    let mut snapshot = LibrarySnapshot::default();
    handle_library_command(
        &mut snapshot,
        LibraryCommand::ScanCompleted {
            recordings: vec![ScannedRecording {
                relative_path: "live.mcap".into(),
                name: "live.mcap".into(),
                size_bytes: 10,
                modified_unix_seconds: 0,
                indexed: true,
            }],
            now_unix_seconds: 0,
        },
        Some("live.mcap"),
    );
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::DeleteRecording {
            path: "live.mcap".into(),
        },
        Some("live.mcap"),
    );
    assert_eq!(
        decision.rejection.as_deref(),
        Some("This recording is still being written.")
    );
}

#[tokio::test]
async fn index_query_returns_chunks_without_summary() {
    let directory = tempfile::tempdir().expect("tempdir");
    let recording = directory.path().join("chunked.mcap");
    std::fs::write(&recording, chunked_mcap_without_summary()).expect("write");
    let folder = Arc::new(std::sync::Mutex::new(
        RecordingsFolder::new(directory.path().to_path_buf()).expect("folder"),
    ));
    let request = RecordingIndexRequest {
        path: "chunked.mcap".into(),
        from_offset: 0,
        limit: 2_000,
    };
    let payload = request.encode().expect("encode");
    let (reply, _encoding) = run_index_query(folder, &payload).await.expect("index");
    let index =
        blueos_idl::msg::blueos_recorder_msgs::RecordingIndex::decode(reply.as_slice().as_slice())
            .expect("decode");
    assert!(!index.chunks.is_empty());
}

fn chunked_mcap_without_summary() -> Vec<u8> {
    use mcap::records::MessageHeader;
    use mcap::{WriteOptions, Writer};
    let file = tempfile::NamedTempFile::new().expect("temp");
    let mut writer = Writer::with_options(
        std::io::BufWriter::new(file.reopen().unwrap()),
        WriteOptions::new(),
    )
    .expect("writer");
    let schema_id = writer
        .add_schema("test", "jsonschema", b"{}")
        .expect("schema");
    let channel_id = writer
        .add_channel(schema_id, "topic", "json", &Default::default())
        .expect("channel");
    for message_index in 0..4 {
        let header = MessageHeader {
            channel_id,
            sequence: message_index as u32,
            log_time: message_index as u64,
            publish_time: message_index as u64,
        };
        writer
            .write_to_known_channel(&header, &[0_u8; 512])
            .expect("write");
    }
    writer.finish().expect("finish");
    std::fs::read(file.path()).expect("read")
}

fn write_truncated_chunked_mcap(path: &std::path::Path) {
    let bytes = chunked_mcap_without_summary();
    std::fs::write(path, &bytes).expect("write");
    let index = walk_index(path, 0, 100).expect("index");
    if index.chunks.len() >= 2 {
        let truncate_at = index.chunks[1].offset as usize + 10;
        std::fs::write(path, &bytes[..truncate_at.min(bytes.len())]).expect("truncate");
        return;
    }
    std::fs::write(path, &bytes[..bytes.len() / 2]).expect("truncate");
}
