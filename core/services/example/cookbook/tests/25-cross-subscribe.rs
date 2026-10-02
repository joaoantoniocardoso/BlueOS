//! Subscribe to another service's State on the shared backbone.

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use tokio::time::timeout;

use blueos_api::{Message, state_key};
use blueos_comms::{CommsBackend, Subscriber, channel::ChannelBackend};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelQueryResponse, SetLevelRequest};
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

    fn build(
        _context: &ServiceContext<PublisherCookbookArguments>,
    ) -> Result<ServiceBuilder<PublisherCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(PublisherCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelRequest| {
                Ok(PublisherCookbookRequest::SetLevel(request.level))
            })
            .state("gauge", |snapshot: &PublisherCookbookSnapshot| {
                LevelQueryResponse {
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

async fn next_state(subscriber: &mut Subscriber) -> LevelQueryResponse {
    let sample = timeout(Duration::from_secs(10), subscriber.recv())
        .await
        .expect("state update arrives before timeout")
        .expect("state stream stays open");
    LevelQueryResponse::decode(&sample.payload().to_bytes()).expect("state payload decodes")
}

#[tokio::test(start_paused = true)]
async fn a_client_subscribes_to_another_services_state_key() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let harness = Harness::<PublisherCookbookService>::start_on(
        Arc::clone(&backend),
        PublisherCookbookArguments,
    )
    .await
    .unwrap();
    let mut gauge = backend
        .subscribe(&state_key(PublisherCookbookService::NAME, "gauge"))
        .await
        .expect("the gauge key subscribes");

    harness
        .send("SetLevel", &SetLevelRequest { level: 33 })
        .await;

    assert_eq!(next_state(&mut gauge).await.level, 33);
}
