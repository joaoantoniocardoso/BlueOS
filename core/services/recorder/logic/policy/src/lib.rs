#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use blueos_cqrs::{Decision, Domain, Effect, TimerId};
use blueos_jobs::Jobs;
pub use blueos_recorder_library::{
    LibraryCommand, LibraryEvent, LibraryIo, LibrarySnapshot,
    OperationKind as RecordingOperationKind, RecordingEntry, RecordingOperationEvent,
    RecordingState as LibraryRecordingState, ScannedRecording, handle_library_command,
    initial_rescan_effects,
};

pub const RAW_MAVLINK_OUT_TOPIC: &str = "mavlink_raw/out";
pub const RAW_MAVLINK_IN_TOPIC: &str = "mavlink_raw/in";
pub const RAW_MAVLINK_OUT_TOPIC_PREFIX: &str = RAW_MAVLINK_OUT_TOPIC;
pub const MAVLINK_TOPIC_PREFIX: &str = "mavlink/";
pub const MAVLINK_RAW_TOPIC_PREFIX: &str = "mavlink_raw/";
pub const VIDEO_TOPIC_PREFIX: &str = "video/";

pub const MAV_CMD_VIDEO_START_CAPTURE: u32 = 2500;
pub const MAV_CMD_VIDEO_STOP_CAPTURE: u32 = 2501;
pub const MAV_CMD_REQUEST_CAMERA_CAPTURE_STATUS: u32 = 522;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct SystemAndComponent {
    pub system_id: u8,
    pub component_id: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingPolicy {
    pub record_mavlink_only_when_armed: bool,
    pub auto_start_recording: bool,
}

impl Default for RecordingPolicy {
    fn default() -> Self {
        Self {
            record_mavlink_only_when_armed: true,
            auto_start_recording: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoStreamState {
    pub topic: String,
    pub camera: SystemAndComponent,
    pub is_recording: bool,
    pub status_interval_millis: u64,
    pub recording_started_millis: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingSession {
    pub file_name: String,
    pub bytes_written: u64,
}

#[derive(Clone, Default)]
pub struct RecorderSnapshot {
    pub policy: RecordingPolicy,
    pub armed: bool,
    pub session_active: bool,
    pub session: Option<RecordingSession>,
    pub video_streams: BTreeMap<String, VideoStreamState>,
    pub recording_cameras: BTreeSet<SystemAndComponent>,
    pub library: LibrarySnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecorderCommand {
    SetPolicy(RecordingPolicy),
    StartRecording {
        rotate_if_active: bool,
    },
    StopRecording,
    ArmedChanged(bool),
    RegisterVideoStream {
        topic: String,
        camera: SystemAndComponent,
    },
    SetCameraRecordingCapability {
        camera: SystemAndComponent,
        capture_video: bool,
    },
    CameraCaptureCommand {
        command: CaptureCommandKind,
        target_system: u8,
        target_component: u8,
        status_interval_hertz: f32,
        now_millis: u64,
    },
    SessionOpened {
        file_name: String,
    },
    SessionFinished,
    SessionBytesWritten(u64),
    Ack,
    IoFailed,
    CaptureStatusTick {
        topic: String,
        now_millis: u64,
    },
    InitializeLibrary,
    RepairRecording {
        path: String,
        now_unix_seconds: i64,
    },
    CancelRepair {
        path: String,
    },
    DeleteRecording {
        path: String,
    },
    SnapshotRecording {
        path: String,
        now_unix_seconds: i64,
    },
    Library(LibraryCommand),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureCommandKind {
    StartCapture,
    StopCapture,
    RequestCaptureStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecorderEvent {
    SessionRotated { file_name: String },
    SessionStopped,
    RecordingOperation(RecordingOperationEvent),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TapPolicy {
    pub session_active: bool,
    pub armed: bool,
    pub record_mavlink_only_when_armed: bool,
    pub recording_video_topics: BTreeSet<String>,
}

impl TapPolicy {
    pub fn from_snapshot(snapshot: &RecorderSnapshot) -> Self {
        let recording_video_topics = snapshot
            .video_streams
            .iter()
            .filter(|(_, stream)| stream.is_recording)
            .map(|(topic, _)| topic.clone())
            .collect();
        Self {
            session_active: snapshot.session_active,
            armed: snapshot.armed,
            record_mavlink_only_when_armed: snapshot.policy.record_mavlink_only_when_armed,
            recording_video_topics,
        }
    }

    pub fn should_record_topic(&self, topic: &str) -> bool {
        if !self.session_active {
            return false;
        }
        if (topic.starts_with(MAVLINK_TOPIC_PREFIX) || topic.starts_with(MAVLINK_RAW_TOPIC_PREFIX))
            && self.record_mavlink_only_when_armed
        {
            return self.armed;
        }
        if topic.starts_with(VIDEO_TOPIC_PREFIX) {
            return self.recording_video_topics.contains(topic);
        }
        true
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MavlinkReply {
    pub frames: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecorderIo {
    OpenSession,
    FinishSession,
    PublishMavlink(Vec<u8>),
    MavlinkCommandAck {
        camera: SystemAndComponent,
        command: u32,
        accepted: bool,
    },
    MavlinkCaptureStatus {
        camera: SystemAndComponent,
        video_status: u8,
        recording_time_ms: u32,
    },
    Library(LibraryIo),
}

pub enum RecorderQuery {
    TapPolicy,
}

#[derive(Clone, PartialEq, Eq)]
pub struct TapPolicyView {
    pub policy: TapPolicy,
}

#[derive(Clone)]
pub enum RecorderJobSpec {}

pub struct RecorderDomain;

const CAPTURE_STATUS_TIMER: TimerId = TimerId(1);

impl Domain for RecorderDomain {
    type Command = RecorderCommand;
    type Event = RecorderEvent;
    type Query = RecorderQuery;
    type View = TapPolicyView;
    type Snapshot = RecorderSnapshot;
    type IoRequest = RecorderIo;
    type JobSpec = RecorderJobSpec;

    fn handle_command(
        snapshot: &mut Self::Snapshot,
        _jobs: &mut Jobs<Self::JobSpec>,
        command: Self::Command,
    ) -> Decision<Self> {
        match command {
            RecorderCommand::SetPolicy(policy) => {
                snapshot.policy = policy;
                Decision::new()
            }
            RecorderCommand::StartRecording { rotate_if_active } => {
                if snapshot.session_active && !rotate_if_active {
                    return Decision::new();
                }
                let mut effects = Vec::new();
                if snapshot.session_active {
                    effects.push(Effect::Io(RecorderIo::FinishSession));
                }
                effects.push(Effect::Io(RecorderIo::OpenSession));
                Decision {
                    events: Vec::new(),
                    effects,
                    rejection: None,
                }
            }
            RecorderCommand::StopRecording => {
                if !snapshot.session_active {
                    return Decision::new();
                }
                snapshot.session_active = false;
                snapshot.session = None;
                for stream in snapshot.video_streams.values_mut() {
                    stream.is_recording = false;
                }
                Decision {
                    events: vec![RecorderEvent::SessionStopped],
                    effects: vec![Effect::Io(RecorderIo::FinishSession)],
                    rejection: None,
                }
            }
            RecorderCommand::ArmedChanged(armed) => {
                snapshot.armed = armed;
                Decision::new()
            }
            RecorderCommand::RegisterVideoStream { topic, camera } => {
                if snapshot.video_streams.contains_key(&topic) {
                    return Decision::new();
                }
                snapshot.video_streams.insert(
                    topic.clone(),
                    VideoStreamState {
                        topic,
                        camera,
                        is_recording: false,
                        status_interval_millis: 1_000,
                        recording_started_millis: None,
                    },
                );
                Decision::new()
            }
            RecorderCommand::SetCameraRecordingCapability {
                camera,
                capture_video,
            } => {
                if capture_video {
                    snapshot.recording_cameras.insert(camera);
                } else {
                    snapshot.recording_cameras.remove(&camera);
                }
                Decision::new()
            }
            RecorderCommand::CameraCaptureCommand {
                command,
                target_system,
                target_component,
                status_interval_hertz,
                now_millis,
            } => handle_camera_capture_command(
                snapshot,
                command,
                target_system,
                target_component,
                status_interval_hertz,
                now_millis,
            ),
            RecorderCommand::SessionOpened { file_name } => {
                snapshot.session_active = true;
                snapshot.session = Some(RecordingSession {
                    file_name,
                    bytes_written: 0,
                });
                Decision::new()
            }
            RecorderCommand::SessionFinished => {
                snapshot.session = None;
                Decision::new()
            }
            RecorderCommand::SessionBytesWritten(bytes) => {
                if let Some(session) = snapshot.session.as_mut() {
                    session.bytes_written = bytes;
                }
                Decision::new()
            }
            RecorderCommand::Ack => Decision::new(),
            RecorderCommand::IoFailed => Decision::new(),
            RecorderCommand::InitializeLibrary => Decision {
                events: Vec::new(),
                effects: map_library_effects(initial_rescan_effects()),
                rejection: None,
            },
            RecorderCommand::RepairRecording {
                path,
                now_unix_seconds,
            } => dispatch_library(
                snapshot,
                LibraryCommand::RepairRecording {
                    path,
                    now_unix_seconds,
                },
            ),
            RecorderCommand::CancelRepair { path } => {
                dispatch_library(snapshot, LibraryCommand::CancelRepair { path })
            }
            RecorderCommand::DeleteRecording { path } => {
                dispatch_library(snapshot, LibraryCommand::DeleteRecording { path })
            }
            RecorderCommand::SnapshotRecording {
                path,
                now_unix_seconds,
            } => dispatch_library(
                snapshot,
                LibraryCommand::SnapshotRecording {
                    path,
                    now_unix_seconds,
                },
            ),
            RecorderCommand::Library(command) => dispatch_library(snapshot, command),
            RecorderCommand::CaptureStatusTick { topic, now_millis } => {
                let Some(stream) = snapshot.video_streams.get(&topic) else {
                    return Decision::new();
                };
                if !stream.is_recording {
                    return Decision::new();
                }
                let recording_time_ms = stream
                    .recording_started_millis
                    .map(|start| now_millis.saturating_sub(start) as u32)
                    .unwrap_or(0);
                let camera = stream.camera;
                let interval = stream.status_interval_millis;
                Decision {
                    events: Vec::new(),
                    effects: vec![
                        Effect::Io(RecorderIo::MavlinkCaptureStatus {
                            camera,
                            video_status: 1,
                            recording_time_ms,
                        }),
                        Effect::Schedule {
                            after: core::time::Duration::from_millis(interval),
                            timer: CAPTURE_STATUS_TIMER,
                            command: RecorderCommand::CaptureStatusTick { topic, now_millis },
                        },
                    ],
                    rejection: None,
                }
            }
        }
    }

    fn handle_query(
        snapshot: &Self::Snapshot,
        _jobs: &Jobs<Self::JobSpec>,
        _query: Self::Query,
    ) -> Self::View {
        TapPolicyView {
            policy: TapPolicy::from_snapshot(snapshot),
        }
    }

    fn io_from_job(_job_id: blueos_jobs::JobId, _job_spec: &Self::JobSpec) -> Self::IoRequest {
        RecorderIo::OpenSession
    }
}

fn handle_camera_capture_command(
    snapshot: &mut RecorderSnapshot,
    command: CaptureCommandKind,
    target_system: u8,
    target_component: u8,
    status_interval_hertz: f32,
    now_millis: u64,
) -> Decision<RecorderDomain> {
    let mut effects = Vec::new();
    let events = Vec::new();
    let interval_millis = (1_000.0 / status_interval_hertz.clamp(1.0, 10.0)) as u64;
    let command_id = capture_command_id(command);

    for stream in snapshot.video_streams.values_mut() {
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
                stream.is_recording = true;
                stream.recording_started_millis = Some(now_millis);
                stream.status_interval_millis = interval_millis.max(100);
                effects.push(Effect::Io(RecorderIo::MavlinkCommandAck {
                    camera,
                    command: command_id,
                    accepted: true,
                }));
                effects.push(Effect::CancelSchedule(CAPTURE_STATUS_TIMER));
                effects.push(Effect::Schedule {
                    after: core::time::Duration::from_millis(0),
                    timer: CAPTURE_STATUS_TIMER,
                    command: RecorderCommand::CaptureStatusTick {
                        topic: stream.topic.clone(),
                        now_millis,
                    },
                });
            }
            CaptureCommandKind::StopCapture => {
                stream.is_recording = false;
                stream.recording_started_millis = None;
                effects.push(Effect::Io(RecorderIo::MavlinkCommandAck {
                    camera,
                    command: command_id,
                    accepted: true,
                }));
                effects.push(Effect::CancelSchedule(CAPTURE_STATUS_TIMER));
            }
            CaptureCommandKind::RequestCaptureStatus => {
                let (video_status, recording_time_ms) = if stream.is_recording {
                    (
                        1u8,
                        stream
                            .recording_started_millis
                            .map(|start| now_millis.saturating_sub(start) as u32)
                            .unwrap_or(0),
                    )
                } else {
                    (0u8, 0u32)
                };
                effects.push(Effect::Io(RecorderIo::MavlinkCommandAck {
                    camera,
                    command: command_id,
                    accepted: true,
                }));
                effects.push(Effect::Io(RecorderIo::MavlinkCaptureStatus {
                    camera,
                    video_status,
                    recording_time_ms,
                }));
            }
        }
    }

    Decision {
        events,
        effects,
        rejection: None,
    }
}

fn dispatch_library(
    snapshot: &mut RecorderSnapshot,
    command: LibraryCommand,
) -> Decision<RecorderDomain> {
    let active = snapshot
        .session
        .as_ref()
        .map(|session| session.file_name.clone());
    let active = active.as_deref();
    let decision = handle_library_command(&mut snapshot.library, command, active);
    map_library_decision(decision)
}

fn map_library_decision(
    decision: Decision<blueos_recorder_library::LibraryDomain>,
) -> Decision<RecorderDomain> {
    if let Some(reason) = decision.rejection {
        return Decision::reject(reason);
    }
    Decision {
        events: decision.events.into_iter().map(map_library_event).collect(),
        effects: map_library_effects(decision.effects),
        rejection: None,
    }
}

fn map_library_event(event: LibraryEvent) -> RecorderEvent {
    match event {
        LibraryEvent::Operation(operation) => RecorderEvent::RecordingOperation(operation),
    }
}

fn map_library_effects(
    effects: Vec<Effect<LibraryCommand, LibraryIo>>,
) -> Vec<Effect<RecorderCommand, RecorderIo>> {
    effects
        .into_iter()
        .map(|effect| match effect {
            Effect::Io(request) => Effect::Io(RecorderIo::Library(request)),
            Effect::Schedule {
                after,
                timer,
                command,
            } => Effect::Schedule {
                after,
                timer,
                command: RecorderCommand::Library(command),
            },
            Effect::CancelSchedule(timer) => Effect::CancelSchedule(timer),
            Effect::Persist => Effect::Persist,
        })
        .collect()
}

fn capture_command_id(command: CaptureCommandKind) -> u32 {
    match command {
        CaptureCommandKind::StartCapture => MAV_CMD_VIDEO_START_CAPTURE,
        CaptureCommandKind::StopCapture => MAV_CMD_VIDEO_STOP_CAPTURE,
        CaptureCommandKind::RequestCaptureStatus => MAV_CMD_REQUEST_CAMERA_CAPTURE_STATUS,
    }
}

pub fn is_addressed_to(
    target_system: u8,
    target_component: u8,
    our_system_id: u8,
    our_component_id: u8,
) -> bool {
    let system_matches = target_system == 0 || target_system == our_system_id;
    let component_matches = target_component == 0 || target_component == our_component_id;
    system_matches && component_matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_snapshot() -> RecorderSnapshot {
        RecorderSnapshot::default()
    }

    #[test]
    fn mavlink_gated_when_disarmed() {
        let snapshot = RecorderSnapshot {
            armed: false,
            session_active: true,
            ..empty_snapshot()
        };
        let policy = TapPolicy::from_snapshot(&snapshot);
        assert!(!policy.should_record_topic("mavlink/vehicle/heartbeat"));
        assert!(!policy.should_record_topic("mavlink_raw/in"));
        assert!(policy.should_record_topic("blueos/v1/ping/state"));
    }

    #[test]
    fn mavlink_recorded_when_armed() {
        let snapshot = RecorderSnapshot {
            armed: true,
            session_active: true,
            ..empty_snapshot()
        };
        let policy = TapPolicy::from_snapshot(&snapshot);
        assert!(policy.should_record_topic("mavlink/vehicle/heartbeat"));
    }

    #[test]
    fn video_gated_per_stream() {
        let mut snapshot = RecorderSnapshot {
            session_active: true,
            ..empty_snapshot()
        };
        snapshot.video_streams.insert(
            "video/front/stream".into(),
            VideoStreamState {
                topic: "video/front/stream".into(),
                camera: SystemAndComponent {
                    system_id: 1,
                    component_id: 100,
                },
                is_recording: true,
                status_interval_millis: 1_000,
                recording_started_millis: None,
            },
        );
        let policy = TapPolicy::from_snapshot(&snapshot);
        assert!(policy.should_record_topic("video/front/stream"));
        assert!(!policy.should_record_topic("video/rear/stream"));
    }

    #[test]
    fn stop_recording_closes_session_flag() {
        let mut snapshot = RecorderSnapshot {
            session_active: true,
            session: Some(RecordingSession {
                file_name: "recorder_20260101_120000.mcap".into(),
                bytes_written: 0,
            }),
            ..empty_snapshot()
        };
        let decision = RecorderDomain::handle_command(
            &mut snapshot,
            &mut Jobs::new(),
            RecorderCommand::StopRecording,
        );
        assert!(!snapshot.session_active);
        assert!(snapshot.session.is_none());
        assert_eq!(decision.effects.len(), 1);
    }

    #[test]
    fn camera_start_marks_stream_recording() {
        let mut snapshot = RecorderSnapshot {
            session_active: true,
            ..empty_snapshot()
        };
        snapshot.video_streams.insert(
            "video/front/stream".into(),
            VideoStreamState {
                topic: "video/front/stream".into(),
                camera: SystemAndComponent {
                    system_id: 1,
                    component_id: 100,
                },
                is_recording: false,
                status_interval_millis: 1_000,
                recording_started_millis: None,
            },
        );
        RecorderDomain::handle_command(
            &mut snapshot,
            &mut Jobs::new(),
            RecorderCommand::CameraCaptureCommand {
                command: CaptureCommandKind::StartCapture,
                target_system: 1,
                target_component: 100,
                status_interval_hertz: 2.0,
                now_millis: 1_000,
            },
        );
        assert!(snapshot.video_streams["video/front/stream"].is_recording);
    }
}
