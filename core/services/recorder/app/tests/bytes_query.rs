//! Recording `bytes` IO query (layer L3, paused clock).

mod common;

use tempfile::{TempDir, tempdir};

use blueos_comms::ReplyError;
use blueos_idl::msg::blueos_recorder_msgs::{RecordingBytesRequest, RecordingBytesResponse};
use blueos_recorder_app::RecorderService;
use blueos_service::testing::Harness;

use common::start_harness;

#[tokio::test(start_paused = true)]
async fn bytes_query_reads_a_range_and_reports_the_file_size() {
    let (_directory, harness) = start_with_recording("sample.mcap", b"0123456789").await;

    let response = harness
        .query::<_, RecordingBytesResponse>("bytes", &bytes_request("sample.mcap", 3, 4))
        .await
        .expect("bytes");

    assert_eq!(response.data, b"3456");
    assert_eq!(response.size, 10);
}

#[tokio::test(start_paused = true)]
async fn bytes_query_returns_fewer_bytes_at_the_end_of_the_file() {
    let (_directory, harness) = start_with_recording("sample.mcap", b"0123456789").await;

    let tail = harness
        .query::<_, RecordingBytesResponse>("bytes", &bytes_request("sample.mcap", 8, 100))
        .await
        .expect("tail");
    let end = harness
        .query::<_, RecordingBytesResponse>("bytes", &bytes_request("sample.mcap", 10, 100))
        .await
        .expect("end");

    assert_eq!(tail.data, b"89");
    assert_eq!(end.data, b"");
    assert_eq!(end.size, 10);
}

#[tokio::test(start_paused = true)]
async fn bytes_query_answers_at_most_one_mebibyte() {
    let (_directory, harness) =
        start_with_recording("large.mcap", &vec![7_u8; 3 * 1024 * 1024]).await;

    let response = harness
        .query::<_, RecordingBytesResponse>("bytes", &bytes_request("large.mcap", 0, u32::MAX))
        .await
        .expect("bytes");

    assert_eq!(response.data.len(), 1024 * 1024);
    assert_eq!(response.size, 3 * 1024 * 1024);
}

#[tokio::test(start_paused = true)]
async fn bytes_query_refuses_an_offset_past_the_end() {
    let (_directory, harness) = start_with_recording("sample.mcap", b"0123456789").await;

    let answer = harness
        .query::<_, RecordingBytesResponse>("bytes", &bytes_request("sample.mcap", 11, 1))
        .await;

    assert_eq!(
        refusal_reason(answer),
        "Offset 11 is past the end of the recording (10 bytes)."
    );
}

#[tokio::test(start_paused = true)]
async fn bytes_query_refuses_invalid_and_unknown_paths() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;

    let outside = harness
        .query::<_, RecordingBytesResponse>("bytes", &bytes_request("../outside.mcap", 0, 1))
        .await;
    let not_mcap = harness
        .query::<_, RecordingBytesResponse>("bytes", &bytes_request("notes.txt", 0, 1))
        .await;
    let missing = harness
        .query::<_, RecordingBytesResponse>("bytes", &bytes_request("missing.mcap", 0, 1))
        .await;

    assert_eq!(refusal_reason(outside), "Invalid recording path.");
    assert_eq!(
        refusal_reason(not_mcap),
        "Only .mcap recordings are supported."
    );
    assert_eq!(refusal_reason(missing), "Recording not found.");
}

/// Starts the Recorder on a folder that holds one recording `name` with `contents`.
async fn start_with_recording(name: &str, contents: &[u8]) -> (TempDir, Harness<RecorderService>) {
    let directory = tempdir().expect("tempdir");
    std::fs::write(directory.path().join(name), contents).expect("write");
    let harness = start_harness(directory.path()).await;
    (directory, harness)
}

fn bytes_request(path: &str, offset: u64, length: u32) -> RecordingBytesRequest {
    RecordingBytesRequest {
        path: path.into(),
        offset,
        length,
    }
}

fn refusal_reason(answer: Result<RecordingBytesResponse, ReplyError>) -> String {
    let error = answer.expect_err("expected refusal");
    assert_eq!(error.encoding(), "text/plain");
    String::from_utf8(error.payload().to_bytes().into_owned()).expect("utf-8 reason")
}
