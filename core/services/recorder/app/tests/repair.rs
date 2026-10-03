//! Native repair through the Harness (layer L3, paused clock).

mod common;

use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::{Arc, Mutex, mpsc},
};

use mcap::{Compression, Writer, write::WriteOptions};
use serde::Serialize;
use tempfile::tempdir;
use tokio::{
    sync::Notify,
    time::{advance, timeout},
};

use blueos_api::job_feedback_key;
use blueos_comms::{CommsBackend, Subscriber, channel::ChannelBackend};
use blueos_idl::{
    Message,
    msg::blueos_msgs::{CommandAckStatus, JobFeedbackList, JobStatusStatus},
    msg::blueos_recorder_msgs::{
        RepairRecordingFeedback, RepairRecordingGoal, RepairRecordingResult,
    },
};
use blueos_jobs::{JobControl, JobId, JobNature, Jobs};
use blueos_recorder_app::{RecorderContext, RecorderService};
use blueos_recorder_domain::durable::RecorderDurableState;
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_recorder_mcap::is_indexed;
use blueos_service::{
    Service, ServiceContext, new_job_id,
    testing::{Harness, WALL_CLOCK_AT_START, lock_unpoisoned},
};
use blueos_settings::ServiceStateStore;

use common::{
    drain_blocking_io, next_job_result, recorder_arguments, start_harness, start_harness_with,
    subscribe_job_results, wait_for_library_file_listed, wait_for_library_file_not_repairing,
    wait_for_library_file_ready, wait_for_library_state,
};

/// Holds the real rewrite Port mid-flight until the rewrite is cancelled or the test ends.
struct HeldRewrite {
    entered: Arc<Notify>,
    release: Arc<AtomicBool>,
    returned: Arc<AtomicBool>,
}

/// Wraps the real rewrite Port so the test sets each read offset it reports before it runs the real rewrite.
struct SteppedRewrite {
    /// A read offset to report, or `None` to run the real rewrite.
    sender: mpsc::Sender<Option<u64>>,
    receiver: Arc<Mutex<mpsc::Receiver<Option<u64>>>>,
}

#[derive(Serialize)]
struct PersistedRecorderState {
    #[serde(rename = "VERSION")]
    version: u32,
    domain: RecorderDurableState,
    jobs: Jobs,
}

impl Drop for HeldRewrite {
    fn drop(&mut self) {
        self.release.store(true, Ordering::Relaxed);
    }
}

impl HeldRewrite {
    fn new() -> Self {
        Self {
            entered: Arc::new(Notify::new()),
            release: Arc::new(AtomicBool::new(false)),
            returned: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Wraps the rewrite Port in `context`: it waits for a cancel or the release, then runs the real rewrite.
    fn wrap(&self, context: &mut RecorderContext) {
        let rewriter = Arc::clone(&context.rewriter);
        let entered = Arc::clone(&self.entered);
        let release = Arc::clone(&self.release);
        let returned = Arc::clone(&self.returned);
        context.rewriter = Arc::new(move |source, output, progress, cancel| {
            entered.notify_one();
            while !release.load(Ordering::Relaxed) && !cancel.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
            let rewritten = rewriter(source, output, progress, cancel);
            returned.store(true, Ordering::Relaxed);
            rewritten
        });
    }
}

impl SteppedRewrite {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    /// Wraps the rewrite Port in `context`: it reports each offset the test sends, then runs the real rewrite.
    fn wrap(&self, context: &mut RecorderContext) {
        let rewriter = Arc::clone(&context.rewriter);
        let receiver = Arc::clone(&self.receiver);
        context.rewriter = Arc::new(move |source, output, progress, cancel| {
            let total_bytes = fs::metadata(source).map_or(0, |metadata| metadata.len());
            while let Ok(Some(bytes_processed)) = lock_unpoisoned(&receiver).recv() {
                progress(bytes_processed, total_bytes);
            }
            rewriter(source, output, progress, cancel)
        });
    }

    fn report(&self, bytes_processed: u64) {
        self.sender
            .send(Some(bytes_processed))
            .expect("the rewrite is waiting for a step");
    }

    fn finish(&self) {
        self.sender
            .send(None)
            .expect("the rewrite is waiting for a step");
    }
}

#[tokio::test(start_paused = true)]
async fn repair_rewrites_truncated_recording_and_publishes_its_job_result() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("broken.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);

    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "RepairRecording").await;

    wait_for_library_file_listed(&harness, "broken.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let job_id = new_job_id();
    let ack = harness
        .submit(
            "RepairRecording",
            job_id,
            &RepairRecordingGoal {
                path: "broken.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    wait_for_library_file_ready(&harness, "broken.mcap").await;

    assert!(is_indexed(&path), "repaired file must be indexed on disk");

    let (job, result) = next_job_result::<RepairRecordingResult>(&mut results).await;
    assert_eq!(
        (job.job_id, job.status, job.reason.as_str()),
        (job_id.to_string(), JobStatusStatus::Succeeded, "")
    );
    assert_eq!(result.path, "broken.mcap");
    assert_eq!(
        repair_job_status(&harness, job_id).await,
        JobStatusStatus::Succeeded
    );
}

#[tokio::test(start_paused = true)]
async fn a_recording_killed_before_its_first_message_is_repaired_empty_and_no_longer_offered_for_repair()
 {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("killed.mcap");
    write_mcap_killed_before_its_first_message(&path);
    set_modified_seconds_ago(&path, 20);

    let harness = start_harness(directory.path()).await;
    let mut results = subscribe_job_results(&harness, "RepairRecording").await;
    wait_for_library_file_listed(&harness, "killed.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let ack = harness
        .send(
            "RepairRecording",
            &RepairRecordingGoal {
                path: "killed.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    let (job, _result) = next_job_result::<RepairRecordingResult>(&mut results).await;
    assert_eq!(
        (job.status, job.reason.as_str()),
        (JobStatusStatus::Succeeded, "")
    );
    wait_for_library_file_ready(&harness, "killed.mcap").await;
    let bytes = fs::read(&path).expect("read the repaired recording");
    assert_eq!(mcap::MessageStream::new(&bytes).expect("stream").count(), 0);
    wait_for_library_state(harness.backend(), |library| {
        library.files.iter().any(|file| {
            file.path == "killed.mcap"
                && !file
                    .allowed_operations
                    .iter()
                    .any(|operation| operation == "RepairRecording")
        })
    })
    .await;
}

#[tokio::test(start_paused = true)]
async fn repair_feedback_reports_a_growing_read_offset_also_to_a_client_that_opens_it_mid_repair() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("progress.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);
    let total_bytes = fs::metadata(&path).expect("metadata").len();

    let stepped = SteppedRewrite::new();
    let harness = start_harness_with(directory.path(), |context| stepped.wrap(context)).await;
    wait_for_library_file_listed(&harness, "progress.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let mut feedback = harness
        .backend()
        .subscribe(&job_feedback_key(RecorderService::NAME, "RepairRecording"))
        .await
        .expect("subscribe to the repair Feedback");
    let mut results = subscribe_job_results(&harness, "RepairRecording").await;
    let job_id = new_job_id();
    let ack = harness
        .submit(
            "RepairRecording",
            job_id,
            &RepairRecordingGoal {
                path: "progress.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "repair rejected: {}", ack.reason);

    let first = total_bytes / 3;
    stepped.report(first);
    wait_for_read_offset(&mut feedback, job_id, first).await;
    let late = harness.job_feedback("RepairRecording").await;
    assert_eq!(
        read_offsets(&late, job_id),
        Some((first, total_bytes)),
        "a client that opens the Feedback mid-repair sees the latest read offset"
    );

    let second = 2 * total_bytes / 3;
    stepped.report(second);
    wait_for_read_offset(&mut feedback, job_id, second).await;

    stepped.finish();
    let (job, _result) = next_job_result::<RepairRecordingResult>(&mut results).await;
    assert_eq!(job.status, JobStatusStatus::Succeeded);
    assert_eq!(
        read_offsets(&harness.job_feedback("RepairRecording").await, job_id),
        None,
        "a Job leaves the Feedback when it ends"
    );
}

#[tokio::test(start_paused = true)]
async fn cancel_job_stops_a_held_repair_and_leaves_the_original_unchanged() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("cancel.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);
    let original = fs::read(&path).expect("read");

    let held = HeldRewrite::new();
    let harness = start_harness_with(directory.path(), |context| held.wrap(context)).await;
    let mut results = subscribe_job_results(&harness, "RepairRecording").await;

    wait_for_library_file_listed(&harness, "cancel.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let job_id = new_job_id();
    let repair_ack = harness
        .submit(
            "RepairRecording",
            job_id,
            &RepairRecordingGoal {
                path: "cancel.mcap".into(),
            },
        )
        .await;
    assert!(
        repair_ack.accepted,
        "repair rejected: {}",
        repair_ack.reason
    );
    held.entered.notified().await;
    let repair_job_id = job_id.to_string();
    wait_for_library_state(harness.backend(), |library| {
        library.files.iter().any(|file| {
            file.path == "cancel.mcap"
                && file.repair_job_id == repair_job_id
                && file
                    .allowed_operations
                    .iter()
                    .any(|operation| operation == "CancelJob")
        })
    })
    .await;

    let cancel_ack = harness.control(job_id, JobControl::Cancel).await;
    assert!(
        cancel_ack.accepted,
        "cancel rejected: {}",
        cancel_ack.reason
    );
    assert_eq!(cancel_ack.status, CommandAckStatus::Canceling);

    wait_for_library_file_not_repairing(&harness, "cancel.mcap").await;

    let (job, result) = next_job_result::<RepairRecordingResult>(&mut results).await;
    assert_eq!(
        (job.status, job.reason.as_str()),
        (JobStatusStatus::Canceled, "")
    );
    assert_eq!(result.path, "cancel.mcap");
    assert_eq!(
        repair_job_status(&harness, job_id).await,
        JobStatusStatus::Canceled
    );

    assert_eq!(fs::read(&path).expect("read"), original);
    assert_eq!(
        recover_files(directory.path()),
        Vec::<String>::new(),
        "cancel must remove the temporary file"
    );
}

#[tokio::test(start_paused = true)]
async fn shutdown_stops_a_held_repair_before_it_finishes() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("held.mcap");
    write_truncated_mcap(&path);
    set_modified_seconds_ago(&path, 20);
    let original = fs::read(&path).expect("read");

    let held = HeldRewrite::new();
    let harness = start_harness_with(directory.path(), |context| held.wrap(context)).await;
    wait_for_library_file_listed(&harness, "held.mcap").await;
    advance(RESCAN_INTERVAL + Duration::from_secs(20)).await;
    drain_blocking_io().await;

    let ack = harness
        .send(
            "RepairRecording",
            &RepairRecordingGoal {
                path: "held.mcap".into(),
            },
        )
        .await;
    assert!(ack.accepted, "repair rejected: {}", ack.reason);
    held.entered.notified().await;

    timeout(Duration::from_secs(5), harness.shutdown())
        .await
        .expect("shutdown finishes");

    assert!(
        held.returned.load(Ordering::Relaxed),
        "shutdown must wait for the rewrite it cancelled"
    );
    assert_eq!(fs::read(&path).expect("read"), original);
    assert_eq!(
        recover_files(directory.path()),
        Vec::<String>::new(),
        "shutdown must remove the temporary file"
    );
}

#[tokio::test(start_paused = true)]
async fn restored_interrupted_repair_job_is_aborted_and_recover_discarded() {
    let directory = tempdir().expect("tempdir");
    let settings_parent = tempdir().expect("settings");
    let path = directory.path().join("broken.mcap");
    write_truncated_mcap(&path);
    fs::write(directory.path().join("broken.recover"), b"temporary").expect("write recover");

    let mut persisted_jobs = Jobs::default();
    let goal = RepairRecordingGoal {
        path: "broken.mcap".into(),
    }
    .encode()
    .expect("encode");
    persisted_jobs
        .submit(
            JobId::from_u128(1),
            "RepairRecording",
            &goal,
            JobNature {
                lasting: true,
                ..JobNature::INSTANT
            },
        )
        .expect("submit");
    let envelope = PersistedRecorderState {
        version: NonZeroU32::MIN.get(),
        domain: RecorderDurableState,
        jobs: persisted_jobs,
    };
    let store = ServiceStateStore::open(
        RecorderService::NAME,
        Some(settings_parent.path().to_path_buf()),
        NonZeroU32::MIN,
    );
    fs::create_dir_all(store.path().parent().expect("state file parent directory")).expect("mkdir");
    fs::write(
        store.path(),
        serde_json::to_vec(&envelope).expect("serialize"),
    )
    .expect("write state");

    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let harness = Harness::<RecorderService>::start_on_with_context(
        Arc::clone(&backend),
        ServiceContext::with_settings_path(
            recorder_arguments(directory.path()),
            Some(settings_parent.path().to_path_buf()),
            backend,
        ),
    )
    .await
    .expect("harness");
    advance(Duration::from_millis(10)).await;
    drain_blocking_io().await;

    assert!(
        !directory.path().join("broken.recover").exists(),
        "startup must discard leftover recover files"
    );
    let job_list = harness.jobs().await;
    let repair_job = job_list
        .jobs
        .iter()
        .find(|job| job.job_type == "RepairRecording")
        .expect("repair job in history");
    assert_eq!(
        (repair_job.status, repair_job.reason.as_str()),
        (JobStatusStatus::Aborted, "interrupted"),
        "interrupted repair must end aborted after restore"
    );
}

#[tokio::test(start_paused = true)]
async fn leftover_recover_file_is_removed_at_startup() {
    let directory = tempdir().expect("tempdir");
    fs::write(directory.path().join("stale.recover"), b"leftover").expect("write");

    let _harness = start_harness(directory.path()).await;

    assert!(
        !directory.path().join("stale.recover").exists(),
        "startup must discard leftover recover files"
    );
}

/// The names of the repair temporary files in `directory`.
fn recover_files(directory: &Path) -> Vec<String> {
    fs::read_dir(directory)
        .expect("read the recordings folder")
        .map(|entry| {
            entry
                .expect("read an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.ends_with(".recover"))
        .collect()
}

/// Waits until `feedback` shows the Job `job_id` at the read offset `bytes_processed`.
async fn wait_for_read_offset(feedback: &mut Subscriber, job_id: JobId, bytes_processed: u64) {
    timeout(Duration::from_secs(5), async {
        while let Some(sample) = feedback.recv().await {
            let list = JobFeedbackList::decode(&sample.payload().to_bytes())
                .expect("decode JobFeedbackList");
            if read_offsets(&list, job_id).is_some_and(|(read, _total)| read == bytes_processed) {
                return;
            }
        }
        panic!("the Feedback subscription closed");
    })
    .await
    .expect("the Feedback shows the read offset");
}

/// The read offset and the total of the Job `job_id` in `feedback`, when it lists the Job.
fn read_offsets(feedback: &JobFeedbackList, job_id: JobId) -> Option<(u64, u64)> {
    let job_id = job_id.to_string();
    feedback
        .jobs
        .iter()
        .find(|entry| entry.job_id == job_id)
        .map(|entry| {
            let progress = RepairRecordingFeedback::decode(&entry.feedback)
                .expect("decode RepairRecordingFeedback");
            (progress.bytes_processed, progress.total_bytes)
        })
}

async fn repair_job_status(harness: &Harness<RecorderService>, job_id: JobId) -> JobStatusStatus {
    let job_id = job_id.to_string();
    harness
        .jobs()
        .await
        .jobs
        .into_iter()
        .find(|job| job.job_id == job_id)
        .expect("the repair Job is in the jobs State")
        .status
}

fn set_modified_seconds_ago(path: &Path, seconds_ago: u64) {
    let stamp = WALL_CLOCK_AT_START.as_secs().saturating_sub(seconds_ago);
    let status = Command::new("touch")
        .args([
            "-d",
            &format!("@{stamp}"),
            path.to_str().expect("utf8 path"),
        ])
        .status()
        .expect("touch");
    assert!(
        status.success(),
        "touch must set an old mtime for the 10 s repair rule"
    );
}

fn write_truncated_mcap(path: &Path) {
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

/// Writes at `path` what a recorder killed right after it opened its first chunk leaves: the header and the open
/// chunk's header, with the first message still in the compressor.
fn write_mcap_killed_before_its_first_message(path: &Path) {
    let writing = path.with_extension("writing");
    let mut writer = Writer::with_options(
        fs::File::create(&writing).expect("create"),
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
