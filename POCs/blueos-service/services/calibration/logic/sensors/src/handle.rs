use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::jobs::{
    cancellable_roots, job_spec_of, parse_sensor, preflight_sequence, running_named,
};
use crate::keys::{CALIBRATION_STATE_SNAPSHOT, CALIBRATION_STREAM_EVENTS};
use crate::model::{Action, CalibrationStatus, Event, IoRequest, Sensor, Snapshot, UseCase};
use blueos_cqrs::Effect;
use blueos_jobs::{JobGraph, JobId, JobSpec, JobStatus, Jobs};

pub(crate) fn start_sensor(
    snapshot: &mut Snapshot,
    jobs: &mut Jobs,
    sensor: Sensor,
    use_case: UseCase,
) -> Vec<Event> {
    snapshot.set_status(sensor, CalibrationStatus::Running);
    snapshot.clear_offset(sensor);
    snapshot.last_ack = None;
    jobs.enqueue(preflight_sequence(sensor));
    vec![
        Event::Requested { use_case },
        Event::Progress {
            sensor,
            detail: "started".to_string(),
        },
    ]
}

pub(crate) fn start_stationary(snapshot: &mut Snapshot, jobs: &mut Jobs) -> Vec<Event> {
    snapshot.gyro = CalibrationStatus::Running;
    snapshot.baro = CalibrationStatus::Running;
    snapshot.gyro_offset = None;
    snapshot.ground_pressure = None;
    snapshot.last_ack = None;
    jobs.enqueue(JobGraph::Sequence(vec![
        preflight_sequence(Sensor::Gyro),
        preflight_sequence(Sensor::Baro),
    ]));
    vec![
        Event::Requested {
            use_case: UseCase::CalibrateStationarySensors,
        },
        Event::Progress {
            sensor: Sensor::Gyro,
            detail: "stationary".to_string(),
        },
    ]
}

pub(crate) fn cancel(snapshot: &mut Snapshot, jobs: &mut Jobs) -> Vec<Event> {
    let mut events = vec![Event::Requested {
        use_case: UseCase::CancelCalibration,
    }];
    let running_leaf = jobs
        .snapshot()
        .jobs
        .iter()
        .any(|job| job.job_spec.is_some() && job.status == JobStatus::Running);
    for job_id in cancellable_roots(jobs) {
        let _ = jobs.cancel(job_id); // already-terminal cancel is a no-op error we ignore
    }
    for sensor in [Sensor::Gyro, Sensor::Baro] {
        if snapshot.status_of(sensor) == CalibrationStatus::Running {
            snapshot.set_status(sensor, CalibrationStatus::Cancelled);
            events.push(Event::Cancelled { sensor });
        }
    }
    let payload = if running_leaf {
        b"cancel".to_vec()
    } else {
        Vec::new()
    };
    jobs.enqueue(JobGraph::Leaf(JobSpec {
        name: Action::CancelCalibration.name().to_string(),
        payload,
    }));
    events
}

pub(crate) fn job_progress(
    snapshot: &mut Snapshot,
    jobs: &mut Jobs,
    job_id: JobId,
    payload: &[u8],
) -> Vec<Event> {
    let job_spec = job_spec_of(jobs, job_id);
    let succeeded = payload != b"FAILED" && payload != b"fail";
    let _ = jobs.complete(job_id, succeeded); // already-terminal complete is a no-op
    let Some(job_spec) = job_spec else {
        return Vec::new();
    };
    let sensor = parse_sensor(&job_spec.payload).unwrap_or(Sensor::Gyro);
    if snapshot.status_of(sensor) == CalibrationStatus::Cancelled {
        return Vec::new();
    }
    if job_spec.name == Action::StartPreflightCalibration.name() {
        if succeeded {
            vec![Event::Progress {
                sensor,
                detail: "preflight".to_string(),
            }]
        } else {
            snapshot.set_status(sensor, CalibrationStatus::Failed);
            vec![Event::Failed {
                sensor,
                reason: "preflight failed".to_string(),
            }]
        }
    } else if job_spec.name == Action::ReadOffsets.name() && succeeded {
        if let Some(value) = snapshot.offset_of(sensor).or_else(|| parse_i32(payload)) {
            snapshot.set_offset(sensor, value);
        }
        if snapshot.offset_of(sensor).is_some() {
            snapshot.set_status(sensor, CalibrationStatus::Succeeded);
            vec![Event::Completed { sensor }]
        } else {
            Vec::new()
        }
    } else if !succeeded {
        snapshot.set_status(sensor, CalibrationStatus::Failed);
        vec![Event::Failed {
            sensor,
            reason: "job failed".to_string(),
        }]
    } else {
        Vec::new()
    }
}

pub(crate) fn ack_observed(
    snapshot: &mut Snapshot,
    jobs: &mut Jobs,
    sensor: Sensor,
    ack: String,
) -> Vec<Event> {
    snapshot.last_ack = Some(ack.clone());
    if snapshot.status_of(sensor) == CalibrationStatus::Cancelled {
        return Vec::new();
    }
    let ack_uppercase = ack.to_ascii_uppercase();
    if ack_uppercase.contains("IN_PROGRESS") {
        return vec![Event::Progress {
            sensor,
            detail: ack,
        }];
    }
    let succeeded = ack_uppercase.contains("ACCEPTED") || ack_uppercase == "OK";
    if let Some(job_id) = running_named(jobs, Action::AwaitCommandAck, sensor) {
        let _ = jobs.complete(job_id, succeeded); // already-terminal complete is a no-op
    }
    if succeeded {
        vec![Event::Progress {
            sensor,
            detail: ack,
        }]
    } else {
        snapshot.set_status(sensor, CalibrationStatus::Failed);
        vec![Event::Failed {
            sensor,
            reason: ack,
        }]
    }
}

pub(crate) fn offsets_read(
    snapshot: &mut Snapshot,
    jobs: &mut Jobs,
    sensor: Sensor,
    value: i32,
) -> Vec<Event> {
    snapshot.set_offset(sensor, value);
    if snapshot.status_of(sensor) == CalibrationStatus::Cancelled {
        return Vec::new();
    }
    if let Some(job_id) = running_named(jobs, Action::ReadOffsets, sensor) {
        let _ = jobs.complete(job_id, true); // already-terminal complete is a no-op
        snapshot.set_status(sensor, CalibrationStatus::Succeeded);
        vec![Event::Completed { sensor }]
    } else {
        Vec::new()
    }
}

pub(crate) fn publish_effects(snapshot: &Snapshot, events: &[Event]) -> Vec<Effect<IoRequest>> {
    let mut effects = vec![Effect::Publish {
        key: CALIBRATION_STATE_SNAPSHOT.to_string(),
        payload: snapshot_bytes(snapshot),
    }];
    for event in events {
        effects.push(Effect::Publish {
            key: CALIBRATION_STREAM_EVENTS.to_string(),
            payload: event_bytes(event),
        });
    }
    effects
}

fn snapshot_bytes(snapshot: &Snapshot) -> Vec<u8> {
    format!(
        "gyro={:?};baro={:?};last_ack={};gyro_offset={};ground_pressure={}",
        snapshot.gyro,
        snapshot.baro,
        snapshot.last_ack.as_deref().unwrap_or(""),
        optional_i32(snapshot.gyro_offset),
        optional_i32(snapshot.ground_pressure),
    )
    .into_bytes()
}

fn event_bytes(event: &Event) -> Vec<u8> {
    format!("{event:?}").into_bytes()
}

fn optional_i32(value: Option<i32>) -> String {
    value.map(|number| number.to_string()).unwrap_or_default()
}

fn parse_i32(payload: &[u8]) -> Option<i32> {
    core::str::from_utf8(payload).ok()?.trim().parse().ok()
}
