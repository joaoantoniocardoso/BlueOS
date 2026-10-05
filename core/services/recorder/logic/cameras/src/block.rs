//! [`Cameras`] Block orchestration and capture command handling.

use alloc::{collections::BTreeMap, collections::BTreeSet, string::String, vec, vec::Vec};
use core::time::Duration;

use blueos_domain::{Effect, Now, Outcome};
use blueos_recorder_capture::Capture;

use crate::{
    capture_effects::{
        is_addressed_to, push_request_capture_status_effects, push_start_capture_effects,
        push_stop_capture_effects,
    },
    types::{
        CamerasIoRequest, CamerasObservedFact, CamerasOutcome, CamerasTick, CamerasTimerKey,
        CaptureCommandInput, CaptureCommandKind, DEFAULT_CAPTURE_STATUS_INTERVAL,
        DiscoveryMessageKind, MAX_CAPTURE_STATUS_INTERVAL, MIN_CAPTURE_STATUS_INTERVAL,
        StartCaptureDispatch, SystemAndComponent, VIDEO_CAPTURE_STATUS_RECORDING, VideoStream,
    },
};

/// Block state for MAVLink camera protocol handling.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cameras {
    video_streams: BTreeMap<String, VideoStream>,
    cameras_with_capture_video: BTreeSet<SystemAndComponent>,
    discovered_cameras: BTreeSet<SystemAndComponent>,
}

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
                CaptureCommandInput {
                    command,
                    sender,
                    target_system,
                    target_component,
                    status_interval,
                },
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

    fn handle_capture_command(&mut self, input: CaptureCommandInput, now: Now) -> CamerasOutcome {
        let interval = input
            .status_interval
            .clamp(MIN_CAPTURE_STATUS_INTERVAL, MAX_CAPTURE_STATUS_INTERVAL);
        let mut effects = Vec::new();
        let topics = self
            .video_streams
            .keys()
            .filter(|topic| {
                let stream = &self.video_streams[*topic];
                is_addressed_to(
                    input.target_system,
                    input.target_component,
                    stream.camera.system_id,
                    stream.camera.component_id,
                )
            })
            .cloned()
            .collect::<Vec<_>>();

        for topic in topics {
            let Some(stream) = self.video_streams.get_mut(&topic) else {
                continue;
            };
            let capture_video_enabled = self.cameras_with_capture_video.contains(&stream.camera);
            match input.command {
                CaptureCommandKind::StartCapture => push_start_capture_effects(
                    stream,
                    topic,
                    &StartCaptureDispatch {
                        input: &input,
                        interval,
                        now,
                        accepted: capture_video_enabled,
                    },
                    &mut effects,
                ),
                CaptureCommandKind::StopCapture => {
                    push_stop_capture_effects(stream, topic, &input, &mut effects);
                }
                CaptureCommandKind::RequestCaptureStatus => {
                    push_request_capture_status_effects(stream, &input, now, &mut effects);
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
        let timer_key = CamerasTimerKey::CaptureStatus {
            topic: topic.clone(),
        };
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
                    key: timer_key,
                    command: CamerasTick::CaptureStatus { topic },
                },
            ],
        }
    }
}
