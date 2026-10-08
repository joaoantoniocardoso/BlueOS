//! Question 4: how do I add a Query?
//!
//! The answer is the `.query(...)` declaration in `build` (request decoder, response encoder) and the
//! `DomainQueries` impl, which answers from the Snapshot. The shared boilerplate is explained in `01-command.rs`.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, DomainQueries, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{LevelRequest, LevelResponse, SetLevelGoal};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

struct QueryCookbookService;

#[derive(Clone, Default, clap::Args)]
struct QueryCookbookArguments;

struct QueryCookbook;

#[derive(Clone, Default)]
struct QueryCookbookSnapshot {
    level: u8,
}

enum QueryCookbookRequest {
    SetLevel(u8),
}

enum QueryCookbookQuery {
    Level,
}

enum QueryCookbookResponse {
    Level(u8),
}

impl Service for QueryCookbookService {
    type Domain = QueryCookbook;
    type Context = ();
    type Arguments = QueryCookbookArguments;

    const NAME: &'static str = "cookbook_query";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<QueryCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<QueryCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<QueryCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(QueryCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(QueryCookbookRequest::SetLevel(request.level))
            })
            // A Query is a request/reply endpoint (D-26): the first closure decodes the wire request into the Domain's
            // own query type, the second encodes the Domain's response. `None` would mean "no answer", and keeps IDL
            // types out of the Domain.
            .query(
                "Level",
                |_: LevelRequest| Ok(QueryCookbookQuery::Level),
                |response: QueryCookbookResponse| match response {
                    QueryCookbookResponse::Level(level) => Some(LevelResponse {
                        level,
                        max_level: 100,
                    }),
                },
            ))
    }
}

impl Domain for QueryCookbook {
    type Snapshot = QueryCookbookSnapshot;
    type Request = QueryCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut QueryCookbookSnapshot,
        command: Command<QueryCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(QueryCookbookRequest::SetLevel(level)) = command;
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

// A Query is answered from the Snapshot by a pure function, outside `handle`: it cannot change state or emit Effects,
// which is why a read never needs a Command (D-26). Answering with IO is a different kind, see `22-io-query.rs`.
impl DomainQueries for QueryCookbook {
    type Query = QueryCookbookQuery;
    type Response = QueryCookbookResponse;

    fn query(
        snapshot: &QueryCookbookSnapshot,
        query: QueryCookbookQuery,
        _now: Now,
    ) -> QueryCookbookResponse {
        match query {
            QueryCookbookQuery::Level => QueryCookbookResponse::Level(snapshot.level),
        }
    }
}

#[tokio::test(start_paused = true)]
async fn query_reads_the_snapshot() {
    let harness = Harness::<QueryCookbookService>::start(QueryCookbookArguments)
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 22 })
        .await
        .unwrap();
    // The Command first, so the answer can only come from state the Domain actually holds, not a default.
    let answer = harness
        .query::<_, LevelResponse>("Level", &LevelRequest::default())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(answer.level, 22);
}
