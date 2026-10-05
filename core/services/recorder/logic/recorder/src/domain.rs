use alloc::{borrow::ToOwned, string::ToString, vec::Vec};

use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_jobs::{JobEnd, JobId};
use blueos_recorder_cameras::CamerasIoResult;
use blueos_recorder_capture::CaptureRequest;
use blueos_recorder_library::{
    LibraryIoRequest, LibraryIoResult, LibraryObservedFact, LibraryRejection, LibraryRequest,
    RepairStartSpec, SnapshotStartSpec, handle_io_result, handle_library_request,
    handle_observed_fact, handle_tick, repair_rejection_for_path, snapshot_output_relative_path,
    start_repair, start_snapshot,
};

use super::{
    RecorderDomain, RecorderEvent, RecorderIoRequest, RecorderIoResult, RecorderObservedFact,
    RecorderRequest, RecorderSnapshot, RecorderTick, RecorderTimerKey,
    active_recording_relative_path, job_end, map_cameras_outcome, map_capture_outcome,
    map_library_outcome, merge_startup, sync_capture_video_recording,
};

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
            Command::Request(request) => handle_request(snapshot, request, active, now),
            Command::Tick(RecorderTick::Restored) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
            Command::Tick(RecorderTick::Library(tick)) => map_library_outcome(handle_tick(tick)),
            Command::Tick(RecorderTick::Cameras(tick)) => {
                let outcome = snapshot.blocks.cameras.handle_tick(tick, now);
                sync_capture_video_recording(snapshot);
                map_cameras_outcome(outcome)
            }
            Command::IoResult(RecorderIoResult::Library(result)) => {
                handle_library_io_result(snapshot, result, active, now)
            }
            Command::IoResult(RecorderIoResult::Cameras(CamerasIoResult::PublishFailed)) => {
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::ObservedFact(RecorderObservedFact::Capture(fact)) => {
                map_capture_outcome(snapshot.blocks.capture.handle_observed_fact(fact))
            }
            Command::ObservedFact(RecorderObservedFact::Cameras(fact)) => {
                let outcome = snapshot.blocks.cameras.handle_observed_fact(fact, now);
                sync_capture_video_recording(snapshot);
                map_cameras_outcome(outcome)
            }
            Command::ObservedFact(RecorderObservedFact::Library(fact)) => {
                handle_library_observed_fact(snapshot, fact, active, now)
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

fn handle_request(
    snapshot: &mut RecorderSnapshot,
    request: RecorderRequest,
    active: Option<&str>,
    now: Now,
) -> Decision<RecorderDomain> {
    match request {
        RecorderRequest::RepairRecording { job_id, path } => {
            handle_repair_recording(snapshot, job_id, path, active, now)
        }
        RecorderRequest::DeleteRecording { job_id, path } => {
            map_library_outcome(handle_library_request(
                &mut snapshot.blocks.library,
                LibraryRequest::DeleteRecording { path, job_id },
                active,
                now,
            ))
        }
        RecorderRequest::SnapshotRecording { job_id, path } => {
            handle_snapshot_recording(snapshot, job_id, path, active, now)
        }
        RecorderRequest::Startup => merge_startup(snapshot, now),
        RecorderRequest::StartRecording { rotate_if_active } => map_capture_outcome(
            snapshot
                .blocks
                .capture
                .handle_request(CaptureRequest::StartRecording { rotate_if_active }, now),
        ),
        RecorderRequest::StopRecording => map_capture_outcome(
            snapshot
                .blocks
                .capture
                .handle_request(CaptureRequest::StopRecording, now),
        ),
        RecorderRequest::UpdateSettings(settings) => map_capture_outcome(
            snapshot
                .blocks
                .capture
                .handle_request(CaptureRequest::UpdateSettings(settings), now),
        ),
        RecorderRequest::StartVideoRecording { topic } => map_capture_outcome(
            snapshot
                .blocks
                .capture
                .handle_request(CaptureRequest::StartVideoRecording { topic }, now),
        ),
        RecorderRequest::StopVideoRecording { topic } => map_capture_outcome(
            snapshot
                .blocks
                .capture
                .handle_request(CaptureRequest::StopVideoRecording { topic }, now),
        ),
    }
}

fn handle_repair_recording(
    snapshot: &mut RecorderSnapshot,
    job_id: JobId,
    path: blueos_recorder_paths::RecordingRelativePath,
    active: Option<&str>,
    now: Now,
) -> Decision<RecorderDomain> {
    match repair_rejection(snapshot, &path, active, now) {
        Some(decision) => decision,
        None => map_library_outcome(start_repair(
            &mut snapshot.blocks.library,
            RepairStartSpec {
                path,
                job_id,
                active_recording_relative_path: active,
                now,
            },
        )),
    }
}

fn repair_rejection(
    snapshot: &RecorderSnapshot,
    path: &blueos_recorder_paths::RecordingRelativePath,
    active: Option<&str>,
    now: Now,
) -> Option<Decision<RecorderDomain>> {
    repair_rejection_for_path(&snapshot.blocks.library, path.as_str(), active, now)
        .map(|reason| Outcome::reject(LibraryRejection::new(reason)))
}

fn handle_snapshot_recording(
    snapshot: &mut RecorderSnapshot,
    job_id: JobId,
    path: blueos_recorder_paths::RecordingRelativePath,
    active: Option<&str>,
    now: Now,
) -> Decision<RecorderDomain> {
    let output_path = snapshot_output_relative_path(path.as_str(), now);
    map_library_outcome(start_snapshot(
        &mut snapshot.blocks.library,
        SnapshotStartSpec {
            path,
            output_path,
            job_id,
            active_recording_relative_path: active,
            now,
        },
    ))
}

fn handle_library_io_result(
    snapshot: &mut RecorderSnapshot,
    result: LibraryIoResult,
    active: Option<&str>,
    now: Now,
) -> Decision<RecorderDomain> {
    if let LibraryIoResult::DeleteFinished { job_id, error, .. } = &result {
        let end = error
            .as_ref()
            .map_or(JobEnd::Succeeded, |error| JobEnd::Aborted {
                reason: error.to_string(),
            });
        let _ended = snapshot.jobs.end(*job_id, end);
    }
    map_library_outcome(handle_io_result(
        &mut snapshot.blocks.library,
        result,
        active,
        now,
    ))
}

fn handle_library_observed_fact(
    snapshot: &mut RecorderSnapshot,
    fact: LibraryObservedFact,
    active: Option<&str>,
    now: Now,
) -> Decision<RecorderDomain> {
    let end = job_end(&fact);
    let decision = map_library_outcome(handle_observed_fact(
        &mut snapshot.blocks.library,
        fact,
        active,
        now,
    ));
    if let Some(end) = end
        && let Some(operation) = snapshot.blocks.library.work().queue().ended_operation()
    {
        let _ended = snapshot.jobs.end(operation.job_id(), end);
    }
    decision
}
