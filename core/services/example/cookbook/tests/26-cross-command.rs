//! Call another service's Command through the shared Session.

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use blueos_api::{Message, cdr_encoding, command_key};
use blueos_comms::{CommsBackend, QueryBody, channel::ChannelBackend};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::{
    blueos_example_msgs::{LevelResponse, SetLevelGoal},
    blueos_msgs::CommandAck,
};
use blueos_service::{
    RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError, TaskFailed, new_job_id,
    testing::Harness,
};

struct TargetCookbookService;

#[derive(Clone, Default, clap::Args)]
struct TargetCookbookArguments;

struct TargetCookbook;

#[derive(Clone, Default)]
struct TargetCookbookSnapshot {
    level: u8,
}

enum TargetCookbookRequest {
    SetLevel(u8),
}

impl Service for TargetCookbookService {
    type Domain = TargetCookbook;
    type Context = ();
    type Arguments = TargetCookbookArguments;

    const NAME: &'static str = "cookbook_target";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TargetCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<TargetCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<TargetCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(TargetCookbookSnapshot::default())
            .command("SetLevel", |request: SetLevelGoal| {
                Ok(TargetCookbookRequest::SetLevel(request.level))
            })
            .state("gauge", |snapshot: &TargetCookbookSnapshot| LevelResponse {
                level: snapshot.level,
                max_level: 100,
            }))
    }
}

impl Domain for TargetCookbook {
    type Snapshot = TargetCookbookSnapshot;
    type Request = TargetCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut TargetCookbookSnapshot,
        command: Command<TargetCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(TargetCookbookRequest::SetLevel(level)) = command;
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

struct CallerCookbookService;

#[derive(Clone, Default, clap::Args)]
struct CallerCookbookArguments;

struct CallerCookbook;

#[derive(Clone, Default)]
struct CallerCookbookSnapshot {
    remote_applied: bool,
}

enum CallerCookbookObserved {
    RemoteApplied,
}

impl Service for CallerCookbookService {
    type Domain = CallerCookbook;
    type Context = ();
    type Arguments = CallerCookbookArguments;

    const NAME: &'static str = "cookbook_caller";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<CallerCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<CallerCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<CallerCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(CallerCookbookSnapshot::default())
            .task(
                "call_remote",
                RestartPolicy::Never,
                |task_context| async move {
                    let body = QueryBody::new(
                        SetLevelGoal { level: 12 }
                            .encode()
                            .expect("the request encodes"),
                        cdr_encoding(SetLevelGoal::SCHEMA_NAME),
                    )
                    .with_attachment(new_job_id().to_string().into_bytes());
                    let replies = task_context
                        .session
                        .get(
                            &command_key(TargetCookbookService::NAME, "SetLevel"),
                            Some(body),
                            Duration::from_secs(10),
                        )
                        .await
                        .expect("the remote command key is valid");
                    let [Ok(reply)] = replies.as_slice() else {
                        return Err(TaskFailed);
                    };
                    let ack = CommandAck::decode(&reply.payload().to_bytes())
                        .expect("reply is CommandAck");
                    if !ack.accepted {
                        return Err(TaskFailed);
                    }
                    task_context
                        .commands
                        .send(Command::ObservedFact(CallerCookbookObserved::RemoteApplied))
                        .await
                        .map_err(|_error| TaskFailed)?;
                    Ok(())
                },
            )
            .state("done", |snapshot: &CallerCookbookSnapshot| LevelResponse {
                level: u8::from(snapshot.remote_applied),
                max_level: 1,
            }))
    }
}

impl Domain for CallerCookbook {
    type Snapshot = CallerCookbookSnapshot;
    type Request = Infallible;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = CallerCookbookObserved;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut CallerCookbookSnapshot,
        command: Command<Infallible, Infallible, Infallible, CallerCookbookObserved>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::ObservedFact(CallerCookbookObserved::RemoteApplied) => {
                snapshot.remote_applied = true;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Request(never) => match never {},
            Command::IoResult(never) => match never {},
            Command::Tick(never) => match never {},
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
async fn a_task_calls_another_services_command_through_the_session() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let target =
        Harness::<TargetCookbookService>::start_on(Arc::clone(&backend), TargetCookbookArguments)
            .await
            .unwrap();
    let caller = Harness::<CallerCookbookService>::start_on(backend, CallerCookbookArguments)
        .await
        .unwrap();

    tokio::time::advance(Duration::from_secs(1)).await;

    assert_eq!(
        target.state::<LevelResponse>("gauge").await.unwrap().level,
        12
    );
    assert_eq!(
        caller.state::<LevelResponse>("done").await.unwrap().level,
        1
    );
    drop(caller);
    drop(target);
}
