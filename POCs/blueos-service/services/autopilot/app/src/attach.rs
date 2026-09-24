use autopilot_preflight::{Command, IoRequest, RPC_PREFLIGHT, Sensor};
use blueos_jobs::{JobId, Jobs};
use blueos_service::{IoPlan, Service};

pub(crate) fn attach(service: &mut Service<autopilot_preflight::Autopilot>) {
    service.on_rpc(RPC_PREFLIGHT, |payload| {
        parse_sensor(payload).map(|sensor| Command::Preflight {
            correlation: 0,
            sensor,
        })
    });
    service.on_io(|app, io_request| {
        let IoRequest::Tick { job_id } = io_request;
        let sensor = sensor_from_job(&app.jobs, job_id).unwrap_or(Sensor::Gyro);
        IoPlan::Local(Some(Command::AdvanceAck { job_id, sensor }))
    });
}

fn sensor_from_job(jobs: &Jobs, job_id: JobId) -> Option<Sensor> {
    jobs.snapshot()
        .jobs
        .into_iter()
        .find(|job| job.job_id == job_id)
        .and_then(|job| job.job_spec)
        .and_then(|job_spec| parse_sensor(&job_spec.payload))
}

fn parse_sensor(bytes: &[u8]) -> Option<Sensor> {
    match bytes {
        b"gyro" => Some(Sensor::Gyro),
        b"baro" => Some(Sensor::Baro),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autopilot_preflight::{Autopilot, Snapshot};
    use blueos_cli::Argv;
    use blueos_comms::{ChannelDriver, Session};
    use blueos_configs::Configs;
    use blueos_cqrs::App;
    use blueos_service::{Adapters, Service};
    use std::thread;

    fn adapters(session: Session) -> Adapters {
        Adapters {
            comms: session,
            configs: Configs::default(),
            cli: Argv::default(),
        }
    }

    #[test]
    fn rpc_preflight_via_channel_driver_without_zenoh() {
        let (local_driver, peer_driver) = ChannelDriver::pair();
        let mut service: Service<Autopilot> = Service::new(
            App::new(Snapshot::default()),
            adapters(Session::with_driver(Box::new(local_driver))),
        );
        attach(&mut service);
        let client = Session::with_driver(Box::new(peer_driver));
        let rpc = client.rpc_client(RPC_PREFLIGHT);
        let join = thread::spawn(move || {
            let run_result = service.run();
            (run_result, service)
        });
        let reply = rpc.call(b"gyro").unwrap();
        assert_eq!(reply, b"IN_PROGRESS");
        let reply = rpc.call(b"baro").unwrap();
        assert_eq!(reply, b"IN_PROGRESS");
        drop(rpc);
        drop(client);
        let (run_result, service) = join.join().expect("server");
        run_result.unwrap();
        assert_eq!(service.app.snapshot.ack.as_deref(), Some("ACCEPTED"));
        assert_eq!(service.app.snapshot.gyro_offset, Some(1));
        assert_eq!(service.app.snapshot.ground_pressure, Some(1013));
    }

    #[test]
    fn rpc_moving_failed_via_channel_driver_without_zenoh() {
        let (local_driver, peer_driver) = ChannelDriver::pair();
        let mut service: Service<Autopilot> = Service::new(
            App::new(Snapshot {
                moving: true,
                ..Snapshot::default()
            }),
            adapters(Session::with_driver(Box::new(local_driver))),
        );
        attach(&mut service);
        let client = Session::with_driver(Box::new(peer_driver));
        let rpc = client.rpc_client(RPC_PREFLIGHT);
        let join = thread::spawn(move || {
            let run_result = service.run();
            (run_result, service)
        });
        let reply = rpc.call(b"gyro").unwrap();
        assert_eq!(reply, b"FAILED");
        drop(rpc);
        drop(client);
        let (run_result, service) = join.join().expect("server");
        run_result.unwrap();
        assert_eq!(service.app.snapshot.gyro_offset, None);
    }
}
