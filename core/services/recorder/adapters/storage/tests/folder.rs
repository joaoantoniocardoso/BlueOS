//! Integration tests for the recordings folder adapter.

use core::time::Duration;

use blueos_recorder_mcap::RecordingContents;
use blueos_recorder_storage::{
    LibraryFooterCache, RecordingsFolder, ScannedRecordingFile, StorageError,
    scan_recordings_library,
};

#[test]
fn resolve_rejects_parent_dir() {
    let temporary = tempfile::tempdir().expect("tempdir");
    let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
    assert!(matches!(
        folder.resolve("../outside.mcap"),
        Err(StorageError::InvalidPath)
    ));
}

#[test]
fn each_rewrite_gets_its_own_temporary_file_and_startup_discards_them_all() {
    let temporary = tempfile::tempdir().expect("tempdir");
    let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
    let repairs = [(); 2].map(|()| folder.recover_temporary_path("dive/one.mcap"));
    let snapshots = [(); 2].map(|()| folder.snapshot_temporary_path("dive/one-indexed.mcap"));
    std::fs::create_dir_all(folder.base().join("dive")).expect("directory");
    for path in repairs.iter().chain(&snapshots) {
        std::fs::write(path, b"partial").expect("temporary file");
    }

    RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");

    assert_ne!(repairs[0], repairs[1]);
    assert_ne!(snapshots[0], snapshots[1]);
    for path in repairs.iter().chain(&snapshots) {
        assert_eq!(path.parent(), Some(folder.base().join("dive").as_path()));
        assert!(!path.exists(), "{}", path.display());
    }
}

#[test]
fn scan_reads_the_duration_and_video_topics_of_an_indexed_recording() {
    let temporary = tempfile::tempdir().expect("tempdir");
    let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
    write_recording(&folder.base().join("dive.mcap"));

    let [scanned] = scan_one(&folder, &mut LibraryFooterCache::default());

    assert!(scanned.indexed);
    assert_eq!(
        scanned.contents,
        Some(RecordingContents {
            duration: Duration::from_millis(2_500),
            video_topics: vec!["video/camera/stream".into()],
            other_topic_count: 1,
        })
    );
}

#[test]
fn scan_knows_no_duration_or_video_of_a_recording_without_summary() {
    let temporary = tempfile::tempdir().expect("tempdir");
    let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
    let path = folder.base().join("cut.mcap");
    write_recording(&path);
    let bytes = std::fs::read(&path).expect("read");
    std::fs::write(&path, &bytes[..bytes.len() / 2]).expect("truncate");

    let [scanned] = scan_one(&folder, &mut LibraryFooterCache::default());

    assert!(!scanned.indexed);
    assert_eq!(scanned.contents, None);
}

#[test]
fn scan_reuses_the_summary_of_a_file_whose_size_and_time_are_unchanged() {
    let temporary = tempfile::tempdir().expect("tempdir");
    let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
    let path = folder.base().join("dive.mcap");
    write_recording(&path);
    let mut footer_cache = LibraryFooterCache::default();
    let [first] = scan_one(&folder, &mut footer_cache);

    let metadata = std::fs::metadata(&path).expect("metadata");
    let mut file = std::fs::File::options()
        .write(true)
        .open(&path)
        .expect("open");
    std::io::Write::write_all(&mut file, &vec![0; metadata.len() as usize]).expect("overwrite");
    file.set_modified(metadata.modified().expect("modified"))
        .expect("restore modified time");
    let [second] = scan_one(&folder, &mut footer_cache);

    assert_eq!(second, first);
}

fn scan_one(
    folder: &RecordingsFolder,
    footer_cache: &mut LibraryFooterCache,
) -> [ScannedRecordingFile; 1] {
    scan_recordings_library(folder, None, footer_cache)
        .expect("scan")
        .try_into()
        .expect("one recording")
}

/// A video message at 1 s and a telemetry message at 3.5 s.
fn write_recording(path: &std::path::Path) {
    let mut writer =
        mcap::Writer::new(std::fs::File::create(path).expect("create")).expect("writer");
    let video_schema = writer
        .add_schema("foxglove.CompressedVideo", "ros2msg", b"uint8[] data")
        .expect("schema");
    let video = writer
        .add_channel(
            video_schema,
            "video/camera/stream",
            "cdr",
            &Default::default(),
        )
        .expect("video channel");
    let telemetry = writer
        .add_channel(0, "mavlink/heartbeat", "json", &Default::default())
        .expect("telemetry channel");
    for (channel_id, log_time) in [(video, 1_000_000_000), (telemetry, 3_500_000_000)] {
        writer
            .write_to_known_channel(
                &mcap::records::MessageHeader {
                    channel_id,
                    sequence: 0,
                    log_time,
                    publish_time: log_time,
                },
                b"{}",
            )
            .expect("write");
    }
    writer.finish().expect("finish");
}
