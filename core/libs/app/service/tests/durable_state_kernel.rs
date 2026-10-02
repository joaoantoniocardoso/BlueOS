//! Durable state across restarts (ticket #56, D-28).

mod common;

use core::{
    convert::Infallible,
    fmt::{self, Display, Formatter},
    num::NonZeroU32,
    time::Duration,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::{task::JoinSet, time};

use blueos_comms::channel::ChannelBackend;
use blueos_domain::{Command, Domain, DomainDurable, Outcome};
use blueos_idl::msg::{blueos_example_msgs::EmptyRequest, blueos_msgs::JobStatusStatus};
use blueos_jobs::{DomainJobs, JobEnd, JobGraph, JobKind, JobStatus, Jobs};
use blueos_service::{
    CommandSender, DurableWriteFlush, Kernel, Service, ServiceBuilder, ServiceContext,
    ServiceError, ShutdownHandle, testing::PausedClock,
};
use blueos_settings::{STATE_NAME_PREFIX, state_file_name};

const STATE_VERSION: NonZeroU32 = NonZeroU32::MIN;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct CounterState {
    value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum Step {
    Alpha,
    Beta,
    Work,
}

impl Display for Step {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alpha => formatter.write_str("Alpha"),
            Self::Beta => formatter.write_str("Beta"),
            Self::Work => formatter.write_str("Work"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum VaultTick {
    Restored,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VaultSnapshot {
    durable: CounterState,
    observed: u32,
    saw_restored_tick: bool,
    jobs: Jobs<Step>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum VaultRequest {
    Bump,
    StartJob,
    StartParallel,
}

struct Vault;

impl Domain for Vault {
    type Snapshot = VaultSnapshot;
    type Request = VaultRequest;
    type IoResult = Infallible;
    type Tick = VaultTick;
    type ObservedFact = Infallible;
    type Event = ();
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        _now: blueos_domain::Now,
    ) -> Outcome<Self::Event, Self::Tick, Self::IoRequest, Self::TimerKey> {
        match command {
            Command::Request(VaultRequest::Bump) => {
                snapshot.durable.value += 1;
                snapshot.observed += 1;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Request(VaultRequest::StartJob) => {
                snapshot.jobs.start(JobGraph::Leaf(Step::Work));
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Request(VaultRequest::StartParallel) => {
                snapshot.jobs.start(JobGraph::Parallel(vec![
                    JobGraph::Leaf(Step::Alpha),
                    JobGraph::Leaf(Step::Beta),
                ]));
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Tick(VaultTick::Restored) => {
                snapshot.saw_restored_tick = true;
                let interrupted_leaves: Vec<_> = snapshot
                    .jobs
                    .list()
                    .into_iter()
                    .filter(|view| view.status == JobStatus::Interrupted)
                    .filter_map(|view| {
                        let JobKind::Leaf(step) = view.kind else {
                            return None;
                        };
                        Some((view.job_id, step.clone()))
                    })
                    .collect();
                for (job_id, step) in interrupted_leaves {
                    match step {
                        Step::Alpha => {
                            snapshot
                                .jobs
                                .finish(job_id, JobEnd::Failed)
                                .expect("Alpha is interrupted");
                        }
                        Step::Beta => {
                            snapshot.jobs.retry(job_id).expect("Beta is interrupted");
                        }
                        Step::Work => {}
                    }
                }
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(never) => match never {},
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        _request: Self::IoRequest,
        _error: blueos_domain::IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        unreachable!()
    }
}

impl DomainDurable for Vault {
    type DurableState = CounterState;

    fn durable_state(snapshot: &Self::Snapshot) -> &Self::DurableState {
        &snapshot.durable
    }

    fn set_durable_state(snapshot: &mut Self::Snapshot, state: Self::DurableState) {
        snapshot.durable = state;
    }

    fn restored_tick() -> Self::Tick {
        VaultTick::Restored
    }
}

impl DomainJobs for Vault {
    type Step = Step;

    fn jobs(snapshot: &Self::Snapshot) -> &Jobs<Self::Step> {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut Self::Snapshot) -> &mut Jobs<Self::Step> {
        &mut snapshot.jobs
    }
}

#[derive(Parser)]
struct VaultArguments;

struct VaultService;

impl Service for VaultService {
    type Domain = Vault;
    type Context = ();
    type Arguments = VaultArguments;

    const NAME: &'static str = "vault";
    const VERSION: &'static str = "0.0.0";

    fn build(
        context: &ServiceContext<Self::Arguments>,
    ) -> Result<ServiceBuilder<Self::Domain>, ServiceError> {
        Ok(vault_builder(context, false).0)
    }
}

fn vault_builder(
    context: &ServiceContext<VaultArguments>,
    shutdown_bump: bool,
) -> (ServiceBuilder<Vault>, Option<ShutdownHandle>) {
    let mut builder = ServiceBuilder::new(VaultSnapshot {
        durable: CounterState::default(),
        observed: 0,
        saw_restored_tick: false,
        jobs: Jobs::default(),
    })
    .durable_state_with_jobs(
        VaultService::NAME,
        context.settings_path().map(PathBuf::from),
        STATE_VERSION,
    )
    .command("Bump", |_: EmptyRequest| Ok(VaultRequest::Bump))
    .command("StartJob", |_: EmptyRequest| Ok(VaultRequest::StartJob))
    .command("StartParallel", |_: EmptyRequest| {
        Ok(VaultRequest::StartParallel)
    })
    .jobs();
    let shutdown = if shutdown_bump {
        builder = builder.on_shutdown(VaultRequest::Bump);
        Some(builder.shutdown_handle())
    } else {
        None
    };
    (builder, shutdown)
}

fn state_path(folder: &Path) -> PathBuf {
    folder.join("vault").join(state_file_name(STATE_VERSION))
}

fn durable_value(folder: &Path) -> Option<u32> {
    common::durable_u32_from_json(&state_path(folder), "/domain/value")
}

async fn flush_and_expect_durable(durable_flush: &DurableWriteFlush, folder: &Path, expected: u32) {
    durable_flush.flush().await;
    assert_eq!(durable_value(folder), Some(expected));
}

async fn start_vault_kernel(
    folder: PathBuf,
    shutdown_bump: bool,
) -> (
    Arc<dyn blueos_comms::CommsBackend>,
    CommandSender<Vault>,
    Option<ShutdownHandle>,
    JoinSet<()>,
    DurableWriteFlush,
) {
    let backend: Arc<dyn blueos_comms::CommsBackend> = Arc::new(ChannelBackend::default());
    let context =
        ServiceContext::with_settings_path(VaultArguments {}, Some(folder), Arc::clone(&backend));
    let (builder, shutdown) = vault_builder(&context, shutdown_bump);
    let clock = Arc::new(PausedClock::start());
    let kernel = Kernel::start(VaultService::NAME, builder, Arc::clone(&backend), clock)
        .await
        .expect("vault starts");
    let command_sender = kernel.command_sender().expect("command sender");
    let durable_flush = kernel
        .durable_write_flush()
        .expect("vault registers durable state");
    let mut tasks = JoinSet::new();
    tasks.spawn(async move {
        kernel.run().await;
    });
    (backend, command_sender, shutdown, tasks, durable_flush)
}

async fn send_command(backend: &Arc<dyn blueos_comms::CommsBackend>, command: &str) {
    use blueos_api::{Message, cdr_encoding, command_key};
    use blueos_comms::QueryBody;
    use blueos_idl::msg::blueos_msgs::CommandAck;

    let body = QueryBody::new(
        EmptyRequest::default().encode().expect("request encodes"),
        cdr_encoding(EmptyRequest::SCHEMA_NAME),
    );
    let replies = backend
        .get(
            &command_key(VaultService::NAME, command),
            Some(body),
            Duration::from_secs(10),
        )
        .await
        .expect("command key");
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one ack");
    };
    CommandAck::decode(&reply.payload().to_bytes()).expect("ack decodes");
}

async fn query_jobs(
    backend: &Arc<dyn blueos_comms::CommsBackend>,
) -> blueos_idl::msg::blueos_msgs::JobList {
    use blueos_api::{Message, jobs_key};
    use blueos_idl::msg::blueos_msgs::JobList;

    let replies = backend
        .get(&jobs_key(VaultService::NAME), None, Duration::from_secs(10))
        .await
        .expect("jobs key");
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one jobs value");
    };
    JobList::decode(&reply.payload().to_bytes()).expect("JobList decodes")
}

#[tokio::test(start_paused = true)]
async fn ten_changes_within_one_second_produce_one_write() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let (backend, _sender, _shutdown, mut tasks, durable_flush) =
        start_vault_kernel(folder.clone(), false).await;

    for _ in 0..10 {
        send_command(&backend, "Bump").await;
    }
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 10).await;
    tasks.abort_all();
}

#[tokio::test(start_paused = true)]
async fn corrupt_file_is_moved_aside_and_the_service_starts_fresh() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let path = state_path(&folder);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(&path, b"{not json").expect("write corrupt file");

    let (backend, _sender, _shutdown, mut tasks, durable_flush) =
        start_vault_kernel(folder.clone(), false).await;
    send_command(&backend, "Bump").await;
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 1).await;
    tasks.abort_all();

    let corrupt = fs::read_dir(path.parent().expect("parent"))
        .expect("read dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(STATE_NAME_PREFIX) && name.contains("corrupt"))
        });
    assert!(corrupt.is_some(), "corrupt file was moved aside");
}

#[tokio::test(start_paused = true)]
async fn wrong_shape_with_matching_version_starts_fresh_without_restored_tick() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let path = state_path(&folder);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(
        &path,
        br#"{"VERSION":1,"domain":"not-an-object","jobs":{}}"#,
    )
    .expect("write malformed file");

    let (_backend, _sender, _shutdown, mut tasks, _durable_flush) =
        start_vault_kernel(folder.clone(), false).await;
    tasks.abort_all();

    let corrupt = fs::read_dir(path.parent().expect("parent"))
        .expect("read dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("corrupt"))
        });
    assert!(corrupt, "malformed file was moved aside");
    assert!(!path.is_file(), "state file was replaced");
}

#[tokio::test(start_paused = true)]
async fn after_restore_interrupted_leaves_can_fail_or_retry_from_the_restored_tick() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let (backend, _sender, _shutdown, mut tasks, durable_flush) =
        start_vault_kernel(folder.clone(), false).await;
    send_command(&backend, "StartParallel").await;
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 0).await;
    tasks.abort_all();

    let (restore_backend, _restore_sender, _restore_shutdown, mut restore_tasks, _restore_flush) =
        start_vault_kernel(folder.clone(), false).await;
    time::advance(Duration::from_millis(10)).await;
    let jobs = query_jobs(&restore_backend).await;
    restore_tasks.abort_all();

    let alpha = jobs
        .jobs
        .iter()
        .find(|job| job.name == "Alpha")
        .expect("Alpha leaf");
    let beta = jobs
        .jobs
        .iter()
        .find(|job| job.name == "Beta")
        .expect("Beta leaf");
    assert_eq!(alpha.status, JobStatusStatus::Failed);
    assert_eq!(beta.status, JobStatusStatus::Running);
}

#[tokio::test(start_paused = true)]
async fn observed_facts_and_rederivable_data_are_never_persisted() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let (backend, _sender, _shutdown, mut tasks, durable_flush) =
        start_vault_kernel(folder.clone(), false).await;
    send_command(&backend, "Bump").await;
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 1).await;
    tasks.abort_all();

    let raw = fs::read_to_string(state_path(&folder)).expect("state file");
    assert!(!raw.contains("observed"));
    assert!(!raw.contains("saw_restored_tick"));
}

#[tokio::test(start_paused = true)]
async fn shutdown_flush_persists_changes_from_the_final_inbox_drain() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let backend: Arc<dyn blueos_comms::CommsBackend> = Arc::new(ChannelBackend::default());
    let context = ServiceContext::with_settings_path(
        VaultArguments {},
        Some(folder.clone()),
        Arc::clone(&backend),
    );
    let (builder, shutdown) = vault_builder(&context, true);
    let shutdown = shutdown.expect("shutdown handle");
    let clock = Arc::new(PausedClock::start());
    let kernel = Kernel::start(VaultService::NAME, builder, Arc::clone(&backend), clock)
        .await
        .expect("vault starts");
    let durable_flush = kernel
        .durable_write_flush()
        .expect("vault registers durable state");
    let run = tokio::spawn(async move { kernel.run().await });

    send_command(&backend, "Bump").await;
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 1).await;

    shutdown.trigger();
    let _ = run.await;

    assert_eq!(durable_value(&folder), Some(2));
}
