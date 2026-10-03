//! Startup, shutdown, liveliness and exit codes (layer L3, channel backend, paused clock).

use core::convert::Infallible;
use std::sync::{Arc, Mutex};

use clap::Args;
use futures_util::future::BoxFuture;
use tokio::time::Duration;

use blueos_api::{service_liveliness_key, state_key};
use blueos_comms::{
    CommsBackend, CommsError, LivelinessSubscriber, LivelinessToken, QueryBody, Queryable, Reply,
    Sample, Subscriber, channel::ChannelBackend,
};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, PumpState};
use blueos_service::{
    Kernel, Service, ServiceBuilder, ServiceContext, ServiceError,
    testing::{Harness, PausedClock},
};

struct LifecycleService;

#[derive(Args, Clone)]
struct LifecycleArguments {}

struct Lifecycle;

#[derive(Clone, Default)]
struct LifecycleSnapshot {
    steps: Vec<&'static str>,
}

enum LifecycleRequest {
    Mark(&'static str),
    Client,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum LifecycleTimerKey {}

struct OrderBackend {
    bus: ChannelBackend,
    journal: Arc<Mutex<Vec<String>>>,
}

impl Service for LifecycleService {
    type Domain = Lifecycle;
    type Context = ();
    type Arguments = LifecycleArguments;

    const NAME: &'static str = "lifecycle";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<LifecycleArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<LifecycleArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Lifecycle>, ServiceError> {
        Ok(ServiceBuilder::new(LifecycleSnapshot::default())
            .on_start(LifecycleRequest::Mark("on_start"))
            .on_shutdown(LifecycleRequest::Mark("on_shutdown"))
            .command("Client", |_: EmptyRequest| Ok(LifecycleRequest::Client))
            .state("progress", |snapshot: &LifecycleSnapshot| PumpState {
                level: snapshot.steps.len() as u8,
                max_level: 0,
                ..PumpState::default()
            }))
    }
}

impl Domain for Lifecycle {
    type Snapshot = LifecycleSnapshot;
    type Request = LifecycleRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = LifecycleTimerKey;

    fn handle(
        snapshot: &mut LifecycleSnapshot,
        command: Command<LifecycleRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(LifecycleRequest::Mark(label)) => {
                snapshot.steps.push(label);
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Request(LifecycleRequest::Client) => {
                snapshot.steps.push("client");
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(_) | Command::Tick(_) | Command::ObservedFact(_) => unreachable!(),
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

impl CommsBackend for OrderBackend {
    fn publish(&self, sample: Sample) -> BoxFuture<'_, Result<(), CommsError>> {
        record(&self.journal, format!("publish {}", sample.key()));
        self.bus.publish(sample)
    }

    fn subscribe<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<Subscriber, CommsError>> {
        self.bus.subscribe(key_expression)
    }

    fn declare_queryable<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<Queryable, CommsError>> {
        record(&self.journal, format!("declare {}", key));
        self.bus.declare_queryable(key)
    }

    fn get<'a>(
        &'a self,
        key_expression: &'a str,
        body: Option<QueryBody>,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<Reply>, CommsError>> {
        self.bus.get(key_expression, body, timeout)
    }

    fn declare_liveliness<'a>(
        &'a self,
        key: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessToken, CommsError>> {
        record(&self.journal, format!("liveliness {}", key));
        self.bus.declare_liveliness(key)
    }

    fn subscribe_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
    ) -> BoxFuture<'a, Result<LivelinessSubscriber, CommsError>> {
        self.bus.subscribe_liveliness(key_expression)
    }

    fn get_liveliness<'a>(
        &'a self,
        key_expression: &'a str,
        timeout: Duration,
    ) -> BoxFuture<'a, Result<Vec<String>, CommsError>> {
        self.bus.get_liveliness(key_expression, timeout)
    }
}

#[tokio::test(start_paused = true)]
async fn liveliness_is_declared_after_initial_state_publish() {
    let journal = Arc::new(Mutex::new(Vec::new()));
    let backend: Arc<dyn CommsBackend> = Arc::new(OrderBackend {
        bus: ChannelBackend::default(),
        journal: Arc::clone(&journal),
    });
    let _kernel = Kernel::start(
        LifecycleService::NAME,
        LifecycleService::build(
            &ServiceContext::new(
                LifecycleArguments {},
                blueos_service::testing::channel_session(),
            ),
            &(),
        )
        .unwrap(),
        (),
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .unwrap();
    let journal = journal.lock().unwrap().clone();
    let liveliness = format!(
        "liveliness {}",
        service_liveliness_key(LifecycleService::NAME)
    );
    let publish = format!("publish {}", state_key(LifecycleService::NAME, "progress"));
    let liveliness_index = journal
        .iter()
        .position(|entry| entry == &liveliness)
        .expect("liveliness");
    let publish_index = journal
        .iter()
        .position(|entry| entry == &publish)
        .expect("state publish");
    assert!(publish_index < liveliness_index, "journal: {journal:?}");
    assert!(journal.iter().all(|entry| {
        !entry.starts_with("declare ")
            || journal
                .iter()
                .position(|other| other == entry)
                .expect("entry")
                < liveliness_index
    }));
}

#[tokio::test(start_paused = true)]
async fn on_start_runs_before_a_client_command() {
    let harness = Harness::<LifecycleService>::start(LifecycleArguments {})
        .await
        .unwrap();
    harness.send("Client", &EmptyRequest::default()).await;
    let progress = harness.state::<PumpState>("progress").await;
    assert_eq!(progress.level, 2);
}

fn record(journal: &Mutex<Vec<String>>, entry: String) {
    journal.lock().unwrap().push(entry);
}
