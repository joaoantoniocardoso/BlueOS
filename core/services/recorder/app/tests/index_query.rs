//! Recording `index` IO query (layer L3, paused clock).

mod common;

use core::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex, PoisonError};

use blueos_idl::msg::blueos_recorder_msgs::{RecordingIndex, RecordingIndexRequest};
use blueos_recorder_app::{IndexQuerySetup, IndexWalker};
use blueos_recorder_mcap::{IndexError, walk_index};
use tempfile::tempdir;
use tokio::{sync::Notify, time::advance};

use blueos_service::Service;

use common::{start_harness, start_recorder_test_harness};

struct ReleaseWalksOnDrop(Arc<AtomicBool>);

impl Drop for ReleaseWalksOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

#[tokio::test(start_paused = true)]
async fn index_query_round_trips_through_the_service() {
    let directory = tempdir().expect("tempdir");
    let relative = "sample.mcap";
    write_minimal_mcap(&directory.path().join(relative));

    let harness = start_harness(directory.path()).await;
    let index = harness
        .query::<_, RecordingIndex>(
            "index",
            &RecordingIndexRequest {
                path: relative.into(),
                from_offset: 0,
                limit: 2000,
            },
        )
        .await
        .expect("index");
    assert!(index.size > 0);
}

#[tokio::test(start_paused = true)]
async fn index_query_serves_one_walk_at_a_time() {
    let directory = tempdir().expect("tempdir");
    let relative = "sample.mcap";
    write_minimal_mcap(&directory.path().join(relative));

    let release = Arc::new(AtomicBool::new(false));
    let _release_walks = ReleaseWalksOnDrop(Arc::clone(&release));
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let walk_events: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let walk_started = Arc::new(Notify::new());
    let walker = controllable_walker(
        Arc::clone(&release),
        Arc::clone(&active),
        Arc::clone(&max_active),
        Arc::clone(&walk_events),
        Arc::clone(&walk_started),
    );
    let harness = start_recorder_test_harness(
        directory.path(),
        IndexQuerySetup {
            walk_timeout: Duration::from_secs(30),
            walker,
        },
    )
    .await;

    let request = RecordingIndexRequest {
        path: relative.into(),
        from_offset: 0,
        limit: 2000,
    };
    let first_backend = Arc::clone(&harness.backend);
    let first_request = request.clone();
    let first = tokio::spawn(async move { index_query_on(&first_backend, &first_request).await });
    let second_backend = Arc::clone(&harness.backend);
    let second = tokio::spawn(async move { index_query_on(&second_backend, &request).await });
    walk_started.notified().await;
    assert_eq!(max_active.load(Ordering::SeqCst), 1);
    release.store(true, Ordering::Relaxed);
    first.await.expect("first task").expect("first index");
    second.await.expect("second task").expect("second index");
    let events = walk_events.lock().unwrap_or_else(PoisonError::into_inner);
    let starts: Vec<_> = events
        .iter()
        .enumerate()
        .filter(|(_, event)| **event == "start")
        .map(|(index, _)| index)
        .collect();
    let ends: Vec<_> = events
        .iter()
        .enumerate()
        .filter(|(_, event)| **event == "end")
        .map(|(index, _)| index)
        .collect();
    assert_eq!(starts.len(), 2);
    assert_eq!(ends.len(), 2);
    assert!(
        starts[1] > ends[0],
        "second walk must start after the first finishes: {events:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn index_query_timeout_cancels_before_the_next_walk_starts() {
    let directory = tempdir().expect("tempdir");
    let relative = "sample.mcap";
    write_minimal_mcap(&directory.path().join(relative));

    let walk_calls = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let active = Arc::new(AtomicUsize::new(0));
    let walk_started = Arc::new(Notify::new());
    let walker = timeout_test_walker(
        Arc::clone(&walk_calls),
        Arc::clone(&active),
        Arc::clone(&max_active),
        Arc::clone(&walk_started),
    );
    let harness = start_recorder_test_harness(
        directory.path(),
        IndexQuerySetup {
            walk_timeout: Duration::from_millis(50),
            walker,
        },
    )
    .await;

    let request = RecordingIndexRequest {
        path: relative.into(),
        from_offset: 0,
        limit: 2000,
    };
    let backend = Arc::clone(&harness.backend);
    let timeout_request = request.clone();
    let pending = tokio::spawn(async move { index_query_on(&backend, &timeout_request).await });
    walk_started.notified().await;
    advance(Duration::from_millis(100)).await;
    let first = pending.await.expect("index query task");
    assert_eq!(
        index_refusal_reason(first),
        "Recording index walk timed out."
    );
    assert_eq!(walk_calls.load(Ordering::SeqCst), 1);
    assert_eq!(max_active.load(Ordering::SeqCst), 1);
    assert_eq!(active.load(Ordering::SeqCst), 0);

    harness
        .query::<_, RecordingIndex>("index", &request)
        .await
        .expect("second index after timeout");
    assert_eq!(walk_calls.load(Ordering::SeqCst), 2);
}

fn controllable_walker(
    release: Arc<AtomicBool>,
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
    walk_events: Arc<Mutex<Vec<&'static str>>>,
    walk_started: Arc<Notify>,
) -> IndexWalker {
    Arc::new(move |path, from_offset, limit, cancel| {
        let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
        max_active.fetch_max(now_active, Ordering::SeqCst);
        walk_events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push("start");
        walk_started.notify_one();
        while !release.load(Ordering::Relaxed) {
            if cancel.load(Ordering::Relaxed) {
                active.fetch_sub(1, Ordering::SeqCst);
                walk_events
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push("cancelled");
                return Err(IndexError::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        active.fetch_sub(1, Ordering::SeqCst);
        walk_events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push("end");
        walk_index(path, from_offset, limit, cancel)
    })
}

fn timeout_test_walker(
    walk_calls: Arc<AtomicUsize>,
    active: Arc<AtomicUsize>,
    max_active: Arc<AtomicUsize>,
    walk_started: Arc<Notify>,
) -> IndexWalker {
    Arc::new(move |path, from_offset, limit, cancel| {
        let call = walk_calls.fetch_add(1, Ordering::SeqCst);
        let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
        max_active.fetch_max(now_active, Ordering::SeqCst);
        if call == 0 {
            walk_started.notify_one();
            while !cancel.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
            active.fetch_sub(1, Ordering::SeqCst);
            return Err(IndexError::Cancelled);
        }
        active.fetch_sub(1, Ordering::SeqCst);
        walk_index(path, from_offset, limit, cancel)
    })
}

async fn index_query_on(
    backend: &Arc<dyn blueos_comms::CommsBackend>,
    request: &RecordingIndexRequest,
) -> Result<RecordingIndex, blueos_comms::ReplyError> {
    use blueos_api::{Message, cdr_encoding, query_key};
    use blueos_comms::QueryBody;
    use blueos_recorder_app::RecorderService;
    use common::REPLY_TIMEOUT;

    let body = QueryBody::new(
        request.encode().expect("encode"),
        cdr_encoding(RecordingIndexRequest::SCHEMA_NAME),
    );
    let replies = backend
        .get(
            &query_key(RecorderService::NAME, "index"),
            Some(body),
            REPLY_TIMEOUT,
        )
        .await
        .expect("query");
    let [reply] = replies.as_slice() else {
        panic!("expected one reply, got {replies:?}");
    };
    reply
        .clone()
        .map(|sample| RecordingIndex::decode(&sample.payload().to_bytes()).expect("decode index"))
}

fn index_refusal_reason<T: core::fmt::Debug>(
    answer: Result<T, blueos_comms::ReplyError>,
) -> String {
    let error = answer.expect_err("expected refusal");
    assert_eq!(error.encoding(), "text/plain");
    String::from_utf8(error.payload().to_bytes().into_owned()).expect("utf-8 reason")
}

fn write_minimal_mcap(path: &std::path::Path) -> Vec<u8> {
    use blueos_recorder_mcap::MCAP_MAGIC;
    let header = {
        let payload = [0_u32.to_le_bytes(), 4_u32.to_le_bytes(), *b"test"].concat();
        let mut record = vec![0x01_u8];
        record.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        record.extend_from_slice(&payload);
        record
    };
    let mut bytes = Vec::from(MCAP_MAGIC);
    bytes.extend_from_slice(&header);
    std::fs::write(path, &bytes).expect("write");
    bytes
}
