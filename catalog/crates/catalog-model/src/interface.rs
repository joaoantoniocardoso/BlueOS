use schemars::JsonSchema;
use serde::Serialize;

use catalog_kernel::id::refs::{PathRef, PortRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "bus", rename_all = "snake_case")]
pub enum PortKind {
    Rest {
        path_prefix: PathRef,
        port: PortRef,
        versions: &'static [&'static str],
    },
    Zenoh {
        topics_produced: &'static [&'static str],
        topics_consumed: &'static [&'static str],
    },
    Mavlink {
        role: MavlinkRole,
        connect: &'static str,
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
        url: &'static str,
    },
    Subprocess {
        command: &'static str,
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
        image: &'static str,
    },
}

// Directional roles derivable from the connect string (in = Endpoint, out = Consumer).
// Router ownership is a judgment, not observable here; it lives in the asserted layer as
// Authority::MavlinkRouterOwner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MavlinkRole {
    Endpoint,
    Bridge,
    Consumer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileAccessMode {
    Read,
    Write,
    ReadWrite,
}
