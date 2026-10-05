//! The Recorder Domain: root Snapshot, public Command origins and Block composition.

#![no_std]

extern crate alloc;

pub mod durable;

mod domain;

use core::convert::Infallible;

use alloc::{string::ToString, vec::Vec};
use blueos_domain::{Decision, DomainDurable, DomainQueries, Now, Outcome};
use blueos_jobs::{DomainJobs, JobEnd, JobId, JobStatus, Jobs};
use blueos_recorder_cameras::{
    Cameras, CamerasIoRequest, CamerasIoResult, CamerasObservedFact, CamerasTick, CamerasTimerKey,
};
use blueos_recorder_capture::{
    Capture, CaptureEvent, CaptureObservedFact, CaptureRequest, CaptureSettings, RecordGate,
    RecordingState as CaptureRecordingState,
};
use blueos_recorder_library::{
    Library, LibraryIoRequest, LibraryIoResult, LibraryObservedFact, LibraryOperation,
    LibraryRepairOutcome, LibrarySnapshotOutcome, LibraryTick, LibraryTimerKey,
};
use blueos_recorder_paths::RecordingRelativePath;

use durable::RecorderDurableState;

/// Persisted Recorder settings (the same fields as [`CaptureSettings`] until the api crate owns conversions).
pub type RecorderSettings = CaptureSettings;

/// Root Snapshot composed from Blocks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecorderSnapshot {
    /// Part of the Snapshot written across restarts.
    pub durable: RecorderDurableState,
    /// Capture, library, and cameras Blocks.
    pub blocks: RecorderBlocks,
    /// Long-running repair Jobs.
    pub jobs: Jobs,
}

/// Live recorder Blocks composed in the Domain Snapshot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecorderBlocks {
    /// Active recording, armed flag, bytes written and the record gate projection.
    pub capture: Capture,
    /// Recording catalog, repair and delete operations.
    pub library: Library,
    /// MAVLink camera protocol and video stream registration.
    pub cameras: Cameras,
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
    /// Rewrites an unindexed recording in place.
    RepairRecording {
        /// The Job the repair runs as.
        job_id: JobId,
        /// Path validated at the api boundary.
        path: RecordingRelativePath,
    },
    /// Removes a finished recording from the library folder.
    DeleteRecording {
        /// The Job the delete runs as.
        job_id: JobId,
        /// Path validated at the api boundary.
        path: RecordingRelativePath,
    },
    /// Writes an indexed copy of a recording (typically while it is still being written).
    SnapshotRecording {
        /// The Job the snapshot runs as.
        job_id: JobId,
        /// Path validated at the api boundary.
        path: RecordingRelativePath,
    },
}

/// Observed facts from the data plane Task and adapters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderObservedFact {
    /// Fact from the capture Block.
    Capture(CaptureObservedFact),
    /// Fact from the cameras Block.
    Cameras(CamerasObservedFact),
    /// Repair progress and the end of a repair or snapshot, from the library operations Task.
    Library(LibraryObservedFact),
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
    /// Library folder scan, repair, or delete.
    Library(LibraryIoRequest),
    /// IO from the cameras Block (MAVLink egress).
    Cameras(CamerasIoRequest),
}

/// IO results delivered back to the Domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderIoResult {
    /// Library IO finished.
    Library(LibraryIoResult),
    /// Result from the cameras Block.
    Cameras(CamerasIoResult),
}

/// Timer ticks for the Recorder Domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderTick {
    /// Kernel delivers this once after durable state restore (D-28).
    Restored,
    /// Library rescan timer.
    Library(LibraryTick),
    /// Tick from the cameras Block.
    Cameras(CamerasTick),
}

/// Timer keys for the Recorder Domain.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RecorderTimerKey {
    /// Library rescan timer.
    Library(LibraryTimerKey),
    /// Timer owned by the cameras Block.
    Cameras(CamerasTimerKey),
}

/// Marker type for the Recorder [`Domain`].
pub struct RecorderDomain;

impl RecorderSnapshot {
    /// Builds the projection the data plane and MAVLink Tasks follow.
    pub fn record_gate(&self) -> RecordGate {
        self.blocks.capture.record_gate()
    }

    /// Builds the projection the library operations Task follows: the library's operations, without a repair
    /// whose Job a client is cancelling, so the Task stops it.
    pub fn library_operations(&self) -> Vec<LibraryOperation> {
        self.blocks
            .library
            .work()
            .queue()
            .operations()
            .iter()
            .filter(|operation| match operation {
                LibraryOperation::Repair { job_id, .. } => !self
                    .jobs
                    .job(*job_id)
                    .is_some_and(|job| job.status == JobStatus::Canceling),
                LibraryOperation::Snapshot { .. } => true,
            })
            .cloned()
            .collect()
    }
}

impl DomainDurable for RecorderDomain {
    type DurableState = RecorderDurableState;

    fn durable_state(snapshot: &Self::Snapshot) -> &Self::DurableState {
        &snapshot.durable
    }

    fn set_durable_state(snapshot: &mut Self::Snapshot, state: Self::DurableState) {
        snapshot.durable = state;
    }

    fn restored_tick() -> Self::Tick {
        RecorderTick::Restored
    }
}

impl DomainJobs for RecorderDomain {
    fn jobs(snapshot: &Self::Snapshot) -> &Jobs {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut Self::Snapshot) -> &mut Jobs {
        &mut snapshot.jobs
    }
}

impl DomainQueries for RecorderDomain {
    type Query = RecorderQuery;
    type Response = RecordGate;

    fn query(snapshot: &Self::Snapshot, query: Self::Query, _now: Now) -> Self::Response {
        match query {
            RecorderQuery::RecordGate => snapshot.record_gate(),
        }
    }
}

/// Relative path in the library for the file being written (flat folder: same as the active base name).
pub(crate) fn active_recording_relative_path(snapshot: &RecorderSnapshot) -> Option<&str> {
    match &snapshot.blocks.capture.recording {
        CaptureRecordingState::Active(active) => Some(active.file_name.as_str()),
        _ => None,
    }
}

/// How the Job of a repair or snapshot ends, when `fact` reports that the operation ended.
pub(crate) fn job_end(fact: &LibraryObservedFact) -> Option<JobEnd> {
    match fact {
        LibraryObservedFact::RepairProgress(_) => None,
        LibraryObservedFact::RepairFinished {
            outcome: LibraryRepairOutcome::Succeeded,
            ..
        }
        | LibraryObservedFact::SnapshotFinished {
            outcome: LibrarySnapshotOutcome::Succeeded,
            ..
        } => Some(JobEnd::Succeeded),
        LibraryObservedFact::RepairFinished {
            outcome: LibraryRepairOutcome::Cancelled,
            ..
        } => Some(JobEnd::Canceled),
        LibraryObservedFact::RepairFinished {
            outcome: LibraryRepairOutcome::Failed(failure),
            ..
        }
        | LibraryObservedFact::SnapshotFinished {
            outcome: LibrarySnapshotOutcome::Failed(failure),
            ..
        } => Some(JobEnd::Aborted {
            reason: failure.to_string(),
        }),
    }
}

pub(crate) fn merge_startup(snapshot: &mut RecorderSnapshot, now: Now) -> Decision<RecorderDomain> {
    let capture = if snapshot.blocks.capture.settings.auto_start_recording {
        snapshot.blocks.capture.handle_request(
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
        effects: blueos_recorder_library::initial_effects(),
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

pub(crate) fn map_capture_outcome(
    outcome: Outcome<CaptureEvent, Infallible, Infallible, Infallible>,
) -> Decision<RecorderDomain> {
    outcome.map(
        RecorderEvent::Capture,
        |never| match never {},
        |never| match never {},
        |never| match never {},
    )
}

pub(crate) fn map_library_outcome(
    outcome: Outcome<Infallible, LibraryTick, LibraryIoRequest, LibraryTimerKey>,
) -> Decision<RecorderDomain> {
    outcome.map(
        |never| match never {},
        RecorderTick::Library,
        RecorderIoRequest::Library,
        RecorderTimerKey::Library,
    )
}

pub(crate) fn map_cameras_outcome(
    outcome: Outcome<Infallible, CamerasTick, CamerasIoRequest, CamerasTimerKey>,
) -> Decision<RecorderDomain> {
    outcome.map(
        |never| match never {},
        RecorderTick::Cameras,
        RecorderIoRequest::Cameras,
        RecorderTimerKey::Cameras,
    )
}

pub(crate) fn sync_capture_video_recording(snapshot: &mut RecorderSnapshot) {
    for (topic, recording) in snapshot.blocks.cameras.video_recording_by_topic() {
        snapshot
            .blocks
            .capture
            .sync_video_topic_recording(topic, recording);
    }
}
