use core::{convert::Infallible, time::Duration};

use blueos_domain::{Command, Decision, Domain, DomainQueries, Effect, IoError, Now, Outcome};
use blueos_idl::msg::{blueos_example_msgs::LevelResponse, std_msgs::Empty};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError};

use super::service::*;

pub struct EffectsWithoutIoService;

impl Domain for Effects {
    type Snapshot = EffectsSnapshot;
    type Request = EffectsRequest;
    type IoResult = EffectsIoResult;
    type Tick = EffectsTick;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = EffectsIoRequest;
    type TimerKey = EffectsTimerKey;

    fn handle(
        snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(request) => Effects::handle_request(snapshot, request),
            Command::IoResult(result) => Effects::handle_io_result(snapshot, result),
            Command::Tick(tick) => Effects::handle_tick(snapshot, tick),
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        Command::IoResult(EffectsIoResult::Failed {
            request,
            message: error.message().to_owned(),
        })
    }

    fn io_runs_on_blocking_thread(request: &Self::IoRequest) -> bool {
        matches!(request, EffectsIoRequest::BlockingHold)
    }
}

impl DomainQueries for Effects {
    type Query = EffectsQuery;
    type Response = bool;

    fn query(snapshot: &Self::Snapshot, query: Self::Query, _now: Now) -> Self::Response {
        match query {
            EffectsQuery::BlockingActive => snapshot.blocking_io_running,
        }
    }
}

impl Effects {
    fn applied(
        effects: Vec<Effect<EffectsTick, EffectsIoRequest, EffectsTimerKey>>,
    ) -> Decision<Effects> {
        Outcome::Applied {
            events: Vec::new(),
            effects,
        }
    }

    fn handle_request(
        snapshot: &mut EffectsSnapshot,
        request: EffectsRequest,
    ) -> Decision<Effects> {
        match request {
            EffectsRequest::RunIoChain => Self::applied(vec![
                Effect::Io(EffectsIoRequest::Fail),
                Effect::Io(EffectsIoRequest::Succeed),
            ]),
            EffectsRequest::RunIoPanic => Self::applied(vec![Effect::Io(EffectsIoRequest::Panic)]),
            EffectsRequest::ArmTimer { after } | EffectsRequest::ReArmTimer { after } => {
                Self::applied(vec![Effect::Schedule {
                    after,
                    key: EffectsTimerKey::Alarm,
                    command: EffectsTick::Fired,
                }])
            }
            EffectsRequest::CancelTimer => Self::applied(vec![
                Effect::Schedule {
                    after: Duration::from_secs(10),
                    key: EffectsTimerKey::Alarm,
                    command: EffectsTick::Fired,
                },
                Effect::Cancel(EffectsTimerKey::Alarm),
            ]),
            EffectsRequest::ScheduleIoWithoutExecutor | EffectsRequest::RecordCapacity => {
                Self::applied(vec![Effect::Io(EffectsIoRequest::RecordCapacity)])
            }
            EffectsRequest::RunBlockingHold | EffectsRequest::RunAsyncHold => {
                snapshot.blocking_io_running = true;
                let io_request = match request {
                    EffectsRequest::RunBlockingHold => EffectsIoRequest::BlockingHold,
                    EffectsRequest::RunAsyncHold => EffectsIoRequest::AsyncHold,
                    _ => unreachable!(),
                };
                Self::applied(vec![Effect::Io(io_request)])
            }
        }
    }

    fn handle_io_result(
        snapshot: &mut EffectsSnapshot,
        result: EffectsIoResult,
    ) -> Decision<Effects> {
        match result {
            EffectsIoResult::Failed {
                request,
                message: _,
            } => {
                snapshot.failed_io_requests += 1;
                snapshot.last_failed_request = Some(request);
                Self::applied(Vec::new())
            }
            EffectsIoResult::Succeeded => {
                snapshot.succeeded_io_requests += 1;
                snapshot.blocking_io_running = false;
                if let Some(applied) = snapshot.blocking_io_applied.take() {
                    let _ = applied.send(());
                }
                Self::applied(Vec::new())
            }
        }
    }

    fn handle_tick(snapshot: &mut EffectsSnapshot, tick: EffectsTick) -> Decision<Effects> {
        let _ = tick;
        snapshot.tick_count += 1;
        Self::applied(Vec::new())
    }
}

impl Service for EffectsWithoutIoService {
    type Domain = Effects;
    type Context = ();
    type Arguments = EffectsArguments;

    const NAME: &'static str = "effects";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<EffectsArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<EffectsArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<Effects, Self::Context>, ServiceError> {
        let capacity = service.arguments().capacity;
        Ok(ServiceBuilder::new(EffectsSnapshot {
            level: 7,
            capacity,
            failed_io_requests: 0,
            succeeded_io_requests: 0,
            last_failed_request: None,
            tick_count: 0,
            blocking_io_running: false,
            blocking_io_applied: None,
        })
        .command("ScheduleIoWithoutExecutor", |_: Empty| {
            Ok(EffectsRequest::ScheduleIoWithoutExecutor)
        })
        .state("level", |snapshot: &EffectsSnapshot| LevelResponse {
            level: snapshot.level,
            max_level: 0,
        }))
    }
}
