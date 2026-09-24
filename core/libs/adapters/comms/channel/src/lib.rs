use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use blueos_comms_driver::{
    CommsError, Dispatcher, Driver, Frame, Kind, PutClient, Result, RpcClient,
};

enum ChannelMessage {
    Pub {
        key: String,
        frame: Frame,
    },
    Rpc {
        key: String,
        frame: Frame,
        reply: Sender<std::result::Result<Vec<u8>, String>>,
    },
}

struct ChannelInner {
    sender: Sender<ChannelMessage>,
    receiver: Mutex<Option<Receiver<ChannelMessage>>>,
    correlation: AtomicU32,
}

impl ChannelInner {
    fn put(&self, key: &str, payload: &[u8], correlation: u32) -> Result<()> {
        let frame = Frame {
            correlation,
            payload: payload.to_vec(),
        };
        self.sender
            .send(ChannelMessage::Pub {
                key: key.to_string(),
                frame,
            })
            .map_err(|_| CommsError::Message("channel closed".into()))
    }

    fn rpc_call(&self, key: &str, payload: &[u8]) -> Result<Vec<u8>> {
        let correlation = self.correlation.fetch_add(1, Ordering::Relaxed);
        let frame = Frame {
            correlation,
            payload: payload.to_vec(),
        };
        let (reply_tx, reply_rx) = mpsc::channel();
        self.sender
            .send(ChannelMessage::Rpc {
                key: key.to_string(),
                frame,
                reply: reply_tx,
            })
            .map_err(|_| CommsError::Message("channel closed".into()))?;
        match reply_rx.recv() {
            Ok(Ok(bytes)) => Ok(bytes),
            Ok(Err(error)) => Err(CommsError::Message(error)),
            Err(_) => Err(CommsError::Message("rpc reply dropped".into())),
        }
    }
}

pub struct ChannelDriver {
    inner: Arc<ChannelInner>,
}

impl ChannelDriver {
    pub fn pair() -> (Self, Self) {
        let (a_to_b_sender, a_to_b_receiver) = mpsc::channel();
        let (b_to_a_sender, b_to_a_receiver) = mpsc::channel();
        let left = Self {
            inner: Arc::new(ChannelInner {
                sender: a_to_b_sender,
                receiver: Mutex::new(Some(b_to_a_receiver)),
                correlation: AtomicU32::new(1),
            }),
        };
        let right = Self {
            inner: Arc::new(ChannelInner {
                sender: b_to_a_sender,
                receiver: Mutex::new(Some(a_to_b_receiver)),
                correlation: AtomicU32::new(1),
            }),
        };
        (left, right)
    }
}

impl Driver for ChannelDriver {
    fn declare(&mut self, key: &str, kind: Kind) -> Result<()> {
        let _ = (key, kind); // in-memory driver has nothing to declare
        Ok(())
    }

    fn send(&self, key: &str, payload: &[u8], correlation: u32) -> Result<()> {
        self.inner.put(key, payload, correlation)
    }

    fn run(&mut self, dispatcher: &mut Dispatcher) -> Result<()> {
        let receiver = self
            .inner
            .receiver
            .lock()
            .map_err(|error| CommsError::Message(error.to_string()))?
            .take()
            .ok_or_else(|| CommsError::Message("driver already running".into()))?;
        while let Ok(message) = receiver.recv() {
            match message {
                ChannelMessage::Pub { key, frame } => dispatcher.dispatch(&key, frame),
                ChannelMessage::Rpc { key, frame, reply } => {
                    dispatcher.dispatch(&key, frame);
                    let reply_result = match dispatcher.take_rpc_reply() {
                        Some(Ok(bytes)) => Ok(bytes),
                        Some(Err(error)) => Err(error.to_string()),
                        None => Err("no rpc handler".into()),
                    };
                    let _ = reply.send(reply_result); // peer gone: drop the RPC
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

#[cfg(test)]
mod tests {
    use super::*;
    use blueos_comms::Session;
    use std::sync::{Arc, Mutex};

    #[test]
    fn channel_pair_watch_send_without_zenoh() {
        let (local_driver, peer_driver) = ChannelDriver::pair();
        let mut local = Session::with_driver(Box::new(local_driver));
        let peer = Session::with_driver(Box::new(peer_driver));
        let got = Arc::new(Mutex::new(None));
        let slot = got.clone();
        local
            .watch("stream/k", move |sample| {
                *slot.lock().unwrap() = Some(sample);
            })
            .unwrap();
        peer.send("stream/k", b"hello", 9).unwrap();
        drop(peer);
        local.run().unwrap();
        let sample = got.lock().unwrap().clone().expect("sample");
        assert_eq!(sample.key, "stream/k");
        assert_eq!(sample.payload, b"hello");
        assert_eq!(sample.correlation, 9);
    }

    #[test]
    fn channel_pair_rpc_without_zenoh() {
        let (local_driver, peer_driver) = ChannelDriver::pair();
        let mut local = Session::with_driver(Box::new(local_driver));
        let peer = Session::with_driver(Box::new(peer_driver));
        local
            .on_rpc("rpc/echo", |payload| Ok(payload.to_vec()))
            .unwrap();
        let client = peer.rpc_client("rpc/echo");
        let join = std::thread::spawn(move || local.run());
        let reply = client.call(b"ping").unwrap();
        assert_eq!(reply, b"ping");
        drop(client);
        drop(peer);
        join.join().expect("run thread").unwrap();
    }

    #[test]
    fn channel_pair_put_client_during_run() {
        let (local_driver, peer_driver) = ChannelDriver::pair();
        let mut local = Session::with_driver(Box::new(local_driver));
        let peer = Session::with_driver(Box::new(peer_driver));
        let got = Arc::new(Mutex::new(None));
        let slot = got.clone();
        local
            .watch("stream/k", move |sample| {
                *slot.lock().unwrap() = Some(sample);
            })
            .unwrap();
        let put_client = peer.put_client();
        let join = std::thread::spawn(move || local.run());
        put_client.send("stream/k", b"hello", 9).unwrap();
        drop(put_client);
        drop(peer);
        join.join().expect("run thread").unwrap();
        let sample = got.lock().unwrap().clone().expect("sample");
        assert_eq!(sample.payload, b"hello");
        assert_eq!(sample.correlation, 9);
    }
}
