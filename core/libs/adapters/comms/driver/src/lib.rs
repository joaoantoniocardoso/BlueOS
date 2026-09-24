use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

mod frame;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Endpoint {
    Local,
    Remote { url: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Stream,
    State,
    Rpc,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sample {
    pub key: String,
    pub payload: Vec<u8>,
    pub correlation: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    pub correlation: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum CommsError {
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, CommsError>;

type RpcHandler = Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send>;
type RpcCall = Box<dyn Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync>;
type PutCall = Arc<dyn Fn(&str, &[u8], u32) -> Result<()> + Send + Sync>;

pub trait Driver: Send {
    fn declare(&mut self, key: &str, kind: Kind) -> Result<()>;
    fn send(&self, key: &str, payload: &[u8], correlation: u32) -> Result<()>;
    fn run(&mut self, dispatcher: &mut Dispatcher) -> Result<()>;
    fn rpc_client(&self, key: &str) -> RpcClient;
    fn put_client(&self) -> PutClient;
}

#[derive(Default)]
pub struct Dispatcher {
    watches: HashMap<String, Box<dyn Fn(Sample) + Send>>,
    rpcs: HashMap<String, RpcHandler>,
    rpc_reply: Option<Result<Vec<u8>>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn dispatch(&mut self, key: &str, frame: Frame) {
        if let Some(rpc) = self.rpcs.get(key) {
            self.rpc_reply = Some(rpc(&frame.payload));
            return;
        }
        if let Some(watch) = self.watches.get(key) {
            watch(Sample {
                key: key.to_string(),
                payload: frame.payload,
                correlation: frame.correlation,
            });
        }
    }

    pub fn register_watch(&mut self, key: &str, callback: Box<dyn Fn(Sample) + Send>) {
        self.watches.insert(key.to_string(), callback);
    }

    pub fn register_rpc(&mut self, key: &str, callback: RpcHandler) {
        self.rpcs.insert(key.to_string(), callback);
    }

    pub fn take_rpc_reply(&mut self) -> Option<Result<Vec<u8>>> {
        self.rpc_reply.take()
    }
}

pub struct RpcClient {
    call: RpcCall,
}

impl RpcClient {
    pub fn from_fn<F>(call: F) -> Self
    where
        F: Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync + 'static,
    {
        Self {
            call: Box::new(call),
        }
    }

    pub fn call(&self, payload: &[u8]) -> Result<Vec<u8>> {
        (self.call)(payload)
    }
}

#[derive(Clone)]
pub struct PutClient {
    put: PutCall,
}

impl PutClient {
    pub fn from_fn<F>(put: F) -> Self
    where
        F: Fn(&str, &[u8], u32) -> Result<()> + Send + Sync + 'static,
    {
        Self { put: Arc::new(put) }
    }

    pub fn send(&self, key: &str, payload: &[u8], correlation: u32) -> Result<()> {
        (self.put)(key, payload, correlation)
    }
}
