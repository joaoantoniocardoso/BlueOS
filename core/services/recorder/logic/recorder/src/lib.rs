//! The Recorder Domain: root Snapshot, public Command origins and Block composition.

#![no_std]

extern crate alloc;

pub mod durable;

use core::convert::Infallible;

use alloc::{borrow::ToOwned, string::ToString, vec::Vec};
use blueos_domain::{Command, Decision, Domain, DomainDurable, DomainQueries, Now, Outcome};
use blueos_jobs::{DomainJobs, JobEnd, JobId, JobStatus, Jobs};
use blueos_recorder_cameras::{
    Cameras, CamerasIoRequest, CamerasIoResult, CamerasObservedFact, CamerasTick, CamerasTimerKey,
};
use blueos_recorder_capture::{
    Capture, CaptureEvent, CaptureObservedFact, CaptureRequest, CaptureSettings, RecordGate,
    RecordingState as CaptureRecordingState,
};
use blueos_recorder_library::{
    InfallibleLibraryEvent, Library, LibraryIoRequest, LibraryIoResult, LibraryObservedFact,
    LibraryOperation, LibraryRejection, LibraryRepairOutcome, LibraryRequest,
    LibrarySnapshotOutcome, LibraryTick, LibraryTimerKey, snapshot_output_relative_path,
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
    /// Active recording, armed flag, bytes written and the record gate projection.
    pub capture: Capture,
    /// Recording catalog, repair and delete operations.
    pub library: Library,
    /// MAVLink camera protocol and video stream registration.
    pub cameras: Cameras,
    /// Long-running repair Jobs.
    pub jobs: Jobs,
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
        self.capture.record_gate()
    }

    /// Builds the projection the library operations Task follows: the library's operations, without a repair
    /// whose Job a client is cancelling, so the Task stops it.
    pub fn library_operations(&self) -> Vec<LibraryOperation> {
        self.library
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
                RecorderRequest::RepairRecording { job_id, path } => {
                    if let Some(reason) =
                        snapshot
                            .library
                            .repair_rejection_for_path(path.as_str(), active, now)
                    {
                        return Outcome::reject(LibraryRejection::new(reason));
                    }
                    map_library_outcome(snapshot.library.start_repair(path, job_id, active, now))
                }
                RecorderRequest::DeleteRecording { job_id, path } => {
                    map_library_outcome(snapshot.library.handle_request(
                        LibraryRequest::DeleteRecording { path, job_id },
                        active,
                        now,
                    ))
                }
                RecorderRequest::SnapshotRecording { job_id, path } => {
                    let output_path = snapshot_output_relative_path(path.as_str(), now);
                    map_library_outcome(snapshot.library.start_snapshot(
                        path,
                        output_path,
                        job_id,
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
            Command::Tick(RecorderTick::Restored) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
            Command::Tick(RecorderTick::Library(tick)) => {
                map_library_outcome(Library::handle_tick(tick))
            }
            Command::Tick(RecorderTick::Cameras(tick)) => {
                let outcome = snapshot.cameras.handle_tick(tick, now);
                sync_capture_video_recording(snapshot);
                map_cameras_outcome(outcome)
            }
            Command::IoResult(RecorderIoResult::Library(result)) => {
                if let LibraryIoResult::DeleteFinished { job_id, error, .. } = &result {
                    let end = error
                        .as_ref()
                        .map_or(JobEnd::Succeeded, |error| JobEnd::Aborted {
                            reason: error.to_string(),
                        });
                    let _ended = snapshot.jobs.end(*job_id, end);
                }
                map_library_outcome(snapshot.library.handle_io_result(result, active, now))
            }
            Command::IoResult(RecorderIoResult::Cameras(CamerasIoResult::PublishFailed)) => {
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::ObservedFact(RecorderObservedFact::Capture(fact)) => {
                map_capture_outcome(snapshot.capture.handle_observed_fact(fact))
            }
            Command::ObservedFact(RecorderObservedFact::Cameras(fact)) => {
                let outcome = snapshot.cameras.handle_observed_fact(fact, now);
                sync_capture_video_recording(snapshot);
                map_cameras_outcome(outcome)
            }
            Command::ObservedFact(RecorderObservedFact::Library(fact)) => {
                let end = job_end(&fact);
                let decision =
                    map_library_outcome(snapshot.library.handle_observed_fact(fact, active, now));
                if let Some(end) = end
                    && let Some(operation) = snapshot.library.ended_operation()
                {
                    let _ended = snapshot.jobs.end(operation.job_id(), end);
                }
                decision
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
            RecorderIoRequest::Library(LibraryIoRequest::Delete { path, job_id }) => {
                Command::IoResult(RecorderIoResult::Library(LibraryIoResult::DeleteFinished {
                    path,
                    job_id,
                    error: Some(error),
                }))
            }
            RecorderIoRequest::Cameras(_) => {
                let _ = error;
                Command::IoResult(RecorderIoResult::Cameras(CamerasIoResult::PublishFailed))
            }
        }
    }

    fn io_runs_on_blocking_thread(request: &Self::IoRequest) -> bool {
        matches!(
            request,
            RecorderIoRequest::Library(LibraryIoRequest::Scan | LibraryIoRequest::Delete { .. })
        )
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
fn active_recording_relative_path(snapshot: &RecorderSnapshot) -> Option<&str> {
    match &snapshot.capture.recording {
        CaptureRecordingState::Active(active) => Some(active.file_name.as_str()),
        _ => None,
    }
}

/// How the Job of a repair or snapshot ends, when `fact` reports that the operation ended.
fn job_end(fact: &LibraryObservedFact) -> Option<JobEnd> {
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
    outcome: Outcome<InfallibleLibraryEvent, LibraryTick, LibraryIoRequest, LibraryTimerKey>,
) -> Decision<RecorderDomain> {
    outcome.map(
        |never| match never {},
        RecorderTick::Library,
        RecorderIoRequest::Library,
        RecorderTimerKey::Library,
    )
}

fn map_cameras_outcome(
    outcome: Outcome<Infallible, CamerasTick, CamerasIoRequest, CamerasTimerKey>,
) -> Decision<RecorderDomain> {
    outcome.map(
        |never| match never {},
        RecorderTick::Cameras,
        RecorderIoRequest::Cameras,
        RecorderTimerKey::Cameras,
    )
}

fn sync_capture_video_recording(snapshot: &mut RecorderSnapshot) {
    for (topic, recording) in snapshot.cameras.video_recording_by_topic() {
        snapshot
            .capture
            .sync_video_topic_recording(topic, recording);
    }
}
