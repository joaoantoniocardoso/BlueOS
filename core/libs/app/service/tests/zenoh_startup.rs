//! L5: on `zenohd`, every endpoint answers before the liveliness token is visible.

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use clap::Args;

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, info_query_key, query_key,
    service_liveliness_key, state_key,
};
use blueos_comms::{CommsBackend, LivelinessEvent, QueryBody};
use blueos_comms_zenoh::ZenohBackend;
use blueos_domain::{Command, Decision, Domain, DomainQueries, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::{EmptyRequest, LevelQueryResponse, PumpState},
    blueos_msgs::ServiceInfo,
};
use blueos_service::{
    Kernel, Service, ServiceBuilder, ServiceContext, ServiceError, testing::PausedClock,
};

struct ZenohStartupService;

#[derive(Args, Clone)]
struct ZenohStartupArguments {}

struct ZenohStartup;

#[derive(Clone, Default)]
struct ZenohStartupSnapshot {
    ready: bool,
}

enum ZenohStartupRequest {
    Noop,
}

enum ZenohStartupQuery {
    Level,
}

impl Service for ZenohStartupService {
    type Domain = ZenohStartup;
    type Context = ();
    type Arguments = ZenohStartupArguments;

    const NAME: &'static str = "zenoh_startup";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<ZenohStartupArguments>,
    ) -> Result<ServiceBuilder<ZenohStartup>, ServiceError> {
        Ok(ServiceBuilder::new(ZenohStartupSnapshot { ready: true })
            .command("Noop", |_: EmptyRequest| Ok(ZenohStartupRequest::Noop))
            .query(
                "level",
                |_: EmptyRequest| Ok(ZenohStartupQuery::Level),
                |response: LevelQueryResponse| Some(response),
            )
            .io_query("Probe", |_request: EmptyRequest| {
                Box::pin(async move {
                    Ok(LevelQueryResponse {
                        level: 4,
                        max_level: 9,
                    })
                })
            })
            .state("ready", |snapshot: &ZenohStartupSnapshot| PumpState {
                level: u8::from(snapshot.ready),
                max_level: 1,
                ..PumpState::default()
            }))
    }
}

impl Domain for ZenohStartup {
    type Snapshot = ZenohStartupSnapshot;
    type Request = ZenohStartupRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(ZenohStartupRequest::Noop) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
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

impl DomainQueries for ZenohStartup {
    type Query = ZenohStartupQuery;
    type Response = LevelQueryResponse;

    fn query(_snapshot: &Self::Snapshot, query: Self::Query, _now: Now) -> Self::Response {
        match query {
            ZenohStartupQuery::Level => LevelQueryResponse {
                level: 3,
                max_level: 9,
            },
        }
    }
}

async fn expect_one_get(
    backend: &Arc<dyn CommsBackend>,
    key: &str,
    body: Option<QueryBody>,
) -> Vec<u8> {
    let replies = backend
        .get(key, body, Duration::from_secs(5))
        .await
        .unwrap_or_else(|error| panic!("get {key} failed: {error}"));
    assert_eq!(replies.len(), 1, "expected one reply for {key}");
    let sample = replies[0]
        .as_ref()
        .unwrap_or_else(|error| panic!("error reply for {key}: {error:?}"));
    sample.payload().to_bytes().into_owned()
}

async fn every_endpoint_answers(backend: &Arc<dyn CommsBackend>, service: &str) {
    let empty = QueryBody::new(
        EmptyRequest::default().encode().expect("encode"),
        cdr_encoding(EmptyRequest::SCHEMA_NAME),
    );
    let info_payload = expect_one_get(backend, &info_query_key(service), None).await;
    let info = ServiceInfo::decode(&info_payload).expect("ServiceInfo");
    assert_eq!(info.name, service);

    let level_payload =
        expect_one_get(backend, &query_key(service, "level"), Some(empty.clone())).await;
    let level = LevelQueryResponse::decode(&level_payload).expect("level query");
    assert_eq!(level.level, 3);

    let probe_payload =
        expect_one_get(backend, &query_key(service, "Probe"), Some(empty.clone())).await;
    let probe = LevelQueryResponse::decode(&probe_payload).expect("Probe io query");
    assert_eq!(probe.level, 4);

    let command_payload = expect_one_get(backend, &command_key(service, "Noop"), Some(empty)).await;
    let ack = CommandAck::decode(&command_payload).expect("CommandAck");
    assert!(ack.accepted);

    let state_payload = expect_one_get(backend, &state_key(service, "ready"), None).await;
    let state = PumpState::decode(&state_payload).expect("PumpState");
    assert_eq!(state.level, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn zenoh_startup_serves_every_endpoint_when_liveliness_appears() {
    let Some(endpoint) = std::env::var("BLUEOS_ZENOH_ENDPOINT").ok() else {
        eprintln!(
            "skip: set BLUEOS_ZENOH_ENDPOINT (for example tcp/127.0.0.1:7447) to run zenoh startup conformance"
        );
        return;
    };
    let backend: Arc<dyn CommsBackend> = Arc::new(
        ZenohBackend::connect(&endpoint)
            .await
            .unwrap_or_else(|error| panic!("could not connect to zenohd at {endpoint}: {error}")),
    );
    let client = Arc::clone(&backend);
    let liveliness_key = service_liveliness_key(ZenohStartupService::NAME);
    let mut liveliness = client
        .subscribe_liveliness(&liveliness_key)
        .await
        .expect("subscribe to service liveliness");
    let kernel_backend = Arc::clone(&backend);
    let kernel = Kernel::start(
        ZenohStartupService::NAME,
        ZenohStartupService::build(&ServiceContext::new(ZenohStartupArguments {})).unwrap(),
        kernel_backend,
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("start");
    let run = tokio::spawn(kernel.run());
    let event = tokio::time::timeout(Duration::from_secs(5), liveliness.recv())
        .await
        .expect("liveliness within deadline")
        .expect("liveliness event");
    assert_eq!(
        event,
        LivelinessEvent::Put {
            key: liveliness_key.clone(),
        }
    );
    every_endpoint_answers(&client, ZenohStartupService::NAME).await;
    run.abort();
}
