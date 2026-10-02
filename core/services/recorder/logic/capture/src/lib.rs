//! Active recording, the armed flag, bytes written and the record gate the data plane follows.

#![no_std]

extern crate alloc;

use alloc::{collections::BTreeMap, collections::BTreeSet, string::String, vec, vec::Vec};
use core::{error::Error, fmt, time::Duration};

use blueos_domain::{Now, Outcome};

/// Backbone prefix for decoded MAVLink samples.
pub const MAVLINK_TOPIC_PREFIX: &str = "mavlink/";
/// Backbone prefix for raw MAVLink frames (ingress and egress).
pub const MAVLINK_RAW_TOPIC_PREFIX: &str = "mavlink_raw/";
/// Backbone prefix for camera manager video samples.
pub const VIDEO_TOPIC_PREFIX: &str = "video/";

/// Block state: settings, armed fact, recording lifecycle, and video streams.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capture {
    /// User settings including armed gating and auto-start.
    pub settings: CaptureSettings,
    /// Latest armed fact from the vehicle.
    pub armed: bool,
    /// MCAP file lifecycle; see [`RecordingState`].
    pub recording: RecordingState,
    /// Next [`RecordGate::desired_file_generation`] assigned on start or rotation.
    next_file_generation: u64,
    video_streams: BTreeMap<String, VideoStream>,
}

/// Commands a client sends to the capture Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureRequest {
    /// Ask the data plane to open a file, or rotate when `rotate_if_active` and already active.
    StartRecording {
        /// When already recording, bump [`RecordGate::desired_file_generation`] so the Task rotates.
        rotate_if_active: bool,
    },
    /// Ask the data plane to finish the current file and stop recording.
    StopRecording,
    /// Changes capture settings; may start recording when [`CaptureSettings::auto_start_recording`] is enabled.
    UpdateSettings(CaptureSettings),
    /// Marks a video topic as recording for [`RecordGate::recording_video_topics`].
    StartVideoRecording {
        /// Zenoh topic prefix `video/...`.
        topic: String,
    },
    /// Removes a video topic from [`RecordGate::recording_video_topics`].
    StopVideoRecording {
        /// Video topic to stop.
        topic: String,
    },
}

/// Control-plane recording lifecycle; illegal combinations are unrepresentable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordingState {
    /// The user is not recording and the Task should close any open file.
    Idle,
    /// Recording was requested; waiting for the Task to report an open file.
    AwaitingMcapFile {
        /// Matches [`RecordGate::desired_file_generation`].
        file_generation: u64,
    },
    /// The Task reported an open file for this recording.
    Active(ActiveRecording),
}

/// Facts the data plane Task reports (full current value, handled idempotently).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureObservedFact {
    /// The Task opened the MCAP file for this generation.
    McapFileOpened {
        /// Matches [`RecordGate::desired_file_generation`] when first opened, or the rotation target.
        file_generation: u64,
        /// Base name of the opened file.
        file_name: String,
    },
    /// The Task finished writing this file generation.
    McapFileFinished {
        /// Generation that was closed.
        file_generation: u64,
    },
    /// Latest byte count for this file generation.
    RecordingBytesWritten {
        /// Generation whose file was written.
        file_generation: u64,
        /// Total bytes in the file so far.
        bytes: u64,
    },
    /// Full current armed state (re-sent periodically so a dropped fact heals).
    ArmedChanged(bool),
}

/// Domain events emitted by the capture Block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureEvent {
    /// The active file was replaced during rotation.
    RecordingRotated {
        /// Name of the new active file.
        file_name: String,
    },
    /// Recording was stopped.
    RecordingStopped,
}

/// Why a capture Command was rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureRejection {
    /// [`CaptureRequest::StopRecording`] arrived while [`RecordingState::Idle`].
    NotRecording,
}

/// Per-video-stream state used for the record gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoStream {
    /// Same string as the map key.
    pub topic: String,
    /// Whether this stream is included in [`RecordGate::recording_video_topics`].
    pub is_recording: bool,
}

/// Projection the data plane Task reconciles against (D-27). Pure function of the Snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordGate {
    /// Whether the control plane wants the Task to keep an MCAP file open.
    pub recording_requested: bool,
    /// Latest armed fact from the vehicle.
    pub armed: bool,
    /// Copy of [`CaptureSettings::record_mavlink_only_when_armed`].
    pub record_mavlink_only_when_armed: bool,
    /// File generation the Task should open or rotate to; bumps on each rotation request.
    pub desired_file_generation: u64,
    /// Video topics that are actively being captured.
    pub recording_video_topics: BTreeSet<String>,
}

/// The recording the Domain tracks after the data plane reports an open [`McapFile`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveRecording {
    /// Monotonic file identity the Task assigns; stale facts for other values are ignored.
    pub file_generation: u64,
    /// Base name of the file being written.
    pub file_name: String,
    /// Bytes reported by the data plane for this file.
    pub bytes_written: u64,
}

/// User settings that affect whether samples are written and when recording starts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureSettings {
    /// When set, MAVLink topics are recorded only while the vehicle is armed.
    pub record_mavlink_only_when_armed: bool,
    /// When set, recording starts as soon as settings allow; applied live on update.
    pub auto_start_recording: bool,
}

type CaptureOutcome = Outcome<
    CaptureEvent,
    core::convert::Infallible,
    core::convert::Infallible,
    core::convert::Infallible,
>;

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            record_mavlink_only_when_armed: true,
            auto_start_recording: true,
        }
    }
}

impl fmt::Display for CaptureRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRecording => formatter.write_str("not recording"),
        }
    }
}

impl Error for CaptureRejection {}

impl Default for Capture {
    fn default() -> Self {
        Self {
            settings: CaptureSettings::default(),
            armed: false,
            recording: RecordingState::Idle,
            next_file_generation: 0,
            video_streams: BTreeMap::new(),
        }
    }
}

impl Capture {
    /// Builds the projection the data plane reconciles against.
    pub fn record_gate(&self) -> RecordGate {
        let recording_requested = !matches!(self.recording, RecordingState::Idle);
        let desired_file_generation = match &self.recording {
            RecordingState::Idle => 0,
            RecordingState::AwaitingMcapFile { file_generation } => *file_generation,
            RecordingState::Active(active) => active.file_generation.max(self.next_file_generation),
        };
        let recording_video_topics = self
            .video_streams
            .iter()
            .filter(|(_, stream)| stream.is_recording)
            .map(|(topic, _)| topic.clone())
            .collect();
        RecordGate {
            recording_requested,
            armed: self.armed,
            record_mavlink_only_when_armed: self.settings.record_mavlink_only_when_armed,
            desired_file_generation,
            recording_video_topics,
        }
    }

    /// Whether a backbone sample should be written for the current gate (MAVLink and video policy).
    pub fn should_record_sample(key: &str, gate: &RecordGate) -> bool {
        if key.starts_with("blueos/v1/recorder/") || key.starts_with("blueos/v1/services/") {
            return false;
        }
        if (key.starts_with(MAVLINK_TOPIC_PREFIX) || key.starts_with(MAVLINK_RAW_TOPIC_PREFIX))
            && gate.record_mavlink_only_when_armed
            && !gate.armed
        {
            return false;
        }
        if key.starts_with(VIDEO_TOPIC_PREFIX) && !gate.recording_video_topics.contains(key) {
            return false;
        }
        true
    }

    /// Elapsed recording time in milliseconds from monotonic clock readings (used by capture status in #70).
    pub fn recording_time_ms(recording_started_at: Duration, now: Now) -> u32 {
        now.monotonic
            .saturating_sub(recording_started_at)
            .as_millis() as u32
    }

    /// Applies a client Request.
    pub fn handle_request(&mut self, request: CaptureRequest, _now: Now) -> CaptureOutcome {
        match request {
            CaptureRequest::StartRecording { rotate_if_active } => {
                self.start_recording(rotate_if_active)
            }
            CaptureRequest::StopRecording => self.stop_recording(),
            CaptureRequest::UpdateSettings(settings) => self.update_settings(settings),
            CaptureRequest::StartVideoRecording { topic } => self.start_video_recording(topic),
            CaptureRequest::StopVideoRecording { topic } => self.stop_video_recording(topic),
        }
    }

    /// Applies an Observed fact from the data plane Task.
    pub fn handle_observed_fact(&mut self, fact: CaptureObservedFact) -> CaptureOutcome {
        match fact {
            CaptureObservedFact::McapFileOpened {
                file_generation,
                file_name,
            } => self.mcap_file_opened(file_generation, file_name),
            CaptureObservedFact::McapFileFinished { file_generation } => {
                self.mcap_file_finished(file_generation)
            }
            CaptureObservedFact::RecordingBytesWritten {
                file_generation,
                bytes,
            } => self.recording_bytes_written(file_generation, bytes),
            CaptureObservedFact::ArmedChanged(armed) => {
                self.armed = armed;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
        }
    }

    fn start_recording(&mut self, rotate_if_active: bool) -> CaptureOutcome {
        match &self.recording {
            RecordingState::Active(active) if !rotate_if_active => {
                return Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                };
            }
            RecordingState::Active(active) => {
                let next_generation = active.file_generation.saturating_add(1);
                self.next_file_generation = next_generation;
                return Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                };
            }
            RecordingState::AwaitingMcapFile { .. } => {
                return Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                };
            }
            RecordingState::Idle => {}
        }

        self.next_file_generation += 1;
        let file_generation = self.next_file_generation;
        self.recording = RecordingState::AwaitingMcapFile { file_generation };
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn stop_recording(&mut self) -> CaptureOutcome {
        if matches!(self.recording, RecordingState::Idle) {
            return Outcome::reject(CaptureRejection::NotRecording);
        }
        self.recording = RecordingState::Idle;
        self.next_file_generation = 0;
        for stream in self.video_streams.values_mut() {
            stream.is_recording = false;
        }
        Outcome::Applied {
            events: vec![CaptureEvent::RecordingStopped],
            effects: Vec::new(),
        }
    }

    fn update_settings(&mut self, settings: CaptureSettings) -> CaptureOutcome {
        self.settings = settings;
        if self.settings.auto_start_recording && matches!(self.recording, RecordingState::Idle) {
            return self.start_recording(false);
        }
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn start_video_recording(&mut self, topic: String) -> CaptureOutcome {
        self.sync_video_topic_recording(&topic, true);
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn stop_video_recording(&mut self, topic: String) -> CaptureOutcome {
        self.sync_video_topic_recording(&topic, false);
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    /// Updates the record gate video set when the cameras Block starts or stops MAVLink capture.
    pub fn sync_video_topic_recording(&mut self, topic: &str, recording: bool) {
        if recording {
            let topic = String::from(topic);
            self.video_streams.insert(
                topic.clone(),
                VideoStream {
                    topic,
                    is_recording: true,
                },
            );
        } else if let Some(stream) = self.video_streams.get_mut(topic) {
            stream.is_recording = false;
        }
    }

    fn mcap_file_opened(&mut self, file_generation: u64, file_name: String) -> CaptureOutcome {
        let gate = self.record_gate();
        if !gate.recording_requested || file_generation != gate.desired_file_generation {
            return Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            };
        }
        let rotated = matches!(self.recording, RecordingState::Active(_));
        self.recording = RecordingState::Active(ActiveRecording {
            file_generation,
            file_name: file_name.clone(),
            bytes_written: 0,
        });
        let events = if rotated {
            vec![CaptureEvent::RecordingRotated { file_name }]
        } else {
            Vec::new()
        };
        Outcome::Applied {
            events,
            effects: Vec::new(),
        }
    }

    fn mcap_file_finished(&mut self, file_generation: u64) -> CaptureOutcome {
        match &self.recording {
            RecordingState::Active(active) if active.file_generation == file_generation => {
                if self.record_gate().desired_file_generation > file_generation {
                    // Task closed the previous file during rotation; keep recording requested.
                } else {
                    self.recording = RecordingState::Idle;
                    self.next_file_generation = 0;
                }
            }
            RecordingState::AwaitingMcapFile {
                file_generation: expected,
            } if *expected != file_generation => {
                // Stale finish for a file that is not the one we are waiting on.
            }
            RecordingState::Active(active) if active.file_generation != file_generation => {
                // Late finish for a superseded file must not clear the active recording.
            }
            _ => {}
        }
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn recording_bytes_written(&mut self, file_generation: u64, bytes: u64) -> CaptureOutcome {
        let RecordingState::Active(active) = &mut self.recording else {
            return Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            };
        };
        if active.file_generation != file_generation {
            return Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            };
        }
        active.bytes_written = bytes;
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }
}
