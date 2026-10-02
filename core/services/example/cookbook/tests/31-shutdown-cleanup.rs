//! `on_shutdown` runs when the Kernel stops.

use core::{
    convert::Infallible,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_service::{
    Kernel, RunOutcome, Service, ServiceBuilder, ServiceContext, ServiceError, testing::PausedClock,
};

static CLEANED_UP: AtomicBool = AtomicBool::new(false);

struct ShutdownCookbookService;

#[derive(Clone, Default, clap::Args)]
struct ShutdownCookbookArguments;

struct ShutdownCookbook;

#[derive(Clone, Default)]
struct ShutdownCookbookSnapshot {
    cleaned_up: bool,
}

enum ShutdownCookbookRequest {
    Cleanup,
}

impl Service for ShutdownCookbookService {
    type Domain = ShutdownCookbook;
    type Context = ();
    type Arguments = ShutdownCookbookArguments;

    const NAME: &'static str = "cookbook_shutdown";
    const VERSION: &'static str = "1.0.0";

    fn build(
        _context: &ServiceContext<ShutdownCookbookArguments>,
    ) -> Result<ServiceBuilder<ShutdownCookbook>, ServiceError> {
        Ok(ServiceBuilder::new(ShutdownCookbookSnapshot::default())
            .on_shutdown(ShutdownCookbookRequest::Cleanup))
    }
}

impl Domain for ShutdownCookbook {
    type Snapshot = ShutdownCookbookSnapshot;
    type Request = ShutdownCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut ShutdownCookbookSnapshot,
        command: Command<ShutdownCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(ShutdownCookbookRequest::Cleanup) = command;
        CLEANED_UP.store(true, Ordering::SeqCst);
        snapshot.cleaned_up = true;
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

#[tokio::test(start_paused = true)]
async fn on_shutdown_marks_cleanup_in_the_snapshot() {
    CLEANED_UP.store(false, Ordering::SeqCst);
    let mut builder =
        ShutdownCookbookService::build(&ServiceContext::new(ShutdownCookbookArguments)).unwrap();
    let shutdown = builder.shutdown_handle();
    let backend: std::sync::Arc<dyn CommsBackend> = std::sync::Arc::new(ChannelBackend::default());
    let kernel = Kernel::start(
        ShutdownCookbookService::NAME,
        builder,
        std::sync::Arc::clone(&backend),
        std::sync::Arc::new(PausedClock::start()),
    )
    .await
    .unwrap();
    let run = tokio::spawn(kernel.run());
    tokio::time::advance(Duration::from_millis(1)).await;
    shutdown.trigger();
    tokio::time::advance(Duration::from_secs(5)).await;
    assert!(CLEANED_UP.load(Ordering::SeqCst));
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(1), run)
            .await
            .unwrap()
            .unwrap(),
        RunOutcome::Stopped
    );
}
