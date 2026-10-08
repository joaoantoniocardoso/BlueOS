//! Question 31: how do I clean up at shutdown?
//!
//! The answer is `.on_shutdown(...)` in `build`, the `Cleanup` arm of `Domain::handle`, and the test at the bottom.
//! Shared boilerplate is explained in `01-command.rs`.

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

// A process global only because the test must observe the Domain from outside the Kernel; a real Service reads its
// Snapshot or its Context instead.
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

    fn context(_service: &ServiceContext<ShutdownCookbookArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        _service: &ServiceContext<ShutdownCookbookArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<ShutdownCookbook>, ServiceError> {
        // Cleanup is a Request like any other, queued as the Kernel's last Command (D-25), so it runs in the Domain
        // with the Snapshot and stays sans-IO (D-03). The Kernel then drains in-flight IO and joins Tasks.
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
    let mut builder = ShutdownCookbookService::build(
        &ServiceContext::new(
            ShutdownCookbookArguments,
            blueos_service::testing::channel_session(),
        ),
        &(),
    )
    .unwrap();
    // `Harness` has no shutdown trigger, so the test drives the Kernel directly to be able to stop it.
    let shutdown = builder.shutdown_handle();
    let backend: std::sync::Arc<dyn CommsBackend> = std::sync::Arc::new(ChannelBackend::default());
    let kernel = Kernel::start(
        ShutdownCookbookService::NAME,
        builder,
        (),
        std::sync::Arc::clone(&backend),
        std::sync::Arc::new(PausedClock::start()),
    )
    .await
    .unwrap();
    let run = tokio::spawn(kernel.run());
    tokio::time::advance(Duration::from_millis(1)).await;
    shutdown.trigger();
    tokio::time::advance(Duration::from_secs(5)).await;
    // Proof: the Domain ran the cleanup Request and the Kernel then reported a clean stop.
    assert!(CLEANED_UP.load(Ordering::SeqCst));
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(1), run)
            .await
            .unwrap()
            .unwrap(),
        RunOutcome::Stopped
    );
}
