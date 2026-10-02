//! The Recorder Domain: root Snapshot, public Command origins and capture Block composition.

#![no_std]

extern crate alloc;

use core::convert::Infallible;

use blueos_domain::{Command, Decision, Domain, DomainQueries, Now, Outcome};
use blueos_recorder_capture::{
    Capture, CaptureEvent, CaptureObservedFact, CaptureRequest, CaptureSettings, RecordGate,
};

/// Persisted Recorder settings (the same fields as [`CaptureSettings`] until the api crate owns conversions).
pub type RecorderSettings = CaptureSettings;

/// Root Snapshot composed from Blocks.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecorderSnapshot {
    /// Active recording, armed flag, bytes written and the record gate projection.
    pub capture: Capture,
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

/// Marker type for the Recorder [`Domain`].
pub struct RecorderDomain;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Timer keys the Recorder Domain does not schedule yet.
pub enum RecorderTimerKey {}

impl Domain for RecorderDomain {
    type Snapshot = RecorderSnapshot;
    type Request = RecorderRequest;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = RecorderObservedFact;
    type Event = RecorderEvent;
    type IoRequest = Infallible;
    type TimerKey = RecorderTimerKey;

    fn handle(
        snapshot: &mut Self::Snapshot,
        command: Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact>,
        now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(request) => map_capture_outcome(match request {
                RecorderRequest::StartRecording { rotate_if_active } => snapshot
                    .capture
                    .handle_request(CaptureRequest::StartRecording { rotate_if_active }, now),
                RecorderRequest::StopRecording => snapshot
                    .capture
                    .handle_request(CaptureRequest::StopRecording, now),
                RecorderRequest::UpdateSettings(settings) => snapshot
                    .capture
                    .handle_request(CaptureRequest::UpdateSettings(settings), now),
                RecorderRequest::StartVideoRecording { topic } => snapshot
                    .capture
                    .handle_request(CaptureRequest::StartVideoRecording { topic }, now),
                RecorderRequest::StopVideoRecording { topic } => snapshot
                    .capture
                    .handle_request(CaptureRequest::StopVideoRecording { topic }, now),
            }),
            Command::IoResult(never) => match never {},
            Command::Tick(never) => match never {},
            Command::ObservedFact(RecorderObservedFact::Capture(fact)) => {
                map_capture_outcome(snapshot.capture.handle_observed_fact(fact))
            }
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: blueos_domain::IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
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
