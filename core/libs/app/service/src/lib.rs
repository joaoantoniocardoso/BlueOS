use std::collections::VecDeque;
use std::sync::Arc;

use blueos_cli::Argv;
use blueos_comms::{CommsError, PutClient, Sample, Session};
use blueos_configs::Configs;
use blueos_cqrs::{App, Domain, Effect};
use blueos_logging::error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Comms(#[from] blueos_comms::CommsError),
    #[error(transparent)]
    App(#[from] blueos_cqrs::AppError),
    #[error("{0}")]
    Message(String),
}

pub struct Adapters {
    pub comms: Session,
    pub configs: Configs,
    pub cli: Argv,
}

type WatchFn<D> = Box<dyn Fn(Sample) -> Option<<D as Domain>::Command> + Send>;
type RpcMapFn<D> = Box<dyn Fn(&[u8]) -> Option<<D as Domain>::Command> + Send>;
type CliFn<D> = Box<dyn Fn(&[String]) -> Option<<D as Domain>::Command> + Send>;
type IoThen<C> = Box<dyn FnOnce(Vec<u8>) -> Option<C> + Send>;
type IoFn<D> =
    dyn Fn(&App<D>, <D as Domain>::IoRequest) -> IoPlan<<D as Domain>::Command> + Send + Sync;
type FeedOut<D> = (Vec<<D as Domain>::Event>, Option<Vec<u8>>);
type ApplyOut<D> = (Vec<<D as Domain>::Command>, Option<Vec<u8>>);

pub enum IoPlan<C> {
    Rpc {
        key: String,
        payload: Vec<u8>,
        then: IoThen<C>,
    },
    Local(Option<C>),
}

pub struct Service<D: Domain> {
    pub app: App<D>,
    pub adapters: Adapters,
    watches: Vec<(String, WatchFn<D>)>,
    rpcs: Vec<(String, RpcMapFn<D>)>,
    cli_command: Option<CliFn<D>>,
    io_executor: Option<Arc<IoFn<D>>>,
}

// ponytail: Dispatcher callbacks run on the Session::run caller thread; a Mutex<App> would need
// Snapshot: Default to move `pub app` out. Upgrade if run() is spawned onto another thread.
struct AppSlot<D: Domain>(*mut App<D>);
struct SessionSlot(*mut Session);

// SAFETY: driver crates dispatch on the thread that called Session::run.
unsafe impl<D: Domain> Send for AppSlot<D> {}
unsafe impl Send for SessionSlot {}

impl<D: Domain> Copy for AppSlot<D> {}
impl Copy for SessionSlot {}

impl<D: Domain> Clone for AppSlot<D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl Clone for SessionSlot {
    fn clone(&self) -> Self {
        *self
    }
}

impl<D: Domain> Service<D> {
    pub fn new(app: App<D>, adapters: Adapters) -> Self {
        let _ = blueos_logging::init(0); // already-initialized is success
        Self {
            app,
            adapters,
            watches: Vec::new(),
            rpcs: Vec::new(),
            cli_command: None,
            io_executor: None,
        }
    }

    pub fn handle(&mut self, command: D::Command) -> Result<Vec<D::Event>, ServiceError> {
        let put_client = self.adapters.comms.put_client();
        let call_rpc = |key: &str, payload: &[u8]| {
            self.adapters
                .comms
                .rpc_client(key)
                .call(payload)
                .map_err(ServiceError::from)
        };
        let io_executor = self.io_executor.as_deref();
        let (events, _reply) = feed(&mut self.app, command, &call_rpc, &put_client, io_executor)?;
        Ok(events)
    }

    pub fn apply_effects(
        &mut self,
        effects: Vec<Effect<D::IoRequest>>,
    ) -> Result<(), ServiceError> {
        let put_client = self.adapters.comms.put_client();
        let call_rpc = |key: &str, payload: &[u8]| {
            self.adapters
                .comms
                .rpc_client(key)
                .call(payload)
                .map_err(ServiceError::from)
        };
        let io_executor = self.io_executor.as_deref();
        let (follow, _reply) = apply(&self.app, effects, &call_rpc, &put_client, io_executor)?;
        for command in follow {
            self.handle(command)?;
        }
        Ok(())
    }

    pub fn watch(
        &mut self,
        key: &str,
        callback: impl Fn(Sample) -> Option<D::Command> + Send + 'static,
    ) {
        self.watches.push((key.to_string(), Box::new(callback)));
    }

    pub fn on_rpc(
        &mut self,
        key: &str,
        callback: impl Fn(&[u8]) -> Option<D::Command> + Send + 'static,
    ) {
        self.rpcs.push((key.to_string(), Box::new(callback)));
    }

    pub fn on_cli(&mut self, callback: impl Fn(&[String]) -> Option<D::Command> + Send + 'static) {
        self.cli_command = Some(Box::new(callback));
    }

    pub fn on_io<F>(&mut self, callback: F)
    where
        F: Fn(&App<D>, D::IoRequest) -> IoPlan<D::Command> + Send + Sync + 'static,
    {
        self.io_executor = Some(Arc::new(callback));
    }

    pub fn run(&mut self) -> Result<(), ServiceError>
    where
        D: 'static,
        D::Command: 'static,
    {
        let args = self.adapters.cli.args.clone();
        let put_client = self.adapters.comms.put_client();
        let watches = std::mem::take(&mut self.watches);
        let rpcs = std::mem::take(&mut self.rpcs);
        let cli_command = self.cli_command.take();
        let io_executor = self.io_executor.clone();
        let slot = AppSlot(&mut self.app);
        let comms = SessionSlot(&mut self.adapters.comms);
        for (key, to_command) in watches {
            let put_client = put_client.clone();
            let io_executor = io_executor.clone();
            self.adapters.comms.watch(&key, move |sample| {
                if let Some(command) = to_command(sample)
                    && let Err(error) = tick(slot, comms, command, &put_client, io_executor.clone())
                {
                    error!("{error}");
                }
            })?;
        }
        for (key, to_command) in rpcs {
            let put_client = put_client.clone();
            let io_executor = io_executor.clone();
            self.adapters.comms.on_rpc(&key, move |payload| {
                let command =
                    to_command(payload).ok_or_else(|| CommsError::Message("rpc payload".into()))?;
                match tick(slot, comms, command, &put_client, io_executor.clone()) {
                    Ok(Some(bytes)) => Ok(bytes),
                    Ok(None) => Ok(Vec::new()),
                    Err(error) => Err(CommsError::Message(error.to_string())),
                }
            })?;
        }
        if let Some(from_cli) = cli_command
            && let Some(command) = from_cli(&args)
        {
            tick(slot, comms, command, &put_client, io_executor)?;
        }
        self.adapters.comms.run().map_err(ServiceError::from)
    }
}

fn tick<D: Domain>(
    slot: AppSlot<D>,
    comms: SessionSlot,
    command: D::Command,
    put_client: &PutClient,
    io_executor: Option<Arc<IoFn<D>>>,
) -> Result<Option<Vec<u8>>, ServiceError> {
    let call_rpc = |key: &str, payload: &[u8]| {
        // SAFETY: `run` does not drop Session while Dispatcher callbacks run on this thread.
        unsafe { &*comms.0 }
            .rpc_client(key)
            .call(payload)
            .map_err(ServiceError::from)
    };
    // SAFETY: `run` does not drop `self.app` while Dispatcher callbacks run on this thread.
    let app = unsafe { &mut *slot.0 };
    let (_events, reply) = feed(app, command, &call_rpc, put_client, io_executor.as_deref())?;
    Ok(reply)
}

fn feed<D, R>(
    app: &mut App<D>,
    command: D::Command,
    call_rpc: &R,
    put_client: &PutClient,
    io_executor: Option<&IoFn<D>>,
) -> Result<FeedOut<D>, ServiceError>
where
    D: Domain,
    R: Fn(&str, &[u8]) -> Result<Vec<u8>, ServiceError>,
{
    let mut events = Vec::new();
    let mut reply = None;
    let mut queue = VecDeque::new();
    queue.push_back(command);
    while let Some(command) = queue.pop_front() {
        let (command_events, effects) = app.handle(command);
        events.extend(command_events);
        let (follow, next_reply) = apply(app, effects, call_rpc, put_client, io_executor)?;
        if next_reply.is_some() {
            reply = next_reply;
        }
        queue.extend(follow);
    }
    Ok((events, reply))
}

fn apply<D, R>(
    app: &App<D>,
    effects: Vec<Effect<D::IoRequest>>,
    call_rpc: &R,
    put_client: &PutClient,
    io_executor: Option<&IoFn<D>>,
) -> Result<ApplyOut<D>, ServiceError>
where
    D: Domain,
    R: Fn(&str, &[u8]) -> Result<Vec<u8>, ServiceError>,
{
    let mut commands = Vec::new();
    let mut reply = None;
    for effect in effects {
        match effect {
            Effect::None => {}
            Effect::Io(io_request) => {
                if let Some(command) = io_via_rpc(app, call_rpc, io_request, io_executor)? {
                    commands.push(command);
                }
            }
            Effect::Publish { key, payload } => put_client.send(&key, &payload, 0)?,
            Effect::Reply {
                correlation: _,
                payload,
            } => reply = Some(payload),
        }
    }
    Ok((commands, reply))
}

fn io_via_rpc<D, R>(
    app: &App<D>,
    call_rpc: &R,
    io_request: D::IoRequest,
    io_executor: Option<&IoFn<D>>,
) -> Result<Option<D::Command>, ServiceError>
where
    D: Domain,
    R: Fn(&str, &[u8]) -> Result<Vec<u8>, ServiceError>,
{
    let Some(executor) = io_executor else {
        return Err(ServiceError::Message("Effect::Io has no adapter".into()));
    };
    match executor(app, io_request) {
        IoPlan::Rpc { key, payload, then } => match call_rpc(&key, &payload) {
            Ok(reply) => Ok(then(reply)),
            Err(_) => Ok(then(b"FAILED".to_vec())),
        },
        IoPlan::Local(command) => Ok(command),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blueos_comms::ChannelDriver;
    use blueos_jobs::{JobId, JobSpec, Jobs};
    use std::sync::{Arc, Mutex};
    use std::thread;

    struct TestDomain;

    #[derive(Default)]
    struct TestSnapshot {
        heard: usize,
    }

    enum TestCommand {
        Go,
        Ping,
        Heard,
    }

    enum TestEvent {
        Ok,
    }

    enum TestQuery {}
    struct TestView;
    enum TestIo {
        Echo,
    }

    impl Domain for TestDomain {
        type Command = TestCommand;
        type Event = TestEvent;
        type Query = TestQuery;
        type View = TestView;
        type Snapshot = TestSnapshot;
        type IoRequest = TestIo;

        fn handle_command(
            snapshot: &mut Self::Snapshot,
            _jobs: &mut Jobs,
            command: Self::Command,
        ) -> (Vec<Self::Event>, Vec<Effect<Self::IoRequest>>) {
            match command {
                TestCommand::Go => (
                    vec![TestEvent::Ok],
                    vec![Effect::Publish {
                        key: "s".into(),
                        payload: b"hi".to_vec(),
                    }],
                ),
                TestCommand::Ping => (vec![TestEvent::Ok], vec![Effect::Io(TestIo::Echo)]),
                TestCommand::Heard => {
                    snapshot.heard += 1;
                    (vec![TestEvent::Ok], vec![Effect::None])
                }
            }
        }

        fn handle_query(
            _snapshot: &Self::Snapshot,
            _jobs: &Jobs,
            _query: Self::Query,
        ) -> Self::View {
            TestView
        }

        fn io_from_job(_job_id: JobId, _job_spec: &JobSpec) -> Self::IoRequest {
            TestIo::Echo
        }
    }

    fn adapters(session: Session) -> Adapters {
        Adapters {
            comms: session,
            configs: Configs::default(),
            cli: Argv::default(),
        }
    }

    #[test]
    fn handle_publish_with_injected_driver() {
        let (local_driver, peer_driver) = ChannelDriver::pair();
        let mut service: Service<TestDomain> = Service::new(
            App::new(TestSnapshot::default()),
            adapters(Session::with_driver(Box::new(local_driver))),
        );
        let mut peer = Session::with_driver(Box::new(peer_driver));
        let got = Arc::new(Mutex::new(None));
        let slot = Arc::clone(&got);
        peer.watch("s", move |sample| {
            *slot.lock().unwrap() = Some(sample);
        })
        .unwrap();
        let join = thread::spawn(move || peer.run());
        service.handle(TestCommand::Go).unwrap();
        drop(service);
        join.join().expect("peer").unwrap();
        let sample = got.lock().unwrap().clone().expect("publish");
        assert_eq!(sample.payload, b"hi");
    }

    #[test]
    fn handle_io_via_rpc_client() {
        let (local_driver, peer_driver) = ChannelDriver::pair();
        let mut service: Service<TestDomain> = Service::new(
            App::new(TestSnapshot::default()),
            adapters(Session::with_driver(Box::new(local_driver))),
        );
        service.on_io(|_app, io_request| {
            let TestIo::Echo = io_request;
            IoPlan::Rpc {
                key: "echo".into(),
                payload: b"ping".to_vec(),
                then: Box::new(|reply| {
                    assert_eq!(reply, b"pong");
                    Some(TestCommand::Heard)
                }),
            }
        });
        let mut peer = Session::with_driver(Box::new(peer_driver));
        peer.on_rpc("echo", |_| Ok(b"pong".to_vec())).unwrap();
        let join = thread::spawn(move || peer.run());
        service.handle(TestCommand::Ping).unwrap();
        assert_eq!(service.app.snapshot.heard, 1);
        drop(service);
        join.join().expect("peer").unwrap();
    }

    #[test]
    fn run_watch_enqueues_command() {
        let (local_driver, peer_driver) = ChannelDriver::pair();
        let mut service: Service<TestDomain> = Service::new(
            App::new(TestSnapshot::default()),
            adapters(Session::with_driver(Box::new(local_driver))),
        );
        service.watch("in", |_| Some(TestCommand::Heard));
        let peer = Session::with_driver(Box::new(peer_driver));
        peer.send("in", b"x", 0).unwrap();
        drop(peer);
        service.run().unwrap();
        assert_eq!(service.app.snapshot.heard, 1);
    }
}
