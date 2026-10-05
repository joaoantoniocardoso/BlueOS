//! Task panics are logged on the service `log` key (D-29).

#[path = "logging/common.rs"]
mod logging_common;

use core::{convert::Infallible, time::Duration};
use std::sync::Arc;

use clap::Args;

use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_idl::{Message, msg::foxglove_msgs::Log};

use blueos_service::{
    RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness,
};
use logging_common::start_log_publisher;

const RECV_TIMEOUT: Duration = Duration::from_secs(10);

struct TasksService;

#[derive(Args, Clone)]
struct TasksArguments {}

struct TasksDomain;

#[derive(Clone, Default)]
struct TasksSnapshot;

impl Service for TasksService {
    type Domain = TasksDomain;
    type Context = ();
    type Arguments = TasksArguments;

    const NAME: &'static str = "logging-recovery-task";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<TasksArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<TasksArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<TasksDomain>, ServiceError> {
        Ok(ServiceBuilder::new(TasksSnapshot).task(
            "panicker",
            RestartPolicy::Never,
            |_task_context| async move {
                panic!("task panic marker");
            },
        ))
    }
}

impl Domain for TasksDomain {
    type Snapshot = TasksSnapshot;
    type Request = Infallible;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        _snapshot: &mut TasksSnapshot,
        _command: Command<Infallible, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: blueos_domain::IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

#[tokio::test(start_paused = true)]
async fn task_panic_message_reaches_log_key() {
    let mut fixture = start_log_publisher(TasksService::NAME, 0).await;

    let _harness =
        Harness::<TasksService>::start_on(Arc::clone(&fixture.backend), TasksArguments {})
            .await
            .expect("harness starts");

    let sample = tokio::time::timeout(RECV_TIMEOUT, fixture.subscriber.recv())
        .await
        .expect("log sample arrives")
        .expect("sample");
    let decoded = Log::decode(sample.payload().to_bytes().as_ref()).expect("decode Log");
    assert!(decoded.message.contains("task panic marker"));
}
