//! The standard `metrics` State as a client sees it (D-35): the Kernel publishes it for every Service, with no code
//! in the Service, on the channel backend with a paused clock.

use core::{convert::Infallible, time::Duration};

use tokio::time::{advance, timeout};

use blueos_api::{Message, state_key};
use blueos_comms::Subscriber;
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_msgs::{EndpointInfo, MetricHistogram, ServiceMetrics};
use blueos_service::{
    Backoff, RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError, TaskFailed,
    testing::Harness,
};

/// How long a client waits for a publication. Time is paused, so the wait costs no real time.
const PUBLICATION_TIMEOUT: Duration = Duration::from_secs(10);

/// A Service whose one Request changes nothing, so a test can apply Commands.
struct CounterService;

/// A Service with no Requests and one Task that records a counter through the `metrics` facade.
struct PingingService;

/// A Service with no Requests and one Task that always fails.
struct FlakyService;

#[derive(clap::Args)]
struct NoArguments {}

struct Counter;

#[derive(Clone, Default)]
struct CounterSnapshot;

enum CounterRequest {
    Bump,
}

impl Service for CounterService {
    type Domain = Counter;
    type Context = ();
    type Arguments = NoArguments;

    const NAME: &'static str = "counter";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<NoArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<NoArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Counter>, ServiceError> {
        Ok(ServiceBuilder::new(CounterSnapshot))
    }
}

impl Service for PingingService {
    type Domain = Counter;
    type Context = ();
    type Arguments = NoArguments;

    const NAME: &'static str = "pinging";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<NoArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<NoArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Counter>, ServiceError> {
        Ok(ServiceBuilder::new(CounterSnapshot).task(
            "pinger",
            RestartPolicy::Never,
            |task| async move {
                metrics::counter!("pings_sent").increment(1);
                task.shutdown.cancelled().await;
                Ok(())
            },
        ))
    }
}

impl Service for FlakyService {
    type Domain = Counter;
    type Context = ();
    type Arguments = NoArguments;

    const NAME: &'static str = "flaky";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<NoArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<NoArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Counter>, ServiceError> {
        Ok(ServiceBuilder::new(CounterSnapshot).task(
            "unreliable",
            RestartPolicy::OnFailure {
                backoff: Backoff::default(),
                max_attempts: 3,
            },
            |_task| async move { Err(TaskFailed) },
        ))
    }
}

impl Domain for Counter {
    type Snapshot = CounterSnapshot;
    type Request = CounterRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut CounterSnapshot,
        _command: Command<CounterRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

#[tokio::test(start_paused = true)]
async fn a_client_that_asks_right_after_boot_gets_the_metrics() {
    let harness = Harness::<CounterService>::start(NoArguments {})
        .await
        .unwrap();

    let metrics = harness.state::<ServiceMetrics>("metrics").await;

    assert_eq!(inbox_steps(&metrics).count, 0);
    assert_eq!(
        metrics
            .gauges
            .iter()
            .map(|gauge| (gauge.name.as_str(), gauge.value.to_bits()))
            .collect::<Vec<_>>(),
        [("inbox_depth", 0.0_f64.to_bits())]
    );
}

#[tokio::test(start_paused = true)]
async fn the_inbox_step_count_grows_with_each_applied_command() {
    let harness = Harness::<CounterService>::start(NoArguments {})
        .await
        .unwrap();
    let mut published = subscribe(&harness).await;

    for _ in 0..3 {
        bump(&harness).await;
    }
    let metrics = next_metrics(&mut published).await;

    assert_eq!(inbox_steps(&metrics).count, 3);
}

#[tokio::test(start_paused = true)]
async fn many_commands_within_one_second_produce_one_publication() {
    let harness = Harness::<CounterService>::start(NoArguments {})
        .await
        .unwrap();
    let mut published = subscribe(&harness).await;

    for _ in 0..10 {
        bump(&harness).await;
        advance(Duration::from_millis(90)).await;
    }
    let metrics = next_metrics(&mut published).await;

    assert_eq!(
        inbox_steps(&metrics).count,
        10,
        "the first publication carries every Command of the second"
    );
}

#[tokio::test(start_paused = true)]
async fn an_idle_service_does_not_republish_unchanged_metrics() {
    let harness = Harness::<CounterService>::start(NoArguments {})
        .await
        .unwrap();
    let mut published = subscribe(&harness).await;
    bump(&harness).await;
    next_metrics(&mut published).await;

    let republished = timeout(PUBLICATION_TIMEOUT, published.recv()).await;

    assert!(
        republished.is_err(),
        "an unchanged value was republished: {republished:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn info_lists_the_metrics_state_with_its_message_type() {
    let harness = Harness::<CounterService>::start(NoArguments {})
        .await
        .unwrap();

    let info = harness.info().await;

    let metrics = info
        .endpoints
        .iter()
        .find(|endpoint| endpoint.name == "metrics");
    assert_eq!(
        metrics,
        Some(&EndpointInfo {
            kind: "state".to_owned(),
            name: "metrics".to_owned(),
            key: state_key(CounterService::NAME, "metrics"),
            interface_type: "blueos_msgs/msg/ServiceMetrics".to_owned(),
            schema: ServiceMetrics::SCHEMA.to_owned(),
        })
    );
}

#[tokio::test(start_paused = true)]
async fn a_counter_a_task_records_is_in_the_metrics_of_its_service_only() {
    let pinging = Harness::<PingingService>::start(NoArguments {})
        .await
        .unwrap();
    let counter = Harness::<CounterService>::start(NoArguments {})
        .await
        .unwrap();
    let mut pinging_published = subscribe(&pinging).await;
    let mut counter_published = subscribe(&counter).await;
    bump(&counter).await;

    let pinging_metrics = next_metrics(&mut pinging_published).await;
    let counter_metrics = next_metrics(&mut counter_published).await;

    assert_eq!(
        pinging_metrics
            .counters
            .iter()
            .filter(|metric| metric.name == "pings_sent")
            .map(|metric| (metric.name.as_str(), metric.value))
            .collect::<Vec<_>>(),
        [("pings_sent", 1)]
    );
    assert_eq!(inbox_steps(&pinging_metrics).count, 0);
    assert!(
        counter_metrics.counters.is_empty(),
        "{:?}",
        counter_metrics.counters
    );
    assert_eq!(inbox_steps(&counter_metrics).count, 1);
}

#[tokio::test(start_paused = true)]
async fn a_task_that_fails_and_restarts_counts_its_restarts() {
    let harness = Harness::<FlakyService>::start(NoArguments {})
        .await
        .unwrap();
    let mut published = subscribe(&harness).await;

    let metrics = next_metrics(&mut published).await;

    let restarts = metrics
        .counters
        .iter()
        .find(|counter| counter.name == "task_restarts")
        .expect("the Kernel counts the restarts of every Task");
    assert_eq!(restarts.value, 2, "three attempts are two restarts");
    assert_eq!(
        restarts
            .labels
            .iter()
            .map(|label| (label.name.as_str(), label.value.as_str()))
            .collect::<Vec<_>>(),
        [("task", "unreliable")]
    );
}

async fn subscribe<S: Service>(harness: &Harness<S>) -> Subscriber {
    harness
        .backend()
        .subscribe(&state_key(S::NAME, "metrics"))
        .await
        .unwrap()
}

async fn bump<S: Service<Domain = Counter>>(harness: &Harness<S>) {
    harness
        .command_sender()
        .send_awaiting_ack(Command::Request(CounterRequest::Bump))
        .await
        .unwrap();
}

async fn next_metrics(subscriber: &mut Subscriber) -> ServiceMetrics {
    let sample = timeout(PUBLICATION_TIMEOUT, subscriber.recv())
        .await
        .expect("the metrics are published")
        .expect("the subscription is open");
    ServiceMetrics::decode(&sample.payload().to_bytes()).unwrap()
}

/// The Inbox step time histogram, which the Kernel records for every Service.
fn inbox_steps(metrics: &ServiceMetrics) -> &MetricHistogram {
    metrics
        .histograms
        .iter()
        .find(|histogram| histogram.name == "inbox_step_seconds")
        .expect("the Kernel records the Inbox step time")
}
