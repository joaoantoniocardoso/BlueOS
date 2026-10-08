//! Question 27: how do I run a long-lived background task?
//!
//! The answer is the `.task(...)` call in `build`: a named, supervised Task with a restart policy, which feeds the
//! Domain through its `CommandSender` (D-27). The test proves the Kernel started it beside the Inbox.

use core::{convert::Infallible, time::Duration};

use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::LevelResponse;
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

    fn context(_service: &ServiceContext<TasksCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<TasksCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<TasksCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(TasksCookbookSnapshot::default())
            // The Kernel owns the handle and the shutdown token, so nothing is a detached `tokio::spawn` (D-27).
            // `Never` suits a one-shot Task; a resource-owning loop would use `Always` with a backoff.
            .task("worker", RestartPolicy::Never, |task_context| async move {
                // A Task changes state only by sending a Command; it never touches the Snapshot.
                task_context
                    .commands
                    .send(Command::Request(TasksCookbookRequest::MarkReady))
                    .await
                    .expect("the Inbox accepts the ready Command");
                // A well-behaved Task returns when the Kernel cancels it, in the shutdown order of D-04.
                task_context.shutdown.cancelled().await;
                Ok(())
            })
            .state("ready", |snapshot: &TasksCookbookSnapshot| LevelResponse {
                level: u8::from(snapshot.ready),
                max_level: 1,
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
    // Advancing the paused clock lets the spawned Task run; the State shows its Command reached the Domain.
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(
        harness.state::<LevelResponse>("ready").await.unwrap().level,
        1
    );
}
