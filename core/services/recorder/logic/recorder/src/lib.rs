//! The Recorder Domain: root Snapshot, public Command origins and Block composition.

#![no_std]

extern crate alloc;

use core::convert::Infallible;

use alloc::{borrow::ToOwned, vec::Vec};
use blueos_domain::{Command, Decision, Domain, DomainQueries, Now, Outcome};
use blueos_recorder_capture::{
    Capture, CaptureEvent, CaptureObservedFact, CaptureRequest, CaptureSettings, RecordGate,
    RecordingState as CaptureRecordingState,
};
use blueos_recorder_library::{
    Library, LibraryIoRequest, LibraryIoResult, LibraryRequest, LibraryTick, LibraryTimerKey,
};
use blueos_recorder_paths::RecordingRelativePath;

/// Persisted Recorder settings (the same fields as [`CaptureSettings`] until the api crate owns conversions).
pub type RecorderSettings = CaptureSettings;

/// Root Snapshot composed from Blocks.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecorderSnapshot {
    /// Active recording, armed flag, bytes written and the record gate projection.
    pub capture: Capture,
    /// Recording catalog and delete operations.
    pub library: Library,
}

/// Client Commands for the Recorder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderRequest {
    /// Ask the data plane to start or rotate recording via [`RecordGate`].
    StartRecording {
        /// When already recording, bump the desired file generation so the Task rotates.
        rotate_if_active: bool,
    },
    /// Ask the data plane to stop recording via [`RecordGate`].
    StopRecording,
    /// Updates settings; [`RecorderSettings::auto_start_recording`] is applied live.
    UpdateSettings(RecorderSettings),
    /// Marks a video topic as recording for the record gate.
    StartVideoRecording {
        /// Video topic (`video/...`).
        topic: alloc::string::String,
    },
    /// Stops a video topic in the record gate.
    StopVideoRecording {
        /// Video topic to stop.
        topic: alloc::string::String,
    },
    /// Kernel startup hook: applies [`RecorderSettings::auto_start_recording`] when enabled.
    Startup,
    /// Removes a finished recording from the library folder.
    DeleteRecording {
        /// Path validated at the api boundary.
        path: RecordingRelativePath,
    },
}

/// Observed facts from the data plane Task and adapters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderObservedFact {
    /// Fact from the capture Block.
    Capture(CaptureObservedFact),
}

/// Domain events published after a Command is acknowledged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderEvent {
    /// Event from the capture Block.
    Capture(CaptureEvent),
}

/// Queries against the Recorder Snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderQuery {
    /// The projection the data plane Task follows.
    RecordGate,
}

/// IO requests scheduled by the Recorder Domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderIoRequest {
    /// Library folder scan or delete.
    Library(LibraryIoRequest),
}

/// IO results delivered back to the Domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderIoResult {
    /// Library IO finished.
    Library(LibraryIoResult),
}

/// Timer ticks for the Recorder Domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderTick {
    /// Library rescan timer.
    Library(LibraryTick),
}

/// Timer keys for the Recorder Domain.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RecorderTimerKey {
    /// Library rescan timer.
    Library(LibraryTimerKey),
}

/// Marker type for the Recorder [`Domain`].
pub struct RecorderDomain;

impl Domain for RecorderDomain {
    type Snapshot = RecorderSnapshot;
    type Request = RecorderRequest;
    type IoResult = RecorderIoResult;
    type Tick = RecorderTick;
    type ObservedFact = RecorderObservedFact;
    type Event = RecorderEvent;
    type IoRequest = RecorderIoRequest;
    type TimerKey = RecorderTimerKey;

    fn handle(
        snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        now: Now,
    ) -> Decision<Self> {
        let active_recording_relative_path =
            active_recording_relative_path(snapshot).map(str::to_owned);
        let active = active_recording_relative_path.as_deref();
        match command {
            Command::Request(request) => match request {
                RecorderRequest::DeleteRecording { path } => {
                    map_library_outcome(snapshot.library.handle_request(
                        LibraryRequest::DeleteRecording { path },
                        active,
                        now,
                    ))
                }
                RecorderRequest::Startup => merge_startup(snapshot, now),
                RecorderRequest::StartRecording { rotate_if_active } => map_capture_outcome(
                    snapshot
                        .capture
                        .handle_request(CaptureRequest::StartRecording { rotate_if_active }, now),
                ),
                RecorderRequest::StopRecording => map_capture_outcome(
                    snapshot
                        .capture
                        .handle_request(CaptureRequest::StopRecording, now),
                ),
                RecorderRequest::UpdateSettings(settings) => map_capture_outcome(
                    snapshot
                        .capture
                        .handle_request(CaptureRequest::UpdateSettings(settings), now),
                ),
                RecorderRequest::StartVideoRecording { topic } => map_capture_outcome(
                    snapshot
                        .capture
                        .handle_request(CaptureRequest::StartVideoRecording { topic }, now),
                ),
                RecorderRequest::StopVideoRecording { topic } => map_capture_outcome(
                    snapshot
                        .capture
                        .handle_request(CaptureRequest::StopVideoRecording { topic }, now),
                ),
            },
            Command::Tick(RecorderTick::Library(tick)) => {
                map_library_outcome(Library::handle_tick(tick))
            }
            Command::IoResult(RecorderIoResult::Library(result)) => {
                map_library_outcome(snapshot.library.handle_io_result(result, active, now))
            }
            Command::ObservedFact(RecorderObservedFact::Capture(fact)) => {
                map_capture_outcome(snapshot.capture.handle_observed_fact(fact))
            }
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        error: blueos_domain::IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {
            RecorderIoRequest::Library(LibraryIoRequest::Scan) => {
                let _ = error;
                Command::IoResult(RecorderIoResult::Library(LibraryIoResult::ScanFailed))
            }
            RecorderIoRequest::Library(LibraryIoRequest::Delete { path }) => {
                Command::IoResult(RecorderIoResult::Library(LibraryIoResult::DeleteFinished {
                    path,
                    error: Some(error),
                }))
            }
        }
    }

    fn io_runs_on_blocking_thread(request: &Self::IoRequest) -> bool {
        matches!(request, RecorderIoRequest::Library(_))
    }
}

impl DomainQueries for RecorderDomain {
    type Query = RecorderQuery;
    type Response = RecordGate;

    fn query(snapshot: &Self::Snapshot, query: Self::Query, _now: Now) -> Self::Response {
        match query {
            RecorderQuery::RecordGate => snapshot.capture.record_gate(),
        }
    }
}

/// Relative path in the library for the file being written (flat folder: same as the active base name).
fn active_recording_relative_path(snapshot: &RecorderSnapshot) -> Option<&str> {
    match &snapshot.capture.recording {
        CaptureRecordingState::Active(active) => Some(active.file_name.as_str()),
        _ => None,
    }
}

fn merge_startup(snapshot: &mut RecorderSnapshot, now: Now) -> Decision<RecorderDomain> {
    let capture = if snapshot.capture.settings.auto_start_recording {
        snapshot.capture.handle_request(
            CaptureRequest::StartRecording {
                rotate_if_active: false,
            },
            now,
        )
    } else {
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    };
    let library = Outcome::Applied {
        events: Vec::new(),
        effects: Library::initial_effects(),
    };
    merge_capture_and_library(map_capture_outcome(capture), map_library_outcome(library))
}

fn merge_capture_and_library(
    capture: Decision<RecorderDomain>,
    library: Decision<RecorderDomain>,
) -> Decision<RecorderDomain> {
    match library {
        Outcome::Applied {
            effects: library_effects,
            ..
        } => match capture {
            Outcome::Applied {
                events,
                mut effects,
            } => {
                effects.extend(library_effects);
                Outcome::Applied { events, effects }
            }
            Outcome::Rejected { .. } => Outcome::Applied {
                events: Vec::new(),
                effects: library_effects,
            },
        },
        Outcome::Rejected { reason } => Outcome::Rejected { reason },
    }
}

fn map_capture_outcome(
    outcome: Outcome<CaptureEvent, Infallible, Infallible, Infallible>,
) -> Decision<RecorderDomain> {
    outcome.map(
        RecorderEvent::Capture,
        |never| match never {},
        |never| match never {},
        |never| match never {},
    )
}

fn map_library_outcome(
    outcome: Outcome<
        blueos_recorder_library::InfallibleLibraryEvent,
        LibraryTick,
        LibraryIoRequest,
        LibraryTimerKey,
    >,
) -> Decision<RecorderDomain> {
    outcome.map(
        |never| match never {},
        RecorderTick::Library,
        RecorderIoRequest::Library,
        RecorderTimerKey::Library,
    )
}
