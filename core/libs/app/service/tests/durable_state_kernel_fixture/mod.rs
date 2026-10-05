//! Durable state across restarts (ticket #56, D-28).

#![expect(
    unreachable_pub,
    reason = "fixture items are re-exported to integration test roots"
)]

#[path = "../common/mod.rs"]
mod common;

use core::{convert::Infallible, num::NonZeroU32, time::Duration};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use blueos_comms::channel::ChannelBackend;
use blueos_domain::{Command, Domain, DomainDurable, Outcome};
use blueos_idl::msg::std_msgs::Empty;
use blueos_jobs::{DomainJobs, JobNature, Jobs};
use blueos_service::{
    CommandSender, DurableWriteFlush, Kernel, Service, ServiceBuilder, ServiceContext,
    ServiceError, ShutdownHandle, new_job_id, testing::PausedClock,
};
use blueos_settings::state_file_name;

const STATE_VERSION: NonZeroU32 = NonZeroU32::MIN;
/// A Job type that runs until its Domain ends it.
const LASTING: JobNature = JobNature {
    lasting: true,
    ..JobNature::INSTANT
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultTick {
    Restored,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultSnapshot {
    durable: CounterState,
    observed: u32,
    saw_restored_tick: bool,
    jobs: Jobs,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CounterState {
    value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaultRequest {
    Bump,
    StartJob,
}

pub struct Vault;

pub struct VaultService;

#[derive(clap::Parser, Default)]
pub struct VaultArguments {}

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
            Command::Request(VaultRequest::StartJob) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
            Command::Tick(VaultTick::Restored) => {
                snapshot.saw_restored_tick = true;
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
    fn jobs(snapshot: &Self::Snapshot) -> &Jobs {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut Self::Snapshot) -> &mut Jobs {
        &mut snapshot.jobs
    }
}

impl Service for VaultService {
    type Domain = Vault;
    type Context = ();
    type Arguments = VaultArguments;

    const NAME: &'static str = "vault";
    const VERSION: &'static str = "0.0.0";

    fn context(_service: &ServiceContext<Self::Arguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<Self::Arguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Self::Domain>, ServiceError> {
        Ok(vault_builder(service, false).0)
    }
}

pub async fn start_vault_kernel(
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
    let kernel = Kernel::start(VaultService::NAME, builder, (), Arc::clone(&backend), clock)
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

pub async fn flush_and_expect_durable(
    durable_flush: &DurableWriteFlush,
    folder: &Path,
    expected: u32,
) {
    durable_flush.flush().await;
    assert_eq!(durable_value(folder), Some(expected));
}

pub fn vault_builder(
    context: &ServiceContext<VaultArguments>,
    shutdown_bump: bool,
) -> (ServiceBuilder<Vault>, Option<ShutdownHandle>) {
    let mut builder = ServiceBuilder::new(VaultSnapshot {
        durable: CounterState::default(),
        observed: 0,
        saw_restored_tick: false,
        jobs: Jobs::default(),
    })
    .durable_state_with_jobs(STATE_VERSION)
    .for_service::<VaultService>(context)
    .command("Bump", |_: Empty| Ok(VaultRequest::Bump))
    .job("StartJob", LASTING, |_job_id, _: Empty| {
        Ok(VaultRequest::StartJob)
    })
    .job(
        "AwaitApproval",
        JobNature {
            needs_permission: true,
            ..LASTING
        },
        |_job_id, _: Empty| Ok(VaultRequest::StartJob),
    );
    let shutdown = if shutdown_bump {
        builder = builder.on_shutdown(VaultRequest::Bump);
        Some(builder.shutdown_handle())
    } else {
        None
    };
    (builder, shutdown)
}

pub fn durable_value(folder: &Path) -> Option<u32> {
    common::durable_u32_from_json(&state_path(folder), "/domain/value")
}

pub fn state_path(folder: &Path) -> PathBuf {
    folder.join("vault").join(state_file_name(STATE_VERSION))
}

pub async fn send_command(backend: &Arc<dyn blueos_comms::CommsBackend>, command: &str) {
    use blueos_api::{Message, cdr_encoding, command_key};
    use blueos_comms::QueryBody;
    use blueos_idl::msg::blueos_msgs::CommandAck;

    let body = QueryBody::new(
        Empty::default().encode().expect("request encodes"),
        cdr_encoding(Empty::SCHEMA_NAME),
    )
    .with_attachment(new_job_id().to_string().into_bytes());
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

pub async fn query_jobs(
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
