use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::{PathRef, PortRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "bus", rename_all = "snake_case")]
pub enum Interface {
    Rest {
        path_prefix: PathRef,
        port: PortRef,
        versions: Vec<String>,
    },
    Zenoh {
        topics_produced: Vec<String>,
        topics_consumed: Vec<String>,
    },
    Mavlink {
        role: MavlinkRole,
        connect: String,
    },
    Websocket {
        path: PathRef,
        port: PortRef,
    },
    HttpStream {
        path: PathRef,
        port: PortRef,
    },
    OutboundHttp {
        url: String,
    },
    Subprocess {
        command: String,
    },
    File {
        path: PathRef,
        mode: FileAccessMode,
    },
    Settings {
        path: PathRef,
    },
    Hardware {
        device: PathRef,
    },
    Docker {
        image: String,
    },
}

// Directional roles derivable from the connect string (in = Endpoint, out = Consumer).
// Router ownership is a judgment, not observable here; it lives in the asserted layer as
// Authority::MavlinkRouterOwner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MavlinkRole {
    Endpoint,
    Bridge,
    Consumer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileAccessMode {
    Read,
    Write,
    ReadWrite,
}
