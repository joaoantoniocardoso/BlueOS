use core::time::Duration;
use std::sync::{Arc, Mutex, mpsc};

use tokio::time::{advance, timeout};

use blueos_api::{Message, cdr_encoding, command_key, state_key};
use blueos_comms::Subscriber;
use blueos_idl::msg::{blueos_example_msgs::LevelResponse, std_msgs::Empty};
use blueos_jobs::JobId;
use blueos_service::{
    Kernel, RunOutcome, Service, ServiceContext,
    testing::{Harness, PausedClock},
};

use super::service::{AsyncHoldLatch, BlockingHoldLatch, EffectsArguments, EffectsService};

pub struct BlockingHoldChannels {
    pub latch: Arc<BlockingHoldLatch>,
    pub started_receiver: mpsc::Receiver<()>,
    pub release_sender: mpsc::Sender<()>,
    pub io_applied_receiver: mpsc::Receiver<()>,
}

pub struct AsyncHoldChannels {
    pub latch: Arc<AsyncHoldLatch>,
    pub started_receiver: mpsc::Receiver<()>,
    pub release_sender: tokio::sync::mpsc::Sender<()>,
    pub io_applied_receiver: mpsc::Receiver<()>,
}

struct AsyncShutdownRun {
    channels: AsyncHoldChannels,
    backend: Arc<dyn blueos_comms::CommsBackend>,
    shutdown: blueos_service::ShutdownHandle,
    run: tokio::task::JoinHandle<RunOutcome>,
}

struct BlockingShutdownRun {
    channels: BlockingHoldChannels,
    backend: Arc<dyn blueos_comms::CommsBackend>,
    shutdown: blueos_service::ShutdownHandle,
    run: tokio::task::JoinHandle<RunOutcome>,
}

pub fn blocking_hold_channels() -> BlockingHoldChannels {
    let (started_sender, started_receiver) = mpsc::sync_channel(0);
    let (release_sender, release_receiver) = mpsc::channel();
    let (io_applied_sender, io_applied_receiver) = mpsc::sync_channel(0);
    let latch = Arc::new(BlockingHoldLatch {
        started: started_sender,
        release: Mutex::new(release_receiver),
        io_applied: io_applied_sender,
    });
    BlockingHoldChannels {
        latch,
        started_receiver,
        release_sender,
        io_applied_receiver,
    }
}

pub fn async_hold_channels() -> AsyncHoldChannels {
    let (started_sender, started_receiver) = mpsc::sync_channel(0);
    let (release_sender, release_receiver) = tokio::sync::mpsc::channel(1);
    let (io_applied_sender, io_applied_receiver) = mpsc::sync_channel(0);
    let latch = Arc::new(AsyncHoldLatch {
        started: started_sender,
        release: tokio::sync::Mutex::new(release_receiver),
        io_applied: io_applied_sender,
    });
    AsyncHoldChannels {
        latch,
        started_receiver,
        release_sender,
        io_applied_receiver,
    }
}

async fn async_shutdown_run() -> AsyncShutdownRun {
    let channels = async_hold_channels();
    let (backend, shutdown, run) = start_effects_kernel_with_shutdown(EffectsArguments {
        async_hold: Some(Arc::clone(&channels.latch)),
        ..EffectsArguments::default()
    })
    .await;
    AsyncShutdownRun {
        channels,
        backend,
        shutdown,
        run,
    }
}

async fn blocking_shutdown_run() -> BlockingShutdownRun {
    let channels = blocking_hold_channels();
    let (backend, shutdown, run) = start_effects_kernel_with_shutdown(EffectsArguments {
        blocking_hold: Some(Arc::clone(&channels.latch)),
        ..EffectsArguments::default()
    })
    .await;
    BlockingShutdownRun {
        channels,
        backend,
        shutdown,
        run,
    }
}

pub async fn subscribe_state(harness: &Harness<EffectsService>, name: &str) -> Subscriber {
    harness
        .backend()
        .subscribe(&state_key(EffectsService::NAME, name))
        .await
        .expect("the state key is valid")
}

pub async fn next_state_sample(subscriber: &mut Subscriber) -> LevelResponse {
    let sample = timeout(Duration::from_secs(10), subscriber.recv())
        .await
        .expect("a state sample is published")
        .expect("the subscription is open");
    LevelResponse::decode(&sample.payload().to_bytes()).expect("the state decodes")
}

pub async fn expect_no_state_sample(subscriber: &mut Subscriber) {
    let observed = timeout(Duration::from_millis(1), subscriber.recv()).await;
    assert!(
        observed.is_err(),
        "expected no further state publish on this subscription"
    );
}

pub async fn start_effects_kernel_with_shutdown(
    arguments: EffectsArguments,
) -> (
    Arc<dyn blueos_comms::CommsBackend>,
    blueos_service::ShutdownHandle,
    tokio::task::JoinHandle<RunOutcome>,
) {
    let service = ServiceContext::new(arguments, blueos_service::testing::channel_session());
    let context = EffectsService::context(&service).expect("the effects context builds");
    let mut builder =
        EffectsService::build(&service, &context).expect("the effects service builds");
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn blueos_comms::CommsBackend> =
        Arc::new(blueos_comms::channel::ChannelBackend::default());
    let kernel = Kernel::start(
        EffectsService::NAME,
        builder,
        context,
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .expect("the kernel starts");
    let run = tokio::spawn(async move { kernel.run().await });
    (backend, shutdown, run)
}

pub async fn run_async_shutdown_waits_for_in_flight_io() {
    let setup = async_shutdown_run().await;
    let AsyncHoldChannels {
        started_receiver,
        release_sender,
        io_applied_receiver,
        ..
    } = setup.channels;
    let backend = setup.backend;
    let shutdown = setup.shutdown;
    let run = setup.run;
    let command = spawn_run_hold_command(backend, "RunAsyncHold");
    await_blocking_hold_started(started_receiver).await;
    shutdown.trigger();
    advance(Duration::from_millis(1)).await;
    release_sender
        .send(())
        .await
        .expect("the test releases async IO");
    finish_in_flight_hold_and_shutdown(command, io_applied_receiver, run, || {}).await;
}

pub async fn run_async_shutdown_abandon_after_five_seconds() {
    let setup = async_shutdown_run().await;
    let AsyncHoldChannels {
        started_receiver, ..
    } = setup.channels;
    let backend = setup.backend;
    let shutdown = setup.shutdown;
    let run = setup.run;
    let _command = spawn_run_hold_command(backend, "RunAsyncHold");
    await_blocking_hold_started(started_receiver).await;
    let (finished_sender, mut finished_receiver) = tokio::sync::oneshot::channel();
    let run_task = tokio::spawn(async move {
        let outcome = run.await.expect("join");
        let _ = finished_sender.send(());
        outcome
    });
    shutdown.trigger();
    let ((), outcome) = tokio::join!(
        async {
            advance(Duration::from_millis(1)).await;
            advance(Duration::from_secs(4)).await;
            assert!(
                finished_receiver.try_recv().is_err(),
                "shutdown must not return before the five second IO drain budget elapses"
            );
            advance(Duration::from_secs(1)).await;
        },
        run_task,
    );
    assert_eq!(outcome.expect("join"), RunOutcome::Stopped);
}

pub async fn run_blocking_shutdown_waits_for_in_flight_io() {
    let setup = blocking_shutdown_run().await;
    let BlockingHoldChannels {
        started_receiver,
        release_sender,
        io_applied_receiver,
        ..
    } = setup.channels;
    let backend = setup.backend;
    let shutdown = setup.shutdown;
    let run = setup.run;
    let command = spawn_run_hold_command(backend, "RunBlockingHold");
    await_blocking_hold_started(started_receiver).await;
    shutdown.trigger();
    advance(Duration::from_millis(1)).await;
    finish_in_flight_hold_and_shutdown(command, io_applied_receiver, run, move || {
        release_sender
            .send(())
            .expect("the test releases blocking IO");
    })
    .await;
}

pub async fn assert_blocking_active_level(harness: &Harness<EffectsService>, expected: u8) {
    let level = harness
        .query::<Empty, LevelResponse>("blocking_active", &Empty::default())
        .await
        .expect("harness query")
        .expect("the Query answers")
        .level;
    assert_eq!(level, expected);
}

pub async fn finish_in_flight_hold_and_shutdown(
    command: tokio::task::JoinHandle<()>,
    io_applied_receiver: mpsc::Receiver<()>,
    run: tokio::task::JoinHandle<RunOutcome>,
    release: impl FnOnce() + Send,
) {
    release();
    await_blocking_hold_applied(io_applied_receiver).await;
    command.await.expect("hold command finishes");
    assert_shutdown_stops_within_one_second(run).await;
}

pub async fn assert_shutdown_stops_within_one_second(run: tokio::task::JoinHandle<RunOutcome>) {
    let outcome = timeout(Duration::from_secs(1), run)
        .await
        .expect("shutdown finishes without waiting the full drain budget")
        .expect("join");
    assert_eq!(outcome, RunOutcome::Stopped);
}

pub async fn await_blocking_hold_applied(io_applied_receiver: mpsc::Receiver<()>) {
    tokio::task::spawn_blocking(move || io_applied_receiver.recv())
        .await
        .expect("io applied join")
        .expect("blocking IO result should reach the Domain");
}

pub async fn await_blocking_hold_started(started_receiver: mpsc::Receiver<()>) {
    tokio::task::spawn_blocking(move || started_receiver.recv())
        .await
        .expect("started join")
        .expect("blocking IO should start");
}

pub fn spawn_run_hold_command(
    backend: Arc<dyn blueos_comms::CommsBackend>,
    command_name: &'static str,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let body = blueos_comms::QueryBody::new(
            Empty::default().encode().unwrap(),
            cdr_encoding(Empty::SCHEMA_NAME),
        )
        .with_attachment(JobId::from_u128(1).to_string().into_bytes());
        backend
            .get(
                &command_key(EffectsService::NAME, command_name),
                Some(body),
                Duration::from_secs(10),
            )
            .await
            .unwrap();
    })
}
