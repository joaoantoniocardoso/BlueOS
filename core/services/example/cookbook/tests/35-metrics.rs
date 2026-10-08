//! Questions 35 and 36: how do I add a metric from a Task, and how does a Domain expose a count (D-35)?
//!
//! The answer is the `reporter` Task in `build` and the two tests at the bottom.
//!
//! A Task records with the `metrics` facade macros, the way any Rust code does: no BlueOS API. Whatever it records
//! lands in its own Service's `metrics` State, next to the Kernel's own numbers (Inbox step time, Task restarts,
//! the Tokio runtime). Keep the handle the macro returns, so a Task on a hot path looks the name up once.
//!
//! A Domain never records: logic crates do not depend on `metrics`, and a gate fails when one does. It keeps the
//! count in its Snapshot, and a Task follows a Projection of it and records the gauge. The Domain stays pure and the
//! count stays a plain value a test of the Domain can read.

use core::{convert::Infallible, time::Duration};

use tokio::time::advance;

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_msgs::ServiceMetrics;
use blueos_service::{
    RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness,
};

struct DoorbellService;

#[derive(Clone, Default, clap::Args)]
struct DoorbellArguments;

struct Doorbell;

/// The Domain owns the count of rings; no metric appears in it.
#[derive(Clone, Default)]
struct DoorbellSnapshot {
    rings: u64,
}

enum DoorbellRequest {
    Ring,
}

impl Service for DoorbellService {
    type Domain = Doorbell;
    type Context = ();
    type Arguments = DoorbellArguments;

    const NAME: &'static str = "cookbook_metrics";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<DoorbellArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<DoorbellArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Doorbell>, ServiceError> {
        // The Projection lets the Task read the Domain's count while the Domain knows nothing of metrics (D-27, D-35).
        let (builder, rings) = ServiceBuilder::<Doorbell>::new(DoorbellSnapshot::default())
            .projection(|snapshot| snapshot.rings);
        Ok(builder.task("reporter", RestartPolicy::Never, move |task| {
            let mut rings = rings.subscribe();
            async move {
                // Handles are created once, outside the loop, so the hot path does no name lookup.
                let rings_gauge = metrics::gauge!("rings");
                let reports = metrics::counter!("rings_reports");
                loop {
                    rings_gauge.set(*rings.borrow_and_update() as f64);
                    tokio::select! {
                        () = task.shutdown.cancelled() => return Ok(()),
                        changed = rings.changed() => {
                            if changed.is_err() {
                                return Ok(());
                            }
                            reports.increment(1);
                        }
                    }
                }
            }
        }))
    }
}

impl Domain for Doorbell {
    type Snapshot = DoorbellSnapshot;
    type Request = DoorbellRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut DoorbellSnapshot,
        command: Command<DoorbellRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(DoorbellRequest::Ring) => {
                snapshot.rings += 1;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(never) => match never {},
            Command::Tick(never) => match never {},
            Command::ObservedFact(never) => match never {},
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
async fn a_task_records_a_counter_that_shows_in_the_metrics_state() {
    let harness = Harness::<DoorbellService>::start(DoorbellArguments)
        .await
        .unwrap();
    harness
        .command_sender()
        .send_awaiting_ack(Command::Request(DoorbellRequest::Ring))
        .await
        .unwrap();
    advance(Duration::from_secs(1)).await;

    let metrics = harness.state::<ServiceMetrics>("metrics").await.unwrap();

    // Proof: the Task's own counter shows up in the Service's `metrics` State beside the Kernel's numbers.
    let reports = metrics
        .counters
        .iter()
        .find(|counter| counter.name == "rings_reports")
        .expect("the Task's counter is in the metrics State");
    assert_eq!(reports.value, 1);
}

#[tokio::test(start_paused = true)]
async fn a_domain_exposes_its_count_through_a_projection_a_task_records() {
    let harness = Harness::<DoorbellService>::start(DoorbellArguments)
        .await
        .unwrap();
    for _ in 0..2 {
        harness
            .command_sender()
            .send_awaiting_ack(Command::Request(DoorbellRequest::Ring))
            .await
            .unwrap();
    }
    advance(Duration::from_secs(1)).await;

    let metrics = harness.state::<ServiceMetrics>("metrics").await.unwrap();

    // Proof: the gauge equals the Domain's Snapshot count, recorded by the Task and not by the Domain.
    let rings = metrics
        .gauges
        .iter()
        .find(|gauge| gauge.name == "rings")
        .expect("the Task's gauge is in the metrics State");
    assert_eq!(rings.value.to_bits(), 2.0_f64.to_bits());
}
