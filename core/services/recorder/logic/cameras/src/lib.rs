//! MAVLink camera capture commands, status replies and video stream registration.

#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, collections::BTreeSet, string::String, vec, vec::Vec};
use core::time::Duration;

use blueos_domain::{Effect, Now, Outcome};
use blueos_recorder_capture::Capture;

/// Topic the camera manager publishes raw MAVLink on.
pub const RAW_MAVLINK_OUT_TOPIC: &str = "mavlink_raw/out";
/// Topic the recorder publishes MAVLink replies on.
pub const RAW_MAVLINK_IN_TOPIC: &str = "mavlink_raw/in";

/// Minimum spacing between periodic capture status replies.
const MIN_CAPTURE_STATUS_INTERVAL: Duration = Duration::from_millis(100);
/// Maximum spacing when the camera manager requests a very low rate.
const MAX_CAPTURE_STATUS_INTERVAL: Duration = Duration::from_secs(1);
/// Default spacing when the command omits a usable rate.
const DEFAULT_CAPTURE_STATUS_INTERVAL: Duration = Duration::from_secs(1);

/// MAVLink `video_status` when the stream is not recording.
pub const VIDEO_CAPTURE_STATUS_IDLE: u8 = 0;
/// MAVLink `video_status` when the stream is recording.
pub const VIDEO_CAPTURE_STATUS_RECORDING: u8 = 1;

/// Block state for MAVLink camera protocol handling.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cameras {
    video_streams: BTreeMap<String, VideoStream>,
    cameras_with_capture_video: BTreeSet<SystemAndComponent>,
    discovered_cameras: BTreeSet<SystemAndComponent>,
}

/// Facts the MAVLink ingress Task reports from raw frames.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CamerasObservedFact {
    /// A camera component sent its first heartbeat.
    CameraHeartbeat {
        /// Camera ids.
        camera: SystemAndComponent,
    },
    /// A camera stream was registered from `VIDEO_STREAM_INFORMATION`.
    RegisterVideoStream {
        /// `video/...` topic written to the backbone.
        topic: String,
        /// Source camera ids.
        camera: SystemAndComponent,
    },
    /// Whether the camera supports video capture.
    SetCameraRecordingCapability {
        /// Camera ids.
        camera: SystemAndComponent,
        /// When false, start capture is rejected.
        capture_video: bool,
    },
    /// A `COMMAND_LONG` capture command from the camera manager.
    CameraCaptureCommand {
        /// Which capture command was sent.
        command: CaptureCommandKind,
        /// System and component that sent the command, which the ack is addressed to.
        sender: SystemAndComponent,
        /// MAVLink target system.
        target_system: u8,
        /// MAVLink target component.
        target_component: u8,
        /// Minimum spacing between periodic capture status replies.
        status_interval: Duration,
    },
}

/// Timer keys; one capture status timer per video topic.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CamerasTimerKey {
    /// Periodic capture status for one stream.
    CaptureStatus {
        /// `video/...` topic.
        topic: String,
    },
}

/// Ticks scheduled by this Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CamerasTick {
    /// Send [`CamerasIoRequest::CaptureStatus`] and re-arm this stream's timer.
    CaptureStatus {
        /// `video/...` topic.
        topic: String,
    },
}

/// IO this Block performs through the MAVLink egress topic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CamerasIoRequest {
    /// Publish a `COMMAND_ACK` frame.
    CommandAck {
        /// Camera that sends the ack.
        camera: SystemAndComponent,
        /// Sender of the command, which the ack is addressed to.
        recipient: SystemAndComponent,
        /// MAVLink command being acknowledged.
        command: CaptureCommandKind,
        /// Whether the command was accepted.
        accepted: bool,
    },
    /// Publish a `CAMERA_CAPTURE_STATUS` frame.
    CaptureStatus {
        /// Camera that sends the status.
        camera: SystemAndComponent,
        /// MAVLink `video_status` field.
        video_status: u8,
        /// Elapsed recording time in milliseconds.
        recording_time_ms: u32,
    },
    /// Publish a `COMMAND_LONG` discovery request to a camera.
    RequestDiscovery {
        /// Camera that should receive the request.
        camera: SystemAndComponent,
        /// Which information message to request.
        message: DiscoveryMessageKind,
    },
}

/// Result of cameras IO executed by the Kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CamerasIoResult {
    /// Publishing a MAVLink reply to the backbone failed.
    PublishFailed,
}

/// Which capture command the camera manager sent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureCommandKind {
    /// Start video capture for the addressed camera stream.
    StartCapture,
    /// Stop video capture for the addressed camera stream.
    StopCapture,
    /// One-shot capture status reply.
    RequestCaptureStatus,
}

/// MAVLink discovery messages sent after a camera heartbeat.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryMessageKind {
    /// Request `CAMERA_INFORMATION`.
    CameraInformation,
    /// Request `VIDEO_STREAM_INFORMATION`.
    VideoStreamInformation,
}

/// Per-stream state for capture status timers and recording time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoStream {
    /// Camera that owns this stream.
    pub camera: SystemAndComponent,
    /// Whether MAVLink capture is active on this stream.
    pub is_recording: bool,
    /// Minimum spacing between periodic capture status replies.
    pub status_interval: Duration,
    /// Monotonic time when recording started, for [`Capture::recording_time_ms`].
    pub recording_started_at: Option<Duration>,
}

/// A MAVLink system and component id pair.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SystemAndComponent {
    /// MAVLink system id.
    pub system_id: u8,
    /// MAVLink component id.
    pub component_id: u8,
}

type CamerasOutcome =
    Outcome<core::convert::Infallible, CamerasTick, CamerasIoRequest, CamerasTimerKey>;

impl Cameras {
    /// Converts a requested status rate into a bounded interval.
    pub fn capture_status_interval_from_rate(rate_hertz: f32) -> Duration {
        if !rate_hertz.is_finite() || rate_hertz <= 0.0 {
            return DEFAULT_CAPTURE_STATUS_INTERVAL;
        }
        let bounded_rate = rate_hertz.clamp(
            1.0 / MAX_CAPTURE_STATUS_INTERVAL.as_secs_f32(),
            1.0 / MIN_CAPTURE_STATUS_INTERVAL.as_secs_f32(),
        );
        Duration::from_secs_f32(1.0 / bounded_rate)
            .clamp(MIN_CAPTURE_STATUS_INTERVAL, MAX_CAPTURE_STATUS_INTERVAL)
    }

    /// Applies an Observed fact from the MAVLink ingress Task.
    pub fn handle_observed_fact(&mut self, fact: CamerasObservedFact, now: Now) -> CamerasOutcome {
        match fact {
            CamerasObservedFact::CameraHeartbeat { camera } => self.camera_heartbeat(camera),
            CamerasObservedFact::RegisterVideoStream { topic, camera } => {
                self.register_video_stream(topic, camera)
            }
            CamerasObservedFact::SetCameraRecordingCapability {
                camera,
                capture_video,
            } => self.set_camera_recording_capability(camera, capture_video),
            CamerasObservedFact::CameraCaptureCommand {
                command,
                sender,
                target_system,
                target_component,
                status_interval,
            } => self.handle_capture_command(
                command,
                sender,
                target_system,
                target_component,
                status_interval,
                now,
            ),
        }
    }

    /// Applies a timer Tick.
    pub fn handle_tick(&mut self, tick: CamerasTick, now: Now) -> CamerasOutcome {
        match tick {
            CamerasTick::CaptureStatus { topic } => self.capture_status_tick(topic, now),
        }
    }

    /// Video topics and whether each is actively recording (for syncing the capture Block).
    pub fn video_recording_by_topic(&self) -> impl Iterator<Item = (&str, bool)> {
        self.video_streams
            .iter()
            .map(|(topic, stream)| (topic.as_str(), stream.is_recording))
    }

    fn camera_heartbeat(&mut self, camera: SystemAndComponent) -> CamerasOutcome {
        if !self.discovered_cameras.insert(camera) {
            return Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            };
        }
        Outcome::Applied {
            events: Vec::new(),
            effects: vec![
                Effect::Io(CamerasIoRequest::RequestDiscovery {
                    camera,
                    message: DiscoveryMessageKind::CameraInformation,
                }),
                Effect::Io(CamerasIoRequest::RequestDiscovery {
                    camera,
                    message: DiscoveryMessageKind::VideoStreamInformation,
                }),
            ],
        }
    }

    fn set_camera_recording_capability(
        &mut self,
        camera: SystemAndComponent,
        capture_video: bool,
    ) -> CamerasOutcome {
        if capture_video {
            self.cameras_with_capture_video.insert(camera);
        } else {
            self.cameras_with_capture_video.remove(&camera);
        }
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn register_video_stream(
        &mut self,
        topic: String,
        camera: SystemAndComponent,
    ) -> CamerasOutcome {
        self.video_streams.insert(
            topic,
            VideoStream {
                camera,
                is_recording: false,
                status_interval: DEFAULT_CAPTURE_STATUS_INTERVAL,
                recording_started_at: None,
            },
        );
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn handle_capture_command(
        &mut self,
        command: CaptureCommandKind,
        sender: SystemAndComponent,
        target_system: u8,
        target_component: u8,
        status_interval: Duration,
        now: Now,
    ) -> CamerasOutcome {
        let interval =
            status_interval.clamp(MIN_CAPTURE_STATUS_INTERVAL, MAX_CAPTURE_STATUS_INTERVAL);
        let mut effects = Vec::new();

        for (topic, stream) in self.video_streams.iter_mut() {
            if !is_addressed_to(
                target_system,
                target_component,
                stream.camera.system_id,
                stream.camera.component_id,
            ) {
                continue;
            }
            let camera = stream.camera;
            match command {
                CaptureCommandKind::StartCapture => {
                    let accepted = self.cameras_with_capture_video.contains(&camera);
                    effects.push(Effect::Io(CamerasIoRequest::CommandAck {
                        camera,
                        recipient: sender,
                        command,
                        accepted,
                    }));
                    if !accepted {
                        continue;
                    }
                    stream.is_recording = true;
                    stream.recording_started_at = Some(now.monotonic);
                    stream.status_interval = interval;
                    effects.push(Effect::Cancel(CamerasTimerKey::CaptureStatus {
                        topic: topic.clone(),
                    }));
                    effects.push(Effect::Schedule {
                        after: Duration::ZERO,
                        key: CamerasTimerKey::CaptureStatus {
                            topic: topic.clone(),
                        },
                        command: CamerasTick::CaptureStatus {
                            topic: topic.clone(),
                        },
                    });
                }
                CaptureCommandKind::StopCapture => {
                    stream.is_recording = false;
                    stream.recording_started_at = None;
                    effects.push(Effect::Io(CamerasIoRequest::CommandAck {
                        camera,
                        recipient: sender,
                        command,
                        accepted: true,
                    }));
                    effects.push(Effect::Io(CamerasIoRequest::CaptureStatus {
                        camera,
                        video_status: VIDEO_CAPTURE_STATUS_IDLE,
                        recording_time_ms: 0,
                    }));
                    effects.push(Effect::Cancel(CamerasTimerKey::CaptureStatus {
                        topic: topic.clone(),
                    }));
                }
                CaptureCommandKind::RequestCaptureStatus => {
                    let (video_status, recording_time_ms) = capture_status_fields(stream, now);
                    effects.push(Effect::Io(CamerasIoRequest::CommandAck {
                        camera,
                        recipient: sender,
                        command,
                        accepted: true,
                    }));
                    effects.push(Effect::Io(CamerasIoRequest::CaptureStatus {
                        camera,
                        video_status,
                        recording_time_ms,
                    }));
                }
            }
        }

        Outcome::Applied {
            events: Vec::new(),
            effects,
        }
    }

    fn capture_status_tick(&mut self, topic: String, now: Now) -> CamerasOutcome {
        let Some(stream) = self.video_streams.get(&topic) else {
            return Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            };
        };
        if !stream.is_recording {
            return Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            };
        }
        let camera = stream.camera;
        let interval = stream.status_interval;
        let recording_time_ms = stream
            .recording_started_at
            .map(|started_at| Capture::recording_time_ms(started_at, now))
            .unwrap_or(0);
        Outcome::Applied {
            events: Vec::new(),
            effects: vec![
                Effect::Io(CamerasIoRequest::CaptureStatus {
                    camera,
                    video_status: VIDEO_CAPTURE_STATUS_RECORDING,
                    recording_time_ms,
                }),
                Effect::Schedule {
                    after: interval,
                    key: CamerasTimerKey::CaptureStatus {
                        topic: topic.clone(),
                    },
                    command: CamerasTick::CaptureStatus { topic },
                },
            ],
        }
    }
}

fn capture_status_fields(stream: &VideoStream, now: Now) -> (u8, u32) {
    if stream.is_recording {
        let recording_time_ms = stream
            .recording_started_at
            .map(|started_at| Capture::recording_time_ms(started_at, now))
            .unwrap_or(0);
        (VIDEO_CAPTURE_STATUS_RECORDING, recording_time_ms)
    } else {
        (VIDEO_CAPTURE_STATUS_IDLE, 0)
    }
}

fn is_addressed_to(
    target_system: u8,
    target_component: u8,
    camera_system: u8,
    camera_component: u8,
) -> bool {
    let system_matches = target_system == 0 || target_system == camera_system;
    let component_matches = target_component == 0 || target_component == camera_component;
    system_matches && component_matches
}
