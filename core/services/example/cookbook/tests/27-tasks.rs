//! A supervised Task runs beside the Inbox and stops on service shutdown.

use core::{convert::Infallible, time::Duration};

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::LevelQueryResponse;
use blueos_service::{
    RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness,
};

struct TasksCookbookService;

#[derive(Clone, Default, clap::Args)]
struct TasksCookbookArguments;

struct TasksCookbook;

#[derive(Clone, Default)]
struct TasksCookbookSnapshot {
    ready: bool,
}

enum TasksCookbookRequest {
    MarkReady,
}

impl Service for TasksCookbookService {
    type Domain = TasksCookbook;
    type Context = ();
    type Arguments = TasksCookbookArguments;

    const NAME: &'static str = "cookbook_tasks";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<TasksCookbookArguments>,
    ) -> Result<ServiceBuilder<TasksCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(TasksCookbookSnapshot::default())
            .task("worker", RestartPolicy::Never, |task_context| async move {
                task_context
                    .commands
                    .send(Command::Request(TasksCookbookRequest::MarkReady))
                    .await
                    .expect("the Inbox accepts the ready Command");
                task_context.shutdown.cancelled().await;
                Ok(())
            })
            .state("ready", |snapshot: &TasksCookbookSnapshot| {
                LevelQueryResponse {
                    level: u8::from(snapshot.ready),
                    max_level: 1,
                }
            }))
    }
}

impl Domain for TasksCookbook {
    type Snapshot = TasksCookbookSnapshot;
    type Request = TasksCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut TasksCookbookSnapshot,
        command: Command<TasksCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(TasksCookbookRequest::MarkReady) => {
                snapshot.ready = true;
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
async fn a_supervised_task_sends_commands_while_the_kernel_runs() {
    let harness = Harness::<TasksCookbookService>::start(TasksCookbookArguments)
        .await
        .unwrap();
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(harness.state::<LevelQueryResponse>("ready").await.level, 1);
}
