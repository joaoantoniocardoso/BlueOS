//! Question 22: how do I add an IO query?
//!
//! The answer is the `.io_query("Probe", ...)` call in `build`, which is the only difference from a plain Query
//! (`04-query.rs`), and the two tests that observe the answer and the refusal. Shared boilerplate is explained in
//! `01-command.rs`.
//!
//! An IO query reaches a device to answer, so it is answered by its handler outside the Inbox and the Domain
//! never sees it (D-26). A client cannot tell the two kinds apart, so moving a Query between them is not an API
//! change.

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
                // The Domain needs one Command only because the trait requires a Request type; it plays no part
                // in the query.
                .command("Bump", |_: LevelRequest| Ok(IoQueryCookbookRequest::Bump))
                // The handler returns a boxed future because it may await the device, and it gets only the request:
                // an answer that needs the Snapshot is a plain Query. `Refusal` is the error the caller sees.
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
        .expect("the IO query answers")
        .expect("the probe answers");
    // The echoed level proves the answer came from the handler, as the Snapshot holds no level.
    assert_eq!(level.level, 55);
}

#[tokio::test(start_paused = true)]
async fn io_query_refusal_does_not_touch_the_snapshot() {
    let harness = Harness::<IoQueryCookbookService>::start(IoQueryCookbookArguments)
        .await
        .unwrap();
    let refused: Result<LevelResponse, _> = harness
        .query("Probe", &SetLevelGoal { level: 200 })
        .await
        .unwrap();
    // A refusal reaches the caller as an error; the Domain never ran, so the Snapshot cannot have changed.
    assert!(refused.is_err());
}
