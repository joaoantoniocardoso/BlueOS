//! Projections, [`CommandSender`] and the reconcile pattern (layer L3).

use core::{
    convert::Infallible,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex};

use clap::Args;
use futures_util::{future::BoxFuture, stream};
use tokio::time::timeout;

use blueos_comms::{
    CommsBackend, CommsError, LivelinessSubscriber, LivelinessToken, Query, QueryBody, Queryable,
    Reply, Sample, Subscriber, channel::ChannelBackend,
};
use blueos_domain::{Command, Decision, Domain, DomainQueries, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, LevelResponse, SetLevelGoal};
use blueos_service::{
    RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError, TaskContext, TaskFailed,
    testing::{Harness, lock_unpoisoned},
};

const RECV_TIMEOUT: Duration = Duration::from_secs(10);

struct ReconcileService;

#[derive(Args, Clone)]
struct ReconcileArguments {
    #[arg(skip)]
    projection_changes: Arc<AtomicUsize>,
}

#[derive(Clone)]
struct ReconcileSnapshot {
    desired_lamp: bool,
    observed_lamp: bool,
    scratch: u8,
}

struct ReconcileContext {
    projection_changes: Arc<AtomicUsize>,
}

enum ReconcileRequest {
    SetDesired(bool),
    BumpScratch,
}

enum ReconcileQuery {
    Observed,
}

enum ReconcileResponse {
    Observed(bool),
}

enum ReconcileObserved {
    Lamp(bool),
}

struct ReconcileDomain;

struct RecordingBackend {
    bus: ChannelBackend,
    journal: Arc<Mutex<Vec<String>>>,
}

impl Service for ReconcileService {
    type Domain = ReconcileDomain;
    type Context = ReconcileContext;
    type Arguments = ReconcileArguments;

    const NAME: &'static str = "reconcile-harness";
    const VERSION: &'static str = "1.0.0";

    fn context(
        service: &ServiceContext<ReconcileArguments>,
    ) -> Result<ReconcileContext, ServiceError> {
        Ok(ReconcileContext {
            projection_changes: Arc::clone(&service.arguments().projection_changes),
        })
    }

    fn build(
        _service: &ServiceContext<ReconcileArguments>,
        _context: &ReconcileContext,
    ) -> Result<ServiceBuilder<ReconcileDomain, ReconcileContext>, ServiceError> {
        let (builder, desired_lamp) = ServiceBuilder::new(ReconcileSnapshot {
            desired_lamp: false,
            observed_lamp: false,
            scratch: 0,
        })
        .projection(|snapshot: &ReconcileSnapshot| snapshot.desired_lamp);
        Ok(builder
            .command("SetDesired", |request: SetLevelGoal| {
                Ok(ReconcileRequest::SetDesired(request.level > 0))
            })
            .command("BumpScratch", |_request: LevelRequest| {
                Ok(ReconcileRequest::BumpScratch)
            })
            .query(
                "Observed",
                |_request: LevelRequest| Ok(ReconcileQuery::Observed),
                |response: ReconcileResponse| match response {
                    ReconcileResponse::Observed(on) => Some(LevelResponse {
                        level: u8::from(on),
                        max_level: 0,
                    }),
                },
            )
            .task(
                "reconcile",
                RestartPolicy::Never,
                move |task_context: TaskContext<ReconcileDomain, ReconcileContext>| {
                    let mut desired = desired_lamp.subscribe();
                    async move {
                        while !task_context.shutdown.is_cancelled() {
                            if timeout(RECV_TIMEOUT, desired.changed()).await.is_err() {
                                continue;
                            }
                            task_context
                                .context
                                .projection_changes
                                .fetch_add(1, Ordering::SeqCst);
                            let on = *desired.borrow();
                            let _ = task_context
                                .commands
                                .send(Command::ObservedFact(ReconcileObserved::Lamp(on)))
                                .await;
                        }
                        Ok::<(), TaskFailed>(())
                    }
                },
            ))
    }
}

impl Domain for ReconcileDomain {
    type Snapshot = ReconcileSnapshot;
    type Request = ReconcileRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = ReconcileObserved;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut ReconcileSnapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(ReconcileRequest::SetDesired(on)) => {
                snapshot.desired_lamp = on;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Request(ReconcileRequest::BumpScratch) => {
                snapshot.scratch = snapshot.scratch.saturating_add(1);
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::ObservedFact(ReconcileObserved::Lamp(on)) => {
                snapshot.observed_lamp = on;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(io_result) => match io_result {},
            Command::Tick(tick) => match tick {},
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: blueos_domain::IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

impl DomainQueries for ReconcileDomain {
    type Query = ReconcileQuery;
    type Response = ReconcileResponse;

    fn query(snapshot: &Self::Snapshot, query: Self::Query, _now: Now) -> Self::Response {
        match query {
            ReconcileQuery::Observed => ReconcileResponse::Observed(snapshot.observed_lamp),
        }
    }
}

impl CommsBackend for RecordingBackend {
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
        Box::pin(async move {
            let queries = stream::unfold(self.bus.declare_queryable(key).await?, {
                let journal = Arc::clone(&self.journal);
                let key = key.to_owned();
                move |mut queryable| {
                    let journal = Arc::clone(&journal);
                    let key = key.clone();
                    async move {
                        let query = queryable.recv().await?;
                        Some((recorded(query, key, journal), queryable))
                    }
                }
            });
            Ok(Queryable::new(queries))
        })
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

fn record(journal: &Arc<Mutex<Vec<String>>>, entry: String) {
    lock_unpoisoned(journal).push(entry);
}

fn recorded(query: Query, key: String, journal: Arc<Mutex<Vec<String>>>) -> Query {
    record(&journal, format!("reply {}", key));
    query
}

#[tokio::test(start_paused = true)]
async fn a_projection_that_does_not_change_is_not_redelivered() {
    let projection_changes = Arc::new(AtomicUsize::new(0));
    let harness = Harness::<ReconcileService>::start(ReconcileArguments {
        projection_changes: Arc::clone(&projection_changes),
    })
    .await
    .expect("the reconcile harness starts");
    harness
        .command_sender()
        .send_awaiting_ack(Command::Request(ReconcileRequest::SetDesired(true)))
        .await
        .expect("the desired lamp turns on");
    let after_first = projection_changes.load(Ordering::SeqCst);
    harness
        .command_sender()
        .send_awaiting_ack(Command::Request(ReconcileRequest::BumpScratch))
        .await
        .expect("the scratch field changes");
    assert_eq!(projection_changes.load(Ordering::SeqCst), after_first);
}

#[tokio::test(start_paused = true)]
async fn send_awaiting_ack_returns_the_domain_verdict() {
    let harness = Harness::<ReconcileService>::start(ReconcileArguments {
        projection_changes: Arc::new(AtomicUsize::new(0)),
    })
    .await
    .expect("the reconcile harness starts");
    let accepted = harness
        .command_sender()
        .send_awaiting_ack(Command::Request(ReconcileRequest::SetDesired(true)))
        .await
        .expect("the Domain accepts the Command");
    assert!(accepted.accepted);
    let observed: LevelResponse = harness
        .query("Observed", &LevelRequest::default())
        .await
        .expect("the Query answers");
    assert_eq!(observed.level, 1);
}

#[tokio::test(start_paused = true)]
async fn an_observed_fact_handled_twice_leaves_the_same_snapshot() {
    let harness = Harness::<ReconcileService>::start(ReconcileArguments {
        projection_changes: Arc::new(AtomicUsize::new(0)),
    })
    .await
    .expect("the reconcile harness starts");
    let sender = harness.command_sender();
    sender
        .send_awaiting_ack(Command::ObservedFact(ReconcileObserved::Lamp(true)))
        .await
        .expect("the first fact is applied");
    let first: LevelResponse = harness
        .query("Observed", &LevelRequest::default())
        .await
        .expect("the Query answers");
    sender
        .send_awaiting_ack(Command::ObservedFact(ReconcileObserved::Lamp(true)))
        .await
        .expect("the second fact is applied");
    let second: LevelResponse = harness
        .query("Observed", &LevelRequest::default())
        .await
        .expect("the Query answers");
    assert_eq!(first, second);
}

#[tokio::test(start_paused = true)]
async fn in_process_commands_never_use_the_backbone() {
    let journal = Arc::new(Mutex::new(Vec::new()));
    let backend: Arc<dyn CommsBackend> = Arc::new(RecordingBackend {
        bus: ChannelBackend::default(),
        journal: Arc::clone(&journal),
    });
    let harness = Harness::<ReconcileService>::start_on(
        backend,
        ReconcileArguments {
            projection_changes: Arc::new(AtomicUsize::new(0)),
        },
    )
    .await
    .expect("the reconcile harness starts");
    harness
        .command_sender()
        .send_awaiting_ack(Command::Request(ReconcileRequest::SetDesired(true)))
        .await
        .expect("the Command is applied");
    let entries = lock_unpoisoned(&journal).clone();
    assert!(entries.iter().all(|entry| !entry.contains("command")));
}
