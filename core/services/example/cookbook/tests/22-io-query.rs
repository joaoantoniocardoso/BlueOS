//! An IO query is answered outside the Inbox.

use core::{convert::Infallible, future::Future, pin::Pin};

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, LevelResponse, SetLevelGoal};
use blueos_service::{
    Refusal, Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness,
};

struct IoQueryCookbookService;

#[derive(Clone, Default, clap::Args)]
struct IoQueryCookbookArguments;

struct IoQueryCookbook;

#[derive(Clone, Default)]
struct IoQueryCookbookSnapshot;

enum IoQueryCookbookRequest {
    Bump,
}

impl Service for IoQueryCookbookService {
    type Domain = IoQueryCookbook;
    type Context = ();
    type Arguments = IoQueryCookbookArguments;

    const NAME: &'static str = "cookbook_io_query";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<IoQueryCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<IoQueryCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<IoQueryCookbook>, ServiceError> {
        Ok(
            ServiceBuilder::new(IoQueryCookbookSnapshot)
                .command("Bump", |_: LevelRequest| Ok(IoQueryCookbookRequest::Bump))
                .io_query(
                    "Probe",
                    |request: SetLevelGoal| -> Pin<
                        Box<dyn Future<Output = Result<LevelResponse, Refusal>> + Send>,
                    > {
                        Box::pin(async move {
                            if request.level > 100 {
                                return Err(Refusal::from("above maximum"));
                            }
                            Ok(LevelResponse {
                                level: request.level,
                                max_level: 100,
                            })
                        })
                    },
                ),
        )
    }
}

impl Domain for IoQueryCookbook {
    type Snapshot = IoQueryCookbookSnapshot;
    type Request = IoQueryCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut IoQueryCookbookSnapshot,
        command: Command<IoQueryCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(IoQueryCookbookRequest::Bump) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
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
async fn io_query_reads_outside_the_inbox() {
    let harness = Harness::<IoQueryCookbookService>::start(IoQueryCookbookArguments)
        .await
        .unwrap();
    let level: LevelResponse = harness
        .query("Probe", &SetLevelGoal { level: 55 })
        .await
        .expect("the IO query answers");
    assert_eq!(level.level, 55);
}

#[tokio::test(start_paused = true)]
async fn io_query_refusal_does_not_touch_the_snapshot() {
    let harness = Harness::<IoQueryCookbookService>::start(IoQueryCookbookArguments)
        .await
        .unwrap();
    let refused: Result<LevelResponse, _> =
        harness.query("Probe", &SetLevelGoal { level: 200 }).await;
    assert!(refused.is_err());
}
