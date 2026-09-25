mod settings_schema;

use blueos_api::cdr_encoding;
use blueos_cli::{Argv, Common};
use blueos_comms::Payload;
use blueos_cqrs::App;
use blueos_example_pump::PumpIoRequest;
use blueos_example_pump::{
    ExampleSettings, PumpCommand, PumpDomain, PumpEvent, PumpQuery, PumpSnapshot, job_spec_name,
};
use blueos_example_simulated_pump::{SimulatedPumpStep, run_step};
use blueos_idl::Message;
use blueos_idl::msg::blueos_example_msgs::{
    EmptyRequest, LevelQueryResponse, PumpState, SelfTestCompleted, SetLevelRequest,
};
use blueos_idl::msg::blueos_msgs::{
    JobList, JobStatus, ServiceInfo, ServiceStatus, constants_job_status as job_status_constants,
    constants_service_status as service_status_constants,
};
use blueos_jobs::JobStatus as InternalJobStatus;
use blueos_logging::error;
use blueos_service::ServiceBuilder;
use bytes::Bytes;
use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;
use tracing::info;

use settings_schema::{ExampleSettingsSchema, settings_from_envelope_json};

const SERVICE_NAME: &str = "example";

#[derive(Parser, Debug)]
#[command(name = "example", about = "BlueOS teaching example service (D-20)")]
struct Cli {
    #[command(flatten)]
    common: Common,
}

pub fn run(arguments: impl IntoIterator<Item = OsString>) {
    let cli = Cli::parse_from(arguments);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    if let Err(run_error) = runtime.block_on(run_async(cli)) {
        error!("example: {run_error}");
    }
}

async fn run_async(cli: Cli) -> Result<(), String> {
    let config_folder = cli.common.config.clone();
    let settings = load_initial_settings(&config_folder)?;
    let application = App::new(PumpSnapshot {
        settings,
        ..PumpSnapshot::default()
    });
    let service_info = ServiceInfo {
        name: SERVICE_NAME.into(),
        version: env!("CARGO_PKG_VERSION").into(),
        build: String::new(),
        capabilities: vec!["pump".into(), "self-test".into()],
    };
    let mut builder = ServiceBuilder::<PumpDomain>::new(SERVICE_NAME)
        .app(application)
        .service_info(service_info)
        .verbosity(cli.common.verbose)
        .cli(Argv::default())
        .status(service_status)
        .jobs(job_list)
        .command("SetLevel", decode_set_level)
        .command(
            "StartSelfTest",
            decode_empty_command(PumpCommand::StartSelfTest),
        )
        .command(
            "CancelSelfTest",
            decode_empty_command(PumpCommand::CancelSelfTest),
        )
        .query("Level", |_, application| {
            let view = application.query(PumpQuery::Level);
            let message = LevelQueryResponse {
                level: view.level,
                max_level: view.max_level,
            };
            encode_message(&message)
        })
        .state("pump", pump_state, encode_pump_state)
        .event(
            "SelfTestCompleted",
            |event| matches!(event, PumpEvent::SelfTestCompleted { .. }),
            |event| {
                let PumpEvent::SelfTestCompleted { passed, detail } = event else {
                    return Err("event filter mismatch".into());
                };
                encode_message(&SelfTestCompleted {
                    passed: *passed,
                    detail: detail.clone(),
                })
            },
        )
        .event(
            "RestartRequired",
            |event| matches!(event, PumpEvent::RestartRequired { .. }),
            |event| {
                let PumpEvent::RestartRequired { fields } = event else {
                    return Err("event filter mismatch".into());
                };
                encode_message(&blueos_idl::msg::blueos_msgs::RestartRequired {
                    fields: fields.clone(),
                })
            },
        )
        .io(|_application, request| async move { Ok(io_to_command(request).await) });

    builder = builder
        .settings(
            config_folder,
            |envelope| {
                Ok(PumpCommand::UpdateSettings(settings_from_envelope_json(
                    &envelope.document_json,
                )?))
            },
            |application| ExampleSettingsSchema::from(application.snapshot.settings.clone()),
        )
        .map_err(|error| error.to_string())?;

    info!("example service starting");
    builder.run().await.map_err(|error| error.to_string())?;
    Ok(())
}

fn load_initial_settings(config_folder: &Option<PathBuf>) -> Result<ExampleSettings, String> {
    if config_folder.is_none() {
        return Ok(ExampleSettings::with_defaults());
    }
    let manager = blueos_settings::SettingsManager::<ExampleSettingsSchema>::new(
        SERVICE_NAME,
        config_folder.clone(),
    )
    .map_err(|error| error.to_string())?;
    Ok(manager.settings().inner().clone())
}

fn decode_empty_command(command: PumpCommand) -> impl Fn(&[u8]) -> Result<PumpCommand, String> {
    move |payload| {
        if payload.is_empty() {
            return Ok(command.clone());
        }
        EmptyRequest::decode(payload).map_err(|error| error.to_string())?;
        Ok(command.clone())
    }
}

fn decode_set_level(payload: &[u8]) -> Result<PumpCommand, String> {
    let request = SetLevelRequest::decode(payload).map_err(|error| error.to_string())?;
    Ok(PumpCommand::SetLevel {
        level: request.level,
    })
}

fn service_status(application: &App<PumpDomain>) -> ServiceStatus {
    let detail = if application.snapshot.self_test_active {
        "self-test running"
    } else {
        "ready"
    };
    ServiceStatus {
        status: service_status_constants::STATUS_READY,
        detail: detail.into(),
    }
}

fn pump_state(application: &App<PumpDomain>) -> PumpState {
    PumpState {
        level: application.snapshot.level,
        max_level: application.snapshot.effective_max_level(),
        self_test_phase: application.snapshot.self_test_phase,
        self_test_active: application.snapshot.self_test_active,
    }
}

fn job_list(application: &App<PumpDomain>) -> JobList {
    let snapshot = application.jobs.snapshot();
    JobList {
        jobs: snapshot
            .jobs
            .into_iter()
            .map(|job| {
                let name = job.job_spec.as_ref().map(job_spec_name).unwrap_or_default();
                JobStatus {
                    job_id: job.job_id.0,
                    parent_job_id: job.parent.map(|parent| parent.0).unwrap_or(0),
                    status: internal_status_to_idl(job.status),
                    name,
                }
            })
            .collect(),
    }
}

fn internal_status_to_idl(status: InternalJobStatus) -> u8 {
    match status {
        InternalJobStatus::Queued => job_status_constants::STATUS_QUEUED,
        InternalJobStatus::Running => job_status_constants::STATUS_RUNNING,
        InternalJobStatus::Cancelling => job_status_constants::STATUS_CANCELLING,
        InternalJobStatus::Succeeded => job_status_constants::STATUS_SUCCEEDED,
        InternalJobStatus::Failed => job_status_constants::STATUS_FAILED,
        InternalJobStatus::Cancelled => job_status_constants::STATUS_CANCELLED,
    }
}

fn encode_pump_state(state: PumpState) -> Result<(Payload, String), String> {
    encode_message(&state)
}

async fn io_to_command(request: PumpIoRequest) -> PumpCommand {
    let PumpIoRequest::RunStep { job_id, step } = request;
    let simulated_step = match step {
        blueos_example_pump::PumpJobSpec::VerifyOff => SimulatedPumpStep::VerifyOff,
        blueos_example_pump::PumpJobSpec::RampUp => SimulatedPumpStep::RampUp,
        blueos_example_pump::PumpJobSpec::VerifyOn => SimulatedPumpStep::VerifyOn,
    };
    let succeeded = run_step(simulated_step).await;
    PumpCommand::JobIoFinished { job_id, succeeded }
}

fn encode_message<MessageType: Message>(
    message: &MessageType,
) -> Result<(Payload, String), String> {
    let bytes = message.encode().map_err(|error| error.to_string())?;
    Ok((
        Payload::from_bytes(Bytes::from(bytes)),
        cdr_encoding(MessageType::SCHEMA_NAME),
    ))
}
