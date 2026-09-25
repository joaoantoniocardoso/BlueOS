use blueos_recorder_mcap::footer::MCAP_MAGIC;
use blueos_recorder_storage::{PathError, RecordingsFolder};
use std::fs;
use tempfile::tempdir;

fn build_indexed_mcap() -> Vec<u8> {
    let header = {
        let payload = 4u32.to_le_bytes().to_vec();
        let mut record = vec![0x01_u8];
        record.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        record.extend_from_slice(&payload);
        record
    };
    let mut data = Vec::from(MCAP_MAGIC);
    data.extend_from_slice(&header);
    let summary_start = data.len() as u64;
    let mut footer_payload = Vec::new();
    footer_payload.extend_from_slice(&summary_start.to_le_bytes());
    footer_payload.extend_from_slice(&0u64.to_le_bytes());
    footer_payload.extend_from_slice(&0u32.to_le_bytes());
    let mut footer = vec![0x02_u8];
    footer.extend_from_slice(&(footer_payload.len() as u64).to_le_bytes());
    footer.extend_from_slice(&footer_payload);
    data.extend_from_slice(&footer);
    data.extend_from_slice(&MCAP_MAGIC);
    data
}

#[test]
fn resolve_rejects_path_traversal() {
    let directory = tempdir().expect("tempdir");
    let recorder_dir = directory.path().join("recorder");
    fs::create_dir_all(&recorder_dir).expect("mkdir");
    let outside = directory.path().join("outside.mcap");
    fs::write(outside, b"not-a-recording").expect("write");

    let folder = RecordingsFolder::new(recorder_dir).expect("folder");
    assert!(matches!(
        folder.resolve("../outside.mcap"),
        Err(PathError::InvalidPath)
    ));
}

#[test]
fn scan_skips_vanished_files() {
    let directory = tempdir().expect("tempdir");
    let recorder_dir = directory.path().join("recorder");
    fs::create_dir_all(&recorder_dir).expect("mkdir");
    let kept = recorder_dir.join("recorder_20240102_030405.mcap");
    fs::write(&kept, build_indexed_mcap()).expect("write");
    let vanished = recorder_dir.join("vanished.mcap");
    fs::write(&vanished, b"gone").expect("write vanished");

    let mut folder = RecordingsFolder::new(recorder_dir).expect("folder");
    fs::remove_file(&vanished).expect("remove");
    let files = folder.scan(None).expect("scan");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "recorder_20240102_030405.mcap");
}

#[test]
fn scan_reuses_footer_of_unchanged_files() {
    let directory = tempdir().expect("tempdir");
    let recorder_dir = directory.path().join("recorder");
    fs::create_dir_all(&recorder_dir).expect("mkdir");
    let kept = recorder_dir.join("recorder_20240102_030405.mcap");
    fs::write(&kept, build_indexed_mcap()).expect("write");

    let mut folder = RecordingsFolder::new(recorder_dir).expect("folder");
    let first = folder.scan(None).expect("scan");
    assert!(first[0].indexed);
    let second = folder.scan(None).expect("scan");
    assert!(second[0].indexed);

    fs::write(&kept, build_indexed_mcap()).expect("rewrite");
    let third = folder.scan(None).expect("scan");
    assert!(third[0].indexed);
}

#[test]
fn scan_skips_footer_of_active_file() {
    let directory = tempdir().expect("tempdir");
    let recorder_dir = directory.path().join("recorder");
    fs::create_dir_all(&recorder_dir).expect("mkdir");
    let live = recorder_dir.join("recorder_20240102_030405.mcap");
    fs::write(&live, build_indexed_mcap()).expect("write");

    let mut folder = RecordingsFolder::new(recorder_dir).expect("folder");
    let files = folder
        .scan(Some("recorder_20240102_030405.mcap"))
        .expect("scan");
    assert!(!files[0].indexed);
}

#[test]
fn discard_leftovers_removes_recover_files() {
    let directory = tempdir().expect("tempdir");
    let recorder_dir = directory.path().join("recorder");
    fs::create_dir_all(&recorder_dir).expect("mkdir");
    let leftover = recorder_dir.join("tmp1234.recover");
    fs::write(&leftover, b"partial copy").expect("write");
    let recording = recorder_dir.join("recorder.mcap");
    fs::write(&recording, b"a recording").expect("write");

    let folder = RecordingsFolder::new(recorder_dir).expect("folder");
    let removed = folder.discard_leftovers();
    assert_eq!(removed.len(), 1);
    assert!(!leftover.exists());
    assert!(recording.exists());
}

#[test]
fn discard_leftovers_removes_nested_recover_files() {
    let directory = tempdir().expect("tempdir");
    let recorder_dir = directory.path().join("recorder");
    let nested = recorder_dir.join("sessions");
    fs::create_dir_all(&nested).expect("mkdir");
    let leftover = nested.join("tmp1234.recover");
    fs::write(&leftover, b"partial copy").expect("write");
    let recording = recorder_dir.join("recorder.mcap");
    fs::write(&recording, b"a recording").expect("write");

    let folder = RecordingsFolder::new(recorder_dir).expect("folder");
    let removed = folder.discard_leftovers();
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].0, "sessions/tmp1234.recover");
    assert!(!leftover.exists());
    assert!(recording.exists());
}

#[test]
fn temporary_output_and_replace() {
    let directory = tempdir().expect("tempdir");
    let recorder_dir = directory.path().join("recorder");
    fs::create_dir_all(&recorder_dir).expect("mkdir");
    let recording = recorder_dir.join("live.mcap");
    fs::write(&recording, b"original").expect("write");

    let folder = RecordingsFolder::new(recorder_dir).expect("folder");
    let temporary = folder.temporary_output("live.mcap");
    fs::write(&temporary, b"repaired").expect("write temporary");
    folder.replace(&temporary, "live.mcap").expect("replace");
    assert_eq!(fs::read(recording).expect("read"), b"repaired");
}
