//! Comms façade. Services depend on this crate only.
//!
//! `Session::open` selects the Zenoh driver. Tests inject `ChannelDriver` via `Session::with_driver`.
//! Driver crates are never named from apps.

pub use blueos_comms_driver::{CommsError, Endpoint, PutClient, Result, RpcClient, Sample};
use blueos_comms_driver::{Dispatcher, Driver, Kind};

#[cfg(feature = "channel")]
pub use blueos_comms_channel::ChannelDriver;

#[cfg(feature = "zenoh")]
use blueos_comms_zenoh::ZenohDriver;

pub struct Session {
    driver: Box<dyn Driver>,
    dispatcher: Dispatcher,
}

impl Session {
    pub fn with_driver(driver: Box<dyn Driver>) -> Self {
        Self {
            driver,
            dispatcher: Dispatcher::new(),
        }
    }

    pub fn open(endpoint: Endpoint) -> Result<Self> {
        #[cfg(feature = "zenoh")]
        {
            Ok(Self::with_driver(Box::new(ZenohDriver::connect(endpoint)?)))
        }
        #[cfg(not(feature = "zenoh"))]
        {
            let _ = endpoint; // zenoh driver is compiled out
            Err(CommsError::Message(
                "blueos_comms built without feature zenoh".into(),
            ))
        }
    }

    pub fn watch(&mut self, key: &str, callback: impl Fn(Sample) + Send + 'static) -> Result<()> {
        self.dispatcher.register_watch(key, Box::new(callback));
        self.driver.declare(key, Kind::Stream)
    }

    pub fn on_rpc(
        &mut self,
        key: &str,
        callback: impl Fn(&[u8]) -> Result<Vec<u8>> + Send + 'static,
    ) -> Result<()> {
        self.dispatcher.register_rpc(key, Box::new(callback));
        self.driver.declare(key, Kind::Rpc)
    }

    pub fn rpc_client(&self, key: &str) -> RpcClient {
        self.driver.rpc_client(key)
    }

    pub fn put_client(&self) -> PutClient {
        self.driver.put_client()
    }

    pub fn send(&self, key: &str, payload: &[u8], correlation: u32) -> Result<()> {
        self.driver.send(key, payload, correlation)
    }

    pub fn run(&mut self) -> Result<()> {
        self.driver.run(&mut self.dispatcher)
    }
}

#[cfg(all(test, feature = "zenoh"))]
mod tests {
    use super::{Endpoint, Session};

    #[test]
    fn zenoh_two_local_sessions_watch_send() {
        let mut subscriber = Session::open(Endpoint::Local).unwrap();
        let publisher = Session::open(Endpoint::Local).unwrap();
        let (sample_sender, sample_receiver) = std::sync::mpsc::channel();
        subscriber
            .watch("zenoh/two-session/stream", move |sample| {
                let _ = sample_sender.send(sample); // drop if the test already timed out
            })
            .unwrap();
        let _run = std::thread::spawn(move || subscriber.run());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let sample = loop {
            publisher
                .send("zenoh/two-session/stream", b"hello", 9)
                .unwrap();
            match sample_receiver.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(sample) => break sample,
                Err(_) if std::time::Instant::now() < deadline => continue,
                Err(_) => panic!("timed out waiting for zenoh sample"),
            }
        };
        assert_eq!(sample.key, "zenoh/two-session/stream");
        assert_eq!(sample.payload, b"hello");
        assert_eq!(sample.correlation, 9);
    }
}
