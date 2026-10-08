//! Question 25: how do I subscribe to another service's topic?
//!
//! The answer is the test at the bottom: both sides share one backbone, and the subscriber addresses the State by
//! `state_key(service, name)` (D-07, D-10). The publishing service above is the ordinary `.state` of `04`-`05`.

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use tokio::time::timeout;

use blueos_api::{Message, state_key};
use blueos_comms::{CommsBackend, Subscriber, channel::ChannelBackend};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelResponse, SetLevelGoal};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct PublisherCookbookService;

#[derive(Clone, Default, clap::Args)]
struct PublisherCookbookArguments;

struct PublisherCookbook;

#[derive(Clone, Default)]
struct PublisherCookbookSnapshot {
    level: u8,
}

enum PublisherCookbookRequest {
    SetLevel(u8),
}

impl Service for PublisherCookbookService {
    type Domain = PublisherCookbook;
    type Context = ();
    type Arguments = PublisherCookbookArguments;

    const NAME: &'static str = "cookbook_publisher";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<PublisherCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<PublisherCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<PublisherCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(PublisherCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(PublisherCookbookRequest::SetLevel(request.level))
            })
            // A State is published on `state_key(NAME, "gauge")` after every applied Command (D-26).
            .state("gauge", |snapshot: &PublisherCookbookSnapshot| {
                LevelResponse {
                    level: snapshot.level,
                    max_level: 100,
                }
            }))
    }
}

impl Domain for PublisherCookbook {
    type Snapshot = PublisherCookbookSnapshot;
    type Request = PublisherCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut PublisherCookbookSnapshot,
        command: Command<PublisherCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(PublisherCookbookRequest::SetLevel(level)) = command;
        snapshot.level = level;
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

// The timeout turns a missing publish into a failure instead of a hang; time is virtual (D-30).
async fn next_state(subscriber: &mut Subscriber) -> LevelResponse {
    let sample = timeout(Duration::from_secs(10), subscriber.recv())
        .await
        .expect("state update arrives before timeout")
        .expect("state stream stays open");
    LevelResponse::decode(&sample.payload().to_bytes()).expect("state payload decodes")
}

#[tokio::test(start_paused = true)]
async fn a_client_subscribes_to_another_services_state_key() {
    // `start_on` runs the Service on a backbone the test owns, so the test is a second client of it. A real
    // consumer is another process on the same Zenoh session; a Domain follows a State through the Kernel (D-25).
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let harness = Harness::<PublisherCookbookService>::start_on(
        Arc::clone(&backend),
        PublisherCookbookArguments,
    )
    .await
    .unwrap();
    // Subscribe before the Command so the update is not missed.
    let mut gauge = backend
        .subscribe(&state_key(PublisherCookbookService::NAME, "gauge"))
        .await
        .expect("the gauge key subscribes");

    harness
        .send("SetLevel", &SetLevelGoal { level: 33 })
        .await
        .unwrap();

    assert_eq!(next_state(&mut gauge).await.level, 33);
}
