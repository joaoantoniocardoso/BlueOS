//! [`Capture`] Block orchestration and request handling.

use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};
use core::time::Duration;

use blueos_domain::{Now, Outcome};

use crate::{
    ActiveRecording, CaptureEvent, CaptureObservedFact, CaptureOutcome, CaptureRejection,
    CaptureRequest, CaptureSettings, MAVLINK_RAW_TOPIC_PREFIX, MAVLINK_TOPIC_PREFIX, RecordGate,
    RecordingState, VIDEO_TOPIC_PREFIX, VideoStream,
};

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
            .map(|(topic, _)| topic)
            .cloned()
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

    /// Applies a client Request.
    pub fn handle_request(&mut self, request: CaptureRequest, _now: Now) -> CaptureOutcome {
        match request {
            CaptureRequest::StartRecording { rotate_if_active } => {
                self.start_recording(rotate_if_active)
            }
            CaptureRequest::StopRecording => {
                stop_recording_outcome(self, idle_stop_rejection(&self.recording))
            }
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
            CaptureObservedFact::RecordingSamplesDropped {
                file_generation,
                samples,
            } => self.recording_samples_dropped(file_generation, samples),
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
            RecordingState::Active(_) if !rotate_if_active => {
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
            samples_dropped: 0,
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

    fn recording_samples_dropped(&mut self, file_generation: u64, samples: u64) -> CaptureOutcome {
        if let RecordingState::Active(active) = &mut self.recording
            && active.file_generation == file_generation
        {
            active.samples_dropped = samples;
        }
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }
}

fn idle_stop_rejection(recording: &RecordingState) -> Option<CaptureRejection> {
    matches!(recording, RecordingState::Idle).then_some(CaptureRejection::NotRecording)
}

fn stop_recording_outcome(
    capture: &mut Capture,
    rejection: Option<CaptureRejection>,
) -> CaptureOutcome {
    match rejection {
        Some(rejection) => rejected_stop_outcome(rejection),
        None => commit_stop_recording(capture),
    }
}

fn rejected_stop_outcome(rejection: CaptureRejection) -> CaptureOutcome {
    Outcome::reject(rejection)
}

fn commit_stop_recording(capture: &mut Capture) -> CaptureOutcome {
    Outcome::Applied {
        events: finish_active_recording(capture),
        effects: Vec::new(),
    }
}

fn finish_active_recording(capture: &mut Capture) -> Vec<CaptureEvent> {
    capture.recording = RecordingState::Idle;
    capture.next_file_generation = 0;
    clear_video_recording_flags(&mut capture.video_streams);
    vec![CaptureEvent::RecordingStopped]
}

fn clear_video_recording_flags(video_streams: &mut BTreeMap<String, VideoStream>) {
    for stream in video_streams.values_mut() {
        stream.is_recording = false;
    }
}
