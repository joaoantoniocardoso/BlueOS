use super::*;
use crate::jobs::{leaf, running_named};
use blueos_cqrs::App;
use blueos_jobs::{JobGraph, JobStatus};

fn first_io_request(effects: &[Effect<IoRequest>]) -> &IoRequest {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::Io(io_request) => Some(io_request),
            _ => None,
        })
        .expect("io request")
}

fn running_leaf_names(app: &App<Calibration>) -> Vec<String> {
    app.jobs
        .snapshot()
        .jobs
        .into_iter()
        .filter(|job| job.status == JobStatus::Running && job.job_spec.is_some())
        .map(|job| job.job_spec.unwrap().name)
        .collect()
}

fn job_names(app: &App<Calibration>) -> Vec<String> {
    app.jobs
        .snapshot()
        .jobs
        .into_iter()
        .filter_map(|job| job.job_spec.map(|job_spec| job_spec.name))
        .collect()
}

#[test]
fn gyro_and_baro_share_start_preflight_calibration() {
    let mut gyro = App::<Calibration>::new(Snapshot::default());
    let (events, effects) = gyro.handle(Command::StartGyro);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Requested {
            use_case: UseCase::CalibrateGyroscope
        }
    )));
    assert!(matches!(
        first_io_request(&effects),
        IoRequest::PreflightCalibration {
            sensor: Sensor::Gyro,
            ..
        }
    ));
    assert!(
        job_names(&gyro)
            .iter()
            .any(|name| name == Action::StartPreflightCalibration.name())
    );

    let mut baro = App::<Calibration>::new(Snapshot::default());
    let (_, effects) = baro.handle(Command::StartBaro);
    assert!(matches!(
        first_io_request(&effects),
        IoRequest::PreflightCalibration {
            sensor: Sensor::Baro,
            ..
        }
    ));
    let gyro_leaf = leaf(Action::StartPreflightCalibration, Sensor::Gyro);
    let baro_leaf = leaf(Action::StartPreflightCalibration, Sensor::Baro);
    match (gyro_leaf, baro_leaf) {
        (JobGraph::Leaf(gyro_spec), JobGraph::Leaf(baro_spec)) => {
            assert_eq!(gyro_spec.name, baro_spec.name);
            assert_eq!(gyro_spec.name, Action::StartPreflightCalibration.name());
            assert_ne!(gyro_spec.payload, baro_spec.payload);
        }
        _ => panic!("expected leaves"),
    }
}

#[test]
fn stationary_sequences_gyro_then_baro() {
    let mut app = App::<Calibration>::new(Snapshot::default());
    app.handle(Command::StartStationary);
    assert_eq!(running_leaf_names(&app), ["start_preflight_calibration"]);
    let start = running_named(&app.jobs, Action::StartPreflightCalibration, Sensor::Gyro).unwrap();
    app.handle(Command::JobProgress {
        job_id: start,
        payload: b"ok".to_vec(),
    });
    assert_eq!(running_leaf_names(&app), ["await_command_ack"]);
    app.handle(Command::AckObserved {
        sensor: Sensor::Gyro,
        ack: "ACCEPTED".to_string(),
    });
    assert_eq!(running_leaf_names(&app), ["read_offsets"]);
    app.handle(Command::OffsetsRead {
        sensor: Sensor::Gyro,
        value: 3,
    });
    assert_eq!(app.snapshot.gyro, CalibrationStatus::Succeeded);
    assert_eq!(running_leaf_names(&app), ["start_preflight_calibration"]);
    let start = running_named(&app.jobs, Action::StartPreflightCalibration, Sensor::Baro).unwrap();
    app.handle(Command::JobProgress {
        job_id: start,
        payload: b"ok".to_vec(),
    });
    app.handle(Command::AckObserved {
        sensor: Sensor::Baro,
        ack: "ACCEPTED".to_string(),
    });
    app.handle(Command::OffsetsRead {
        sensor: Sensor::Baro,
        value: 1013,
    });
    assert_eq!(app.snapshot.gyro, CalibrationStatus::Succeeded);
    assert_eq!(app.snapshot.baro, CalibrationStatus::Succeeded);
    assert_eq!(app.snapshot.gyro_offset, Some(3));
    assert_eq!(app.snapshot.ground_pressure, Some(1013));
    let view = app.query(Query::GetSnapshot);
    assert_eq!(view.snapshot.gyro, CalibrationStatus::Succeeded);
    assert_eq!(view.snapshot.baro, CalibrationStatus::Succeeded);
}

#[test]
fn cancel_marks_jobs_and_snapshot() {
    let mut app = App::<Calibration>::new(Snapshot::default());
    app.handle(Command::StartGyro);
    assert_eq!(app.snapshot.gyro, CalibrationStatus::Running);
    let (events, effects) = app.handle(Command::Cancel);
    assert_eq!(app.snapshot.gyro, CalibrationStatus::Cancelled);
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Cancelled {
            sensor: Sensor::Gyro
        }
    )));
    assert!(matches!(
        first_io_request(&effects),
        IoRequest::CancelAutopilot { .. }
    ));
    assert!(
        app.jobs
            .snapshot()
            .jobs
            .iter()
            .any(|job| { matches!(job.status, JobStatus::Cancelled | JobStatus::Cancelling) })
    );
}

#[test]
fn query_reads_snapshot_only() {
    let mut app = App::<Calibration>::new(Snapshot::default());
    app.handle(Command::StartGyro);
    let before = app.snapshot.clone();
    let jobs_before = app.jobs.snapshot();
    let view = app.query(Query::GetSnapshot);
    assert_eq!(view.snapshot.gyro, before.gyro);
    assert_eq!(app.snapshot.gyro, before.gyro);
    assert_eq!(app.jobs.snapshot(), jobs_before);
    assert_eq!(app.snapshot.gyro, CalibrationStatus::Running);
}

#[test]
fn io_from_job_maps_action_names() {
    let start = JobSpec {
        name: Action::StartPreflightCalibration.name().to_string(),
        payload: b"baro".to_vec(),
    };
    assert!(matches!(
        Calibration::io_from_job(JobId(1), &start),
        IoRequest::PreflightCalibration {
            job_id: JobId(1),
            sensor: Sensor::Baro
        }
    ));
    let await_ack = JobSpec {
        name: Action::AwaitCommandAck.name().to_string(),
        payload: b"gyro".to_vec(),
    };
    assert!(matches!(
        Calibration::io_from_job(JobId(2), &await_ack),
        IoRequest::ReadOffsets {
            job_id: JobId(2),
            sensor: Sensor::Gyro
        }
    ));
    let cancel = JobSpec {
        name: Action::CancelCalibration.name().to_string(),
        payload: Vec::new(),
    };
    assert!(matches!(
        Calibration::io_from_job(JobId(3), &cancel),
        IoRequest::CancelAutopilot { job_id: JobId(3) }
    ));
}
