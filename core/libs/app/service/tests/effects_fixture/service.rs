use core::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex, mpsc};

use blueos_domain::IoError;
use blueos_idl::msg::{blueos_example_msgs::LevelResponse, std_msgs::Empty};
use blueos_service::{
    Service, ServiceBuilder, ServiceContext, ServiceError, testing::lock_unpoisoned,
};

pub static PANIC_GUARD: AtomicUsize = AtomicUsize::new(0);

pub struct EffectsService;

#[derive(Clone, clap::Args)]
pub struct EffectsArguments {
    #[arg(long, default_value_t = 10)]
    pub capacity: u8,
    #[arg(skip)]
    pub blocking_hold: Option<Arc<BlockingHoldLatch>>,
    #[arg(skip)]
    pub async_hold: Option<Arc<AsyncHoldLatch>>,
    #[arg(skip)]
    pub record_capacity_done: Option<mpsc::SyncSender<()>>,
}

#[derive(Clone)]
pub struct EffectsContext {
    expected_capacity: u8,
    blocking_hold: Option<Arc<BlockingHoldLatch>>,
    async_hold: Option<Arc<AsyncHoldLatch>>,
    record_capacity_done: Option<mpsc::SyncSender<()>>,
}

#[derive(Clone)]
pub struct EffectsSnapshot {
    pub level: u8,
    pub capacity: u8,
    pub failed_io_requests: u8,
    pub succeeded_io_requests: u8,
    pub last_failed_request: Option<EffectsIoRequest>,
    pub tick_count: u8,
    pub blocking_io_running: bool,
    pub blocking_io_applied: Option<mpsc::SyncSender<()>>,
}

pub enum EffectsRequest {
    RunIoChain,
    RunIoPanic,
    ArmTimer { after: Duration },
    ReArmTimer { after: Duration },
    CancelTimer,
    ScheduleIoWithoutExecutor,
    RecordCapacity,
    RunBlockingHold,
    RunAsyncHold,
}

pub enum EffectsQuery {
    BlockingActive,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectsIoResult {
    Failed {
        request: EffectsIoRequest,
        message: String,
    },
    Succeeded,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectsIoRequest {
    Fail,
    Succeed,
    Panic,
    RecordCapacity,
    BlockingHold,
    AsyncHold,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectsTick {
    Fired,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EffectsTimerKey {
    Alarm,
}

pub struct Effects;

/// Per-test gate for [`EffectsIoRequest::BlockingHold`]: no busy-wait and no process-wide statics.
pub struct BlockingHoldLatch {
    pub(crate) started: mpsc::SyncSender<()>,
    pub(crate) release: Mutex<mpsc::Receiver<()>>,
    pub(crate) io_applied: mpsc::SyncSender<()>,
}

/// Per-test gate for async-held IO under a paused clock (shutdown drain tests).
pub struct AsyncHoldLatch {
    pub(crate) started: mpsc::SyncSender<()>,
    pub(crate) release: tokio::sync::Mutex<tokio::sync::mpsc::Receiver<()>>,
    pub(crate) io_applied: mpsc::SyncSender<()>,
}

impl Default for EffectsArguments {
    fn default() -> Self {
        Self {
            capacity: 10,
            blocking_hold: None,
            async_hold: None,
            record_capacity_done: None,
        }
    }
}

impl Service for EffectsService {
    type Domain = Effects;
    type Context = EffectsContext;
    type Arguments = EffectsArguments;

    const NAME: &'static str = "effects";
    const VERSION: &'static str = "1.0.0";

    fn context(service: &ServiceContext<EffectsArguments>) -> Result<EffectsContext, ServiceError> {
        Ok(EffectsContext {
            expected_capacity: service.arguments().capacity,
            blocking_hold: service.arguments().blocking_hold.clone(),
            async_hold: service.arguments().async_hold.clone(),
            record_capacity_done: service.arguments().record_capacity_done.clone(),
        })
    }

    fn build(
        service: &ServiceContext<EffectsArguments>,
        _context: &EffectsContext,
    ) -> Result<ServiceBuilder<Effects, Self::Context>, ServiceError> {
        let capacity = service.arguments().capacity;
        let blocking_io_applied = service
            .arguments()
            .blocking_hold
            .as_ref()
            .map(|latch| latch.io_applied.clone())
            .or_else(|| {
                service
                    .arguments()
                    .async_hold
                    .as_ref()
                    .map(|latch| latch.io_applied.clone())
            });
        Ok(register_effects_commands(
            ServiceBuilder::new(EffectsSnapshot {
                level: 0,
                capacity,
                failed_io_requests: 0,
                succeeded_io_requests: 0,
                last_failed_request: None,
                tick_count: 0,
                blocking_io_running: false,
                blocking_io_applied,
            })
            .blocking_io(|io_context: &EffectsContext, _snapshot, request| {
                run_blocking_hold_io(io_context, request)
            })
            .io(
                |io_context: &EffectsContext, snapshot: &EffectsSnapshot, request| {
                    let expected_capacity = io_context.expected_capacity;
                    let snapshot_capacity = snapshot.capacity;
                    let record_capacity_done = io_context.record_capacity_done.clone();
                    let async_hold = io_context.async_hold.clone();
                    async move {
                        run_effects_io_request(
                            request,
                            expected_capacity,
                            snapshot_capacity,
                            record_capacity_done,
                            async_hold,
                        )
                        .await
                    }
                },
            ),
        ))
    }
}

async fn run_effects_io_request(
    request: EffectsIoRequest,
    expected_capacity: u8,
    snapshot_capacity: u8,
    record_capacity_done: Option<mpsc::SyncSender<()>>,
    async_hold: Option<Arc<AsyncHoldLatch>>,
) -> Result<Option<EffectsIoResult>, IoError> {
    match request {
        EffectsIoRequest::Fail => Err(IoError::new("the first IO step failed")),
        EffectsIoRequest::Succeed => Ok(Some(EffectsIoResult::Succeeded)),
        EffectsIoRequest::Panic => {
            PANIC_GUARD.fetch_add(1, Ordering::SeqCst);
            panic!("io panic test");
        }
        EffectsIoRequest::RecordCapacity => {
            assert_eq!(expected_capacity, snapshot_capacity);
            if let Some(done) = record_capacity_done {
                let _ = done.send(());
            }
            Ok(None)
        }
        EffectsIoRequest::BlockingHold => {
            Err(IoError::new("blocking IO belongs on a blocking thread"))
        }
        EffectsIoRequest::AsyncHold => {
            let Some(latch) = &async_hold else {
                return Err(IoError::new("AsyncHold requires a per-test AsyncHoldLatch"));
            };
            latch
                .started
                .send(())
                .map_err(|_| IoError::new("the test stopped waiting for async IO to start"))?;
            let mut release = latch.release.lock().await;
            release
                .recv()
                .await
                .ok_or_else(|| IoError::new("the test stopped before releasing async IO"))?;
            Ok(Some(EffectsIoResult::Succeeded))
        }
    }
}

fn run_blocking_hold_io(
    io_context: &EffectsContext,
    request: EffectsIoRequest,
) -> Result<Option<EffectsIoResult>, IoError> {
    match request {
        EffectsIoRequest::BlockingHold => {
            let Some(latch) = &io_context.blocking_hold else {
                return Err(IoError::new(
                    "BlockingHold requires a per-test BlockingHoldLatch",
                ));
            };
            latch
                .started
                .send(())
                .map_err(|_| IoError::new("the test stopped waiting for blocking IO to start"))?;
            lock_unpoisoned(&latch.release)
                .recv()
                .map_err(|_| IoError::new("the test stopped before releasing blocking IO"))?;
            Ok(Some(EffectsIoResult::Succeeded))
        }
        EffectsIoRequest::Fail
        | EffectsIoRequest::Succeed
        | EffectsIoRequest::Panic
        | EffectsIoRequest::RecordCapacity
        | EffectsIoRequest::AsyncHold => Err(IoError::new("not a blocking IO request")),
    }
}

fn register_effects_commands(
    builder: ServiceBuilder<Effects, EffectsContext>,
) -> ServiceBuilder<Effects, EffectsContext> {
    builder
        .command("RunIoChain", |_: Empty| Ok(EffectsRequest::RunIoChain))
        .command("RunIoPanic", |_: Empty| Ok(EffectsRequest::RunIoPanic))
        .command("ArmTimer", |_: Empty| {
            Ok(EffectsRequest::ArmTimer {
                after: Duration::from_secs(10),
            })
        })
        .command("ReArmTimer", |_: Empty| {
            Ok(EffectsRequest::ReArmTimer {
                after: Duration::from_secs(5),
            })
        })
        .command("CancelTimer", |_: Empty| Ok(EffectsRequest::CancelTimer))
        .command("ScheduleIoWithoutExecutor", |_: Empty| {
            Ok(EffectsRequest::ScheduleIoWithoutExecutor)
        })
        .command("RecordCapacity", |_: Empty| {
            Ok(EffectsRequest::RecordCapacity)
        })
        .command("RunBlockingHold", |_: Empty| {
            Ok(EffectsRequest::RunBlockingHold)
        })
        .command("RunAsyncHold", |_: Empty| Ok(EffectsRequest::RunAsyncHold))
        .query(
            "blocking_active",
            |_: Empty| Ok(EffectsQuery::BlockingActive),
            |active: bool| {
                Some(LevelResponse {
                    level: u8::from(active),
                    max_level: 0,
                })
            },
        )
        .state("io", |snapshot: &EffectsSnapshot| LevelResponse {
            level: snapshot.failed_io_requests,
            max_level: snapshot.succeeded_io_requests,
        })
        .state("ticks", |snapshot: &EffectsSnapshot| LevelResponse {
            level: snapshot.tick_count,
            max_level: 0,
        })
}
