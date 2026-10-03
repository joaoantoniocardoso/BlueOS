//! MAVLink ingress Task: vehicle armed state and camera protocol facts.

use core::time::Duration;

use tokio::time::{MissedTickBehavior, interval};
use tracing::warn;

use blueos_comms::Sample;
use blueos_domain::Command;
use blueos_recorder_cameras::{
    Cameras, CamerasObservedFact, CaptureCommandKind, RAW_MAVLINK_OUT_TOPIC,
};
use blueos_recorder_capture::CaptureObservedFact;
use blueos_recorder_domain::{RecorderDomain, RecorderObservedFact};
use blueos_recorder_mavlink::{
    MavlinkCaptureCommand, MavlinkFact, MavlinkIngressState, facts_from_frame, last_vehicle_armed,
};
use blueos_service::{CommandSender, TaskContext, TaskFailed};

use crate::context::RecorderContext;

const ARMED_RESEND_INTERVAL: Duration = Duration::from_secs(1);

/// Runs until shutdown, parsing `mavlink_raw/out` and re-sending armed facts periodically.
pub(crate) async fn run_mavlink_ingress(
    task_context: TaskContext<RecorderDomain, RecorderContext>,
) -> Result<(), TaskFailed> {
    let mut ingress_state = MavlinkIngressState::default();
    let mut subscriber = task_context
        .session
        .subscribe(RAW_MAVLINK_OUT_TOPIC)
        .await
        .map_err(|error| {
            warn!(%error, topic = RAW_MAVLINK_OUT_TOPIC, "Failed to subscribe for MAVLink ingress");
            TaskFailed
        })?;
    let mut armed_timer = interval(ARMED_RESEND_INTERVAL);
    armed_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            () = task_context.shutdown.cancelled() => break,
            _ = armed_timer.tick() => {
                if let Some(armed) = last_vehicle_armed(&ingress_state) {
                    send_armed_fact(&task_context.commands, armed).await?;
                }
            }
            sample = subscriber.recv() => {
                let Some(sample) = sample else {
                    break;
                };
                handle_sample(&sample, &mut ingress_state, &task_context.commands).await?;
            }
        }
    }
    Ok(())
}

async fn handle_sample(
    sample: &Sample,
    ingress_state: &mut MavlinkIngressState,
    commands: &CommandSender<RecorderDomain>,
) -> Result<(), TaskFailed> {
    let payload = sample.payload().to_bytes();
    let facts = facts_from_frame(ingress_state, payload.as_ref());
    for fact in facts {
        deliver_fact(commands, fact).await?;
    }
    Ok(())
}

async fn deliver_fact(
    commands: &CommandSender<RecorderDomain>,
    fact: MavlinkFact,
) -> Result<(), TaskFailed> {
    match fact {
        MavlinkFact::ArmedChanged(armed) => send_armed_fact(commands, armed).await,
        MavlinkFact::CameraHeartbeat(camera) => {
            send_observed(
                commands,
                RecorderObservedFact::Cameras(CamerasObservedFact::CameraHeartbeat {
                    camera: to_cameras_system(camera),
                }),
            )
            .await
        }
        MavlinkFact::CameraRecordingCapability {
            camera,
            capture_video,
        } => {
            send_observed(
                commands,
                RecorderObservedFact::Cameras(CamerasObservedFact::SetCameraRecordingCapability {
                    camera: to_cameras_system(camera),
                    capture_video,
                }),
            )
            .await
        }
        MavlinkFact::VideoStreamRegistered { topic, camera } => {
            send_observed(
                commands,
                RecorderObservedFact::Cameras(CamerasObservedFact::RegisterVideoStream {
                    topic,
                    camera: to_cameras_system(camera),
                }),
            )
            .await
        }
        MavlinkFact::CameraCaptureCommand {
            command,
            target_system,
            target_component,
            status_interval_hertz,
        } => {
            send_observed(
                commands,
                RecorderObservedFact::Cameras(CamerasObservedFact::CameraCaptureCommand {
                    command: to_capture_command_kind(command),
                    target_system,
                    target_component,
                    status_interval: Cameras::capture_status_interval_from_rate(
                        status_interval_hertz,
                    ),
                }),
            )
            .await
        }
    }
}

async fn send_armed_fact(
    commands: &CommandSender<RecorderDomain>,
    armed: bool,
) -> Result<(), TaskFailed> {
    send_observed(
        commands,
        RecorderObservedFact::Capture(CaptureObservedFact::ArmedChanged(armed)),
    )
    .await
}

async fn send_observed(
    commands: &CommandSender<RecorderDomain>,
    fact: RecorderObservedFact,
) -> Result<(), TaskFailed> {
    commands
        .send(Command::ObservedFact(fact))
        .await
        .map_err(|error| {
            warn!(%error, "Failed to deliver observed fact to the Inbox");
            TaskFailed
        })
}

fn to_cameras_system(
    camera: blueos_recorder_mavlink::SystemAndComponent,
) -> blueos_recorder_cameras::SystemAndComponent {
    blueos_recorder_cameras::SystemAndComponent {
        system_id: camera.system_id,
        component_id: camera.component_id,
    }
}

fn to_capture_command_kind(command: MavlinkCaptureCommand) -> CaptureCommandKind {
    match command {
        MavlinkCaptureCommand::StartCapture => CaptureCommandKind::StartCapture,
        MavlinkCaptureCommand::StopCapture => CaptureCommandKind::StopCapture,
        MavlinkCaptureCommand::RequestCaptureStatus => CaptureCommandKind::RequestCaptureStatus,
    }
}
