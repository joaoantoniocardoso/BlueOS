use blueos_cqrs::App;
use blueos_jobs::JobId;
use blueos_service::IoPlan;
use calibration_sensors::{
    AUTOPILOT_RPC_PREFLIGHT, Action, Calibration, Command, IoRequest, Sensor,
};

pub(crate) fn execute_io(app: &App<Calibration>, io_request: IoRequest) -> IoPlan<Command> {
    match io_request {
        IoRequest::PreflightCalibration { job_id, sensor } => {
            let payload = match sensor {
                Sensor::Gyro => b"gyro".to_vec(),
                Sensor::Baro => b"baro".to_vec(),
            };
            IoPlan::Rpc {
                key: AUTOPILOT_RPC_PREFLIGHT.to_string(),
                payload,
                then: Box::new(move |reply| {
                    Some(Command::JobProgress {
                        job_id,
                        payload: reply,
                    })
                }),
            }
        }
        IoRequest::ReadOffsets { job_id, sensor } => {
            let name = job_spec_of(app, job_id).map(|(name, _)| name);
            if name.as_deref() == Some(Action::AwaitCommandAck.name()) {
                return IoPlan::Local(None);
            }
            let ready = match sensor {
                Sensor::Gyro => app.snapshot.gyro_offset.is_some(),
                Sensor::Baro => app.snapshot.ground_pressure.is_some(),
            };
            if ready {
                IoPlan::Local(Some(Command::JobProgress {
                    job_id,
                    payload: Vec::new(),
                }))
            } else {
                IoPlan::Local(None)
            }
        }
        IoRequest::CancelAutopilot { job_id } => {
            let payload = job_spec_of(app, job_id)
                .map(|(_, payload)| payload)
                .filter(|payload| !payload.is_empty())
                .unwrap_or_else(|| b"cancel".to_vec());
            IoPlan::Rpc {
                key: AUTOPILOT_RPC_PREFLIGHT.to_string(),
                payload,
                then: Box::new(move |_| {
                    Some(Command::JobProgress {
                        job_id,
                        payload: Vec::new(),
                    })
                }),
            }
        }
    }
}

fn job_spec_of(app: &App<Calibration>, job_id: JobId) -> Option<(String, Vec<u8>)> {
    app.jobs
        .snapshot()
        .jobs
        .into_iter()
        .find(|job| job.job_id == job_id)
        .and_then(|job| {
            job.job_spec
                .map(|job_spec| (job_spec.name, job_spec.payload))
        })
}
