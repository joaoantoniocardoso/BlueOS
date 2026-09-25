use std::time::Duration;

use blueos_api::{cdr_encoding, command_key, event_key, jobs_key};
use blueos_comms::{ChannelBackend, Payload, Session};
use blueos_cqrs::App;
use blueos_example_pump::{
    PumpCommand, PumpDomain, PumpEvent, PumpIoRequest, PumpJobSpec, PumpSnapshot, job_spec_name,
};
use blueos_example_simulated_pump::{SimulatedPumpStep, run_step};
use blueos_idl::Message;
use blueos_idl::msg::blueos_example_msgs::{PumpState, SelfTestCompleted};
use blueos_idl::msg::blueos_msgs::{CommandAck, JobList};
use blueos_service::ServiceBuilder;
use bytes::Bytes;
use futures::StreamExt;
use tokio::time;

async fn spawn_example_service() -> (
    Session,
    tokio::task::JoinHandle<Result<(), blueos_service::ServiceError>>,
) {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);

    let handle = tokio::spawn(async move {
        ServiceBuilder::<PumpDomain>::new("example")
            .app(App::new(PumpSnapshot::default()))
            .service_info(blueos_idl::msg::blueos_msgs::ServiceInfo {
                name: "example".into(),
                version: "0.0.0".into(),
                build: String::new(),
                capabilities: Vec::new(),
            })
            .jobs(|application| {
                let snapshot = application.jobs.snapshot();
                blueos_idl::msg::blueos_msgs::JobList {
                    jobs: snapshot
                        .jobs
                        .into_iter()
                        .filter_map(|job| {
                            job.job_spec.as_ref().map(|spec| {
                                blueos_idl::msg::blueos_msgs::JobStatus {
                                    job_id: job.job_id.0,
                                    parent_job_id: job.parent.map(|parent| parent.0).unwrap_or(0),
                                    status: job.status as u8,
                                    name: job_spec_name(spec),
                                }
                            })
                        })
                        .collect(),
                }
            })
            .command("StartSelfTest", |_| Ok(PumpCommand::StartSelfTest))
            .state(
                "pump",
                |application| blueos_idl::msg::blueos_example_msgs::PumpState {
                    level: application.snapshot.level,
                    max_level: application.snapshot.effective_max_level(),
                    self_test_phase: application.snapshot.self_test_phase,
                    self_test_active: application.snapshot.self_test_active,
                },
                |state| {
                    let bytes = state.encode().map_err(|error| error.to_string())?;
                    Ok((
                        Payload::from_bytes(Bytes::from(bytes)),
                        cdr_encoding(PumpState::SCHEMA_NAME),
                    ))
                },
            )
            .event(
                "SelfTestCompleted",
                |event| matches!(event, PumpEvent::SelfTestCompleted { .. }),
                |event| {
                    let PumpEvent::SelfTestCompleted { passed, detail } = event else {
                        return Err("filter".into());
                    };
                    let message = SelfTestCompleted {
                        passed: *passed,
                        detail: detail.clone(),
                    };
                    let bytes = message.encode().map_err(|error| error.to_string())?;
                    Ok((
                        Payload::from_bytes(Bytes::from(bytes)),
                        cdr_encoding(SelfTestCompleted::SCHEMA_NAME),
                    ))
                },
            )
            .io(|_application, request| async move {
                let PumpIoRequest::RunStep { job_id, step } = request;
                let simulated = match step {
                    PumpJobSpec::VerifyOff => SimulatedPumpStep::VerifyOff,
                    PumpJobSpec::RampUp => SimulatedPumpStep::RampUp,
                    PumpJobSpec::VerifyOn => SimulatedPumpStep::VerifyOn,
                };
                let succeeded = run_step(simulated).await;
                Ok(PumpCommand::JobIoFinished { job_id, succeeded })
            })
            .run_with_session(service_session)
            .await
    });

    time::sleep(Duration::from_millis(30)).await;
    (client_session, handle)
}

async fn command_ack(session: &Session, command: &str) -> CommandAck {
    let reply = session
        .query(
            &command_key("example", command),
            Payload::empty(),
            "",
            Duration::from_secs(1),
        )
        .await
        .expect("command");
    CommandAck::decode(reply.payload.as_slice().as_slice()).expect("ack")
}

#[tokio::test]
async fn start_self_test_completes_and_emits_event() {
    let (client, service_handle) = spawn_example_service().await;
    let mut events = client
        .subscribe(&event_key("example", "SelfTestCompleted"))
        .await
        .expect("subscribe");

    let ack = command_ack(&client, "StartSelfTest").await;
    assert!(ack.accepted);

    for _ in 0..100 {
        let reply = client
            .query(
                &jobs_key("example"),
                Payload::empty(),
                "",
                Duration::from_secs(1),
            )
            .await
            .expect("jobs");
        let jobs = JobList::decode(reply.payload.as_slice().as_slice()).expect("job list");
        if jobs.jobs.iter().any(|job| {
            job.name == "verify_on"
                && job.status
                    == blueos_idl::msg::blueos_msgs::constants_job_status::STATUS_SUCCEEDED
        }) {
            break;
        }
        time::sleep(Duration::from_millis(10)).await;
    }

    let sample = time::timeout(Duration::from_secs(2), events.next())
        .await
        .expect("timeout")
        .expect("event");
    let completed =
        SelfTestCompleted::decode(sample.payload.as_slice().as_slice()).expect("event body");
    assert!(completed.passed);

    service_handle.abort();
}
