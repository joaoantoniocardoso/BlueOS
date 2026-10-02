//! The Recorder Domain: root Snapshot, public Command origins and Block composition.

#![no_std]

extern crate alloc;

pub mod durable;
pub mod job;

use core::{convert::Infallible, fmt};

use alloc::{borrow::ToOwned, vec::Vec};
use blueos_domain::{Command, Decision, Domain, DomainDurable, DomainQueries, Now, Outcome};
use blueos_jobs::{DomainJobs, JobEnd, JobGraph, JobId, JobKind, JobStatus, Jobs, LeafJob};
use blueos_recorder_cameras::{
    Cameras, CamerasIoRequest, CamerasIoResult, CamerasObservedFact, CamerasTick, CamerasTimerKey,
};
use blueos_recorder_capture::{
    Capture, CaptureEvent, CaptureObservedFact, CaptureRequest, CaptureSettings, RecordGate,
    RecordingState as CaptureRecordingState,
};
use blueos_recorder_library::{
    Library, LibraryEvent, LibraryIoRequest, LibraryIoResult, LibraryRejection,
    LibraryRepairOutcome, LibraryRepairProgress, LibraryRequest, LibrarySnapshotOutcome,
    LibraryTick, LibraryTimerKey, RecordingOperationEvent, RepairFailure,
    snapshot_output_relative_path,
};
use blueos_recorder_paths::RecordingRelativePath;

use durable::RecorderDurableState;
use job::RecorderJobStep;

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
    pub jobs: Jobs<RecorderJobStep>,
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
        /// Path validated at the api boundary.
        path: RecordingRelativePath,
    },
    /// Stops a running repair without touching the original file.
    CancelRepair {
        /// Path validated at the api boundary.
        path: RecordingRelativePath,
    },
    /// Removes a finished recording from the library folder.
    DeleteRecording {
        /// Path validated at the api boundary.
        path: RecordingRelativePath,
    },
    /// Writes an indexed copy of a recording (typically while it is still being written).
    SnapshotRecording {
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
    /// Repair read offset from library IO.
    LibraryRepairProgress(LibraryRepairProgress),
}

/// Domain events published after a Command is acknowledged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecorderEvent {
    /// Event from the capture Block.
    Capture(CaptureEvent),
    /// Repair, snapshot, or delete finished.
    RecordingOperation(RecordingOperationEvent),
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
                RecorderRequest::RepairRecording { path } => {
                    if let Some(reason) =
                        snapshot
                            .library
                            .repair_rejection_for_path(path.as_str(), active, now)
                    {
                        return Outcome::reject(LibraryRejection::new(reason));
                    }
                    let started =
                        snapshot
                            .jobs
                            .start(JobGraph::Leaf(RecorderJobStep::RepairRecording {
                                path: path.clone(),
                            }));
                    let leaf_job_id = started
                        .leaves
                        .first()
                        .expect("repair Job has one leaf")
                        .job_id;
                    let library = snapshot.library.start_repair(
                        path,
                        started.job_id,
                        leaf_job_id,
                        active,
                        now,
                    );
                    merge_job_and_library(started.leaves, library)
                }
                RecorderRequest::CancelRepair { path } => {
                    let relative = path.as_str();
                    if let Some(root_job_id) = snapshot.library.repair_root_job_id(relative) {
                        let _cancelling = snapshot.jobs.cancel(root_job_id);
                    }
                    map_library_outcome(snapshot.library.handle_request(
                        LibraryRequest::CancelRepair { path },
                        active,
                        now,
                    ))
                }
                RecorderRequest::DeleteRecording { path } => {
                    map_library_outcome(snapshot.library.handle_request(
                        LibraryRequest::DeleteRecording { path },
                        active,
                        now,
                    ))
                }
                RecorderRequest::SnapshotRecording { path } => {
                    let output_path = snapshot_output_relative_path(path.as_str(), now);
                    map_library_outcome(snapshot.library.start_snapshot(
                        path,
                        output_path,
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
            Command::Tick(RecorderTick::Restored) => merge_restored(snapshot),
            Command::Tick(RecorderTick::Library(tick)) => {
                map_library_outcome(Library::handle_tick(tick))
            }
            Command::Tick(RecorderTick::Cameras(tick)) => {
                let outcome = snapshot.cameras.handle_tick(tick, now);
                sync_capture_video_recording(snapshot);
                map_cameras_outcome(outcome)
            }
            Command::IoResult(RecorderIoResult::Library(result)) => {
                let job_id = match &result {
                    LibraryIoResult::RepairFinished { path, .. } => {
                        snapshot.library.repair_leaf_job_id(path.as_str())
                    }
                    _ => None,
                };
                let decision =
                    map_library_outcome(snapshot.library.handle_io_result(result, active, now));
                if let Some(job_id) = job_id {
                    finish_repair_job(snapshot, job_id, &decision);
                }
                decision
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
            Command::ObservedFact(RecorderObservedFact::LibraryRepairProgress(progress)) => {
                map_library_outcome(
                    snapshot
                        .library
                        .handle_repair_progress(progress, active, now),
                )
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
            RecorderIoRequest::Library(LibraryIoRequest::Repair { path }) => {
                Command::IoResult(RecorderIoResult::Library(LibraryIoResult::RepairFinished {
                    path,
                    outcome: LibraryRepairOutcome::Failed(RepairFailure::Io),
                }))
            }
            RecorderIoRequest::Library(LibraryIoRequest::CancelRepair { path }) => {
                let _ = (path, error);
                Command::IoResult(RecorderIoResult::Library(LibraryIoResult::ScanFailed))
            }
            RecorderIoRequest::Library(LibraryIoRequest::Snapshot { path, output_path }) => {
                Command::IoResult(RecorderIoResult::Library(
                    LibraryIoResult::SnapshotFinished {
                        path,
                        output_path,
                        outcome: LibrarySnapshotOutcome::Failed(RepairFailure::Io),
                    },
                ))
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
    type Step = RecorderJobStep;

    fn jobs(snapshot: &Self::Snapshot) -> &Jobs<Self::Step> {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut Self::Snapshot) -> &mut Jobs<Self::Step> {
        &mut snapshot.jobs
    }
}

impl fmt::Display for RecorderJobStep {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepairRecording { path } => {
                formatter.write_str("repair ")?;
                formatter.write_str(path.as_str())
            }
        }
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

fn finish_repair_job(
    snapshot: &mut RecorderSnapshot,
    job_id: JobId,
    decision: &Decision<RecorderDomain>,
) {
    let end = match decision {
        Outcome::Applied { events, .. } => {
            let cancelled = events.iter().any(|event| {
                matches!(
                    event,
                    RecorderEvent::RecordingOperation(operation)
                        if operation.cancelled
                )
            });
            let failed = events.iter().any(|event| {
                matches!(
                    event,
                    RecorderEvent::RecordingOperation(operation)
                        if !operation.succeeded && !operation.cancelled
                )
            });
            if cancelled {
                JobEnd::Cancelled
            } else if failed {
                JobEnd::Failed
            } else {
                JobEnd::Succeeded
            }
        }
        Outcome::Rejected { .. } => JobEnd::Failed,
    };
    let _finished = snapshot.jobs.finish(job_id, end);
}

fn merge_job_and_library(
    _leaves: Vec<LeafJob<RecorderJobStep>>,
    library: Outcome<LibraryEvent, LibraryTick, LibraryIoRequest, LibraryTimerKey>,
) -> Decision<RecorderDomain> {
    map_library_outcome(library)
}

fn merge_restored(snapshot: &mut RecorderSnapshot) -> Decision<RecorderDomain> {
    let interrupted_repairs: Vec<JobId> = snapshot
        .jobs
        .list()
        .into_iter()
        .filter(|view| view.status == JobStatus::Interrupted)
        .filter(|view| {
            matches!(
                view.kind,
                JobKind::Leaf(RecorderJobStep::RepairRecording { .. })
            )
        })
        .map(|view| view.job_id)
        .collect();
    for job_id in interrupted_repairs {
        let _finished = snapshot.jobs.finish(job_id, JobEnd::Failed);
    }
    Outcome::Applied {
        events: Vec::new(),
        effects: Vec::new(),
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
    outcome: Outcome<LibraryEvent, LibraryTick, LibraryIoRequest, LibraryTimerKey>,
) -> Decision<RecorderDomain> {
    outcome.map(
        |event| match event {
            LibraryEvent::Operation(operation) => RecorderEvent::RecordingOperation(operation),
        },
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
