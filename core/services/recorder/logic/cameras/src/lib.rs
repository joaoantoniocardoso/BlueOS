//! MAVLink camera capture commands, status replies and video stream registration.

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "types and block split for module size; public API stays on the crate root"
)]

extern crate alloc;

mod block;
mod capture_effects;
mod types;

pub use block::Cameras;
pub use types::{
    CamerasIoRequest, CamerasIoResult, CamerasObservedFact, CamerasTick, CamerasTimerKey,
    CaptureCommandKind, DiscoveryMessageKind, RAW_MAVLINK_IN_TOPIC, RAW_MAVLINK_OUT_TOPIC,
    SystemAndComponent, VIDEO_CAPTURE_STATUS_IDLE, VIDEO_CAPTURE_STATUS_RECORDING, VideoStream,
};
