//! A test varies a Service only through its Context: `Harness::start_with` changes it between `context` and
//! `build` (layer L4).

use core::convert::Infallible;
use std::{path::PathBuf, sync::Arc};

use blueos_domain::{Command, Decision, Domain, Effect, IoError, Now, Outcome};
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};

/// Reads the gauge's sensor: the Port a test replaces.
type ReadSensor = Arc<dyn Fn() -> u8 + Send + Sync>;

struct GaugeService;

#[derive(Clone, Default, clap::Args)]
struct GaugeArguments;

struct GaugeContext {
    read_sensor: ReadSensor,
    max_level: u8,
    settings_path: Option<PathBuf>,
}

struct Gauge;

#[derive(Clone, Default)]
struct GaugeSnapshot {
    level: u8,
}

enum GaugeRequest {
    ReadLevel,
}

#[derive(Clone, Debug, PartialEq)]
enum GaugeIoRequest {
    ReadSensor,
}

#[derive(Clone, Debug, PartialEq)]
enum GaugeIoResult {
    Level(u8),
}

impl Service for GaugeService {
    type Domain = Gauge;
    type Context = GaugeContext;
    type Arguments = GaugeArguments;

    const NAME: &'static str = "gauge";
    const VERSION: &'static str = "1.0.0";

    fn context(service: &ServiceContext<GaugeArguments>) -> Result<GaugeContext, ServiceError> {
        Ok(GaugeContext {
            read_sensor: Arc::new(|| 42),
            max_level: 100,
            settings_path: service.settings_path().map(PathBuf::from),
        })
    }

    fn build(
        _service: &ServiceContext<GaugeArguments>,
        context: &GaugeContext,
    ) -> Result<ServiceBuilder<Gauge, GaugeContext>, ServiceError> {
        let max_level = context.max_level;
        Ok(ServiceBuilder::new(GaugeSnapshot::default())
            .io(|io_context: &GaugeContext, _snapshot, request| {
                let level = match request {
                    GaugeIoRequest::ReadSensor => (io_context.read_sensor)(),
                };
                async move { Ok(Some(GaugeIoResult::Level(level))) }
            })
            .command("ReadLevel", |_: EmptyRequest| Ok(GaugeRequest::ReadLevel))
            .state("gauge", move |snapshot: &GaugeSnapshot| {
                LevelQueryResponse {
                    level: snapshot.level,
                    max_level,
                }
            }))
    }
}

impl Domain for Gauge {
    type Snapshot = GaugeSnapshot;
    type Request = GaugeRequest;
    type Event = Infallible;
    type IoResult = GaugeIoResult;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = GaugeIoRequest;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut GaugeSnapshot,
        command: Command<GaugeRequest, GaugeIoResult, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(GaugeRequest::ReadLevel) => Outcome::Applied {
                events: Vec::new(),
                effects: vec![Effect::Io(GaugeIoRequest::ReadSensor)],
            },
            Command::IoResult(GaugeIoResult::Level(level)) => {
                snapshot.level = level;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Tick(never) => match never {},
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        request: GaugeIoRequest,
        _error: IoError,
    ) -> Command<GaugeRequest, GaugeIoResult, Infallible, Infallible> {
        match request {
            GaugeIoRequest::ReadSensor => Command::IoResult(GaugeIoResult::Level(0)),
        }
    }
}

#[tokio::test(start_paused = true)]
async fn start_runs_the_context_that_ships() {
    let harness = Harness::<GaugeService>::start(GaugeArguments)
        .await
        .unwrap();

    let ack = harness.send("ReadLevel", &EmptyRequest::default()).await;

    assert!(ack.accepted);
    let gauge = harness.state::<LevelQueryResponse>("gauge").await;
    assert_eq!((gauge.level, gauge.max_level), (42, 100));
}

#[tokio::test(start_paused = true)]
async fn start_gives_the_service_a_settings_path_under_a_temporary_directory() {
    let mut settings_paths = Vec::new();
    for _ in 0..2 {
        let mut settings_path = None;
        let _harness = Harness::<GaugeService>::start_with(GaugeArguments, |context| {
            settings_path = context.settings_path.clone();
        })
        .await
        .unwrap();
        settings_paths.push(settings_path.expect("the harness supplies a settings path"));
    }

    for settings_path in &settings_paths {
        assert!(
            settings_path.starts_with(std::env::temp_dir()),
            "{settings_path:?} is outside the temporary directory"
        );
    }
    assert_ne!(settings_paths[0], settings_paths[1]);
}

#[tokio::test(start_paused = true)]
async fn start_with_replaces_a_port_and_a_field_before_build() {
    let harness = Harness::<GaugeService>::start_with(GaugeArguments, |context| {
        context.read_sensor = Arc::new(|| 7);
        context.max_level = 50;
    })
    .await
    .unwrap();

    let ack = harness.send("ReadLevel", &EmptyRequest::default()).await;

    assert!(ack.accepted);
    let gauge = harness.state::<LevelQueryResponse>("gauge").await;
    assert_eq!((gauge.level, gauge.max_level), (7, 50));
}
