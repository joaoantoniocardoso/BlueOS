//! Add a Query endpoint and answer from the Snapshot.

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, DomainQueries, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse, SetLevelRequest};
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

    fn build(
        _context: &ServiceContext<QueryCookbookArguments>,
    ) -> Result<ServiceBuilder<QueryCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(QueryCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelRequest| {
                Ok(QueryCookbookRequest::SetLevel(request.level))
            })
            .query(
                "Level",
                |_: EmptyRequest| Ok(QueryCookbookQuery::Level),
                |response: QueryCookbookResponse| match response {
                    QueryCookbookResponse::Level(level) => Some(LevelQueryResponse {
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
        .send("SetLevel", &SetLevelRequest { level: 22 })
        .await;
    let answer = harness
        .query::<_, LevelQueryResponse>("Level", &EmptyRequest::default())
        .await
        .unwrap();
    assert_eq!(answer.level, 22);
}
