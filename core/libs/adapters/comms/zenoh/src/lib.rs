use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use blueos_comms_driver::{
    CommsError, Dispatcher, Driver, Endpoint, Frame, Kind, PutClient, Result, RpcClient,
};
use zenoh::Wait;

enum ZenohMessage {
    Pub {
        key: String,
        frame: Frame,
    },
    Rpc {
        key: String,
        frame: Frame,
        query: zenoh::query::Query,
    },
}

struct ZenohInner {
    session: zenoh::Session,
    sender: Sender<ZenohMessage>,
    correlation: AtomicU32,
}

impl ZenohInner {
    fn put(&self, key: &str, payload: &[u8], correlation: u32) -> Result<()> {
        let wire = Frame {
            correlation,
            payload: payload.to_vec(),
        }
        .encode();
        self.session.put(key, wire).wait().map_err(zenoh_error)?;
        Ok(())
    }

    fn rpc_call(&self, key: &str, payload: &[u8]) -> Result<Vec<u8>> {
        let correlation = self.correlation.fetch_add(1, Ordering::Relaxed);
        let wire = Frame {
            correlation,
            payload: payload.to_vec(),
        }
        .encode();
        let replies = self
            .session
            .get(key)
            .payload(wire)
            .wait()
            .map_err(zenoh_error)?;
        let reply = replies.recv().map_err(zenoh_error)?;
        match reply.into_result() {
            Ok(sample) => {
                let bytes = sample.payload().to_bytes();
                Ok(Frame::decode(&bytes)?.payload)
            }
            Err(error) => Err(CommsError::Message(error.to_string())),
        }
    }
}

fn zenoh_error(error: impl ToString) -> CommsError {
    CommsError::Message(error.to_string())
}

fn zenoh_config(endpoint: Endpoint) -> Result<zenoh::Config> {
    match endpoint {
        Endpoint::Local => {
            let mut config = zenoh::Config::default();
            // Local peers rendezvous on tcp/127.0.0.1:7447. Default listen is tcp/[::]:0 plus
            // multicast scouting; both fail when IPv6 is off or multicast does not loop.
            // First Local listens; later Locals skip listen (exit_on_failure false) and connect.
            // No zenohd.
            config
                .insert_json5("listen/endpoints", r#"["tcp/127.0.0.1:7447"]"#)
                .map_err(zenoh_error)?;
            config
                .insert_json5("listen/exit_on_failure", "false")
                .map_err(zenoh_error)?;
            config
                .insert_json5("connect/endpoints", r#"["tcp/127.0.0.1:7447"]"#)
                .map_err(zenoh_error)?;
            config
                .insert_json5("connect/timeout_ms", "1000")
                .map_err(zenoh_error)?;
            config
                .insert_json5("connect/exit_on_failure", "false")
                .map_err(zenoh_error)?;
            config
                .insert_json5("scouting/multicast/enabled", "false")
                .map_err(zenoh_error)?;
            Ok(config)
        }
        Endpoint::Remote { url } => {
            let mut config = zenoh::Config::default();
            let endpoints = format!("[\"{}\"]", url);
            config
                .insert_json5("connect/endpoints", &endpoints)
                .map_err(zenoh_error)?;
            Ok(config)
        }
    }
}

pub struct ZenohDriver {
    inner: Arc<ZenohInner>,
    receiver: Mutex<Option<Receiver<ZenohMessage>>>,
}

impl ZenohDriver {
    pub fn connect(endpoint: Endpoint) -> Result<Self> {
        let config = zenoh_config(endpoint)?;
        let session = zenoh::open(config).wait().map_err(zenoh_error)?;
        let (sender, receiver) = mpsc::channel();
        Ok(Self {
            inner: Arc::new(ZenohInner {
                session,
                sender,
                correlation: AtomicU32::new(1),
            }),
            receiver: Mutex::new(Some(receiver)),
        })
    }
}

impl Driver for ZenohDriver {
    fn declare(&mut self, key: &str, kind: Kind) -> Result<()> {
        let sender = self.inner.sender.clone();
        let key_owned = key.to_string();
        match kind {
            Kind::Stream | Kind::State => {
                self.inner
                    .session
                    .declare_subscriber(key)
                    .callback(move |sample| {
                        let bytes = sample.payload().to_bytes();
                        if let Ok(frame) = Frame::decode(&bytes) {
                            let _ = sender.send(ZenohMessage::Pub {
                                key: key_owned.clone(),
                                frame,
                            });
                        }
                    })
                    .background()
                    .wait()
                    .map_err(zenoh_error)?;
            }
            Kind::Rpc => {
                self.inner
                    .session
                    .declare_queryable(key)
                    .callback(move |query| {
                        let bytes = query
                            .payload()
                            .map(|payload| payload.to_bytes().into_owned())
                            .unwrap_or_default();
                        let frame = Frame::decode(&bytes).unwrap_or(Frame {
                            correlation: 0,
                            payload: bytes,
                        });
                        let _ = sender.send(ZenohMessage::Rpc {
                            key: key_owned.clone(),
                            frame,
                            query,
                        });
                    })
                    .background()
                    .wait()
                    .map_err(zenoh_error)?;
            }
        }
        Ok(())
    }

    fn send(&self, key: &str, payload: &[u8], correlation: u32) -> Result<()> {
        self.inner.put(key, payload, correlation)
    }

    fn run(&mut self, dispatcher: &mut Dispatcher) -> Result<()> {
        let receiver = self
            .receiver
            .lock()
            .map_err(|error| CommsError::Message(error.to_string()))?
            .take()
            .ok_or_else(|| CommsError::Message("driver already running".into()))?;
        while let Ok(message) = receiver.recv() {
            match message {
                ZenohMessage::Pub { key, frame } => dispatcher.dispatch(&key, frame),
                ZenohMessage::Rpc { key, frame, query } => {
                    dispatcher.dispatch(&key, frame);
                    match dispatcher.take_rpc_reply() {
                        Some(Ok(bytes)) => {
                            let wire = Frame {
                                correlation: 0,
                                payload: bytes,
                            }
                            .encode();
                            let _ = query.reply(&key, wire).wait();
                        }
                        Some(Err(error)) => {
                            let _ = query.reply_err(error.to_string()).wait();
                        }
                        None => {
                            let _ = query.reply_err("no rpc handler").wait();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn rpc_client(&self, key: &str) -> RpcClient {
        let inner = self.inner.clone();
        let key = key.to_string();
        RpcClient::from_fn(move |payload| inner.rpc_call(&key, payload))
    }

    fn put_client(&self) -> PutClient {
        let inner = self.inner.clone();
        PutClient::from_fn(move |key, payload, correlation| inner.put(key, payload, correlation))
    }
}
