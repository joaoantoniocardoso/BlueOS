use blueos_recorder_policy::{
    CaptureCommandKind, RecorderCommand, RecordingOperationKind, ScannedRecording,
    SystemAndComponent as PolicySystem,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InjectedScannedRecording {
    relative_path: String,
    name: String,
    size_bytes: u64,
    modified_unix_seconds: i64,
    indexed: bool,
}

fn to_scanned_recording(recording: InjectedScannedRecording) -> ScannedRecording {
    ScannedRecording {
        relative_path: recording.relative_path,
        name: recording.name,
        size_bytes: recording.size_bytes,
        modified_unix_seconds: recording.modified_unix_seconds,
        indexed: recording.indexed,
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InjectedCommand {
    ArmedChanged {
        armed: bool,
    },
    SetCameraRecordingCapability {
        camera: InjectedSystem,
        capture_video: bool,
    },
    RegisterVideoStream {
        topic: String,
        camera: InjectedSystem,
    },
    CameraCaptureCommand {
        command: String,
        target_system: u8,
        target_component: u8,
        status_interval_hertz: f32,
        now_millis: u64,
    },
    LibraryScanCompleted {
        recordings: Vec<InjectedScannedRecording>,
        now_unix_seconds: i64,
    },
    LibraryRepairProgress {
        path: String,
        bytes_processed: u64,
        total_bytes: u64,
        now_unix_seconds: i64,
    },
    LibraryOperationFinished {
        operation: String,
        path: String,
        output_path: String,
        succeeded: bool,
        cancelled: bool,
        error: String,
    },
    InitializeLibrary,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InjectedSystem {
    pub system_id: u8,
    pub component_id: u8,
}

pub fn decode_injected(payload: &[u8]) -> Result<RecorderCommand, String> {
    let message: InjectedCommand =
        serde_json::from_slice(payload).map_err(|error| error.to_string())?;
    Ok(match message {
        InjectedCommand::ArmedChanged { armed } => RecorderCommand::ArmedChanged(armed),
        InjectedCommand::SetCameraRecordingCapability {
            camera,
            capture_video,
        } => RecorderCommand::SetCameraRecordingCapability {
            camera: to_policy(camera),
            capture_video,
        },
        InjectedCommand::RegisterVideoStream { topic, camera } => {
            RecorderCommand::RegisterVideoStream {
                topic,
                camera: to_policy(camera),
            }
        }
        InjectedCommand::LibraryScanCompleted {
            recordings,
            now_unix_seconds,
        } => RecorderCommand::Library(blueos_recorder_policy::LibraryCommand::ScanCompleted {
            recordings: recordings.into_iter().map(to_scanned_recording).collect(),
            now_unix_seconds,
        }),
        InjectedCommand::LibraryRepairProgress {
            path,
            bytes_processed,
            total_bytes,
            now_unix_seconds,
        } => RecorderCommand::Library(blueos_recorder_policy::LibraryCommand::RepairProgress {
            path,
            bytes_processed,
            total_bytes,
            now_unix_seconds,
        }),
        InjectedCommand::LibraryOperationFinished {
            operation,
            path,
            output_path,
            succeeded,
            cancelled,
            error,
        } => RecorderCommand::Library(blueos_recorder_policy::LibraryCommand::OperationFinished {
            operation: match operation.as_str() {
                "repair" => RecordingOperationKind::Repair,
                "snapshot" => RecordingOperationKind::Snapshot,
                _ => RecordingOperationKind::Delete,
            },
            path,
            output_path,
            succeeded,
            cancelled,
            error,
        }),
        InjectedCommand::InitializeLibrary => RecorderCommand::InitializeLibrary,
        InjectedCommand::CameraCaptureCommand {
            command,
            target_system,
            target_component,
            status_interval_hertz,
            now_millis,
        } => RecorderCommand::CameraCaptureCommand {
            command: match command.as_str() {
                "start_capture" => CaptureCommandKind::StartCapture,
                "stop_capture" => CaptureCommandKind::StopCapture,
                _ => CaptureCommandKind::RequestCaptureStatus,
            },
            target_system,
            target_component,
            status_interval_hertz,
            now_millis,
        },
    })
}

pub fn encode_injected(command: RecorderCommand) -> Result<Vec<u8>, String> {
    let injected = match command {
        RecorderCommand::ArmedChanged(armed) => InjectedCommand::ArmedChanged { armed },
        RecorderCommand::SetCameraRecordingCapability {
            camera,
            capture_video,
        } => InjectedCommand::SetCameraRecordingCapability {
            camera: from_policy(camera),
            capture_video,
        },
        RecorderCommand::RegisterVideoStream { topic, camera } => {
            InjectedCommand::RegisterVideoStream {
                topic,
                camera: from_policy(camera),
            }
        }
        RecorderCommand::Library(command) => match command {
            blueos_recorder_policy::LibraryCommand::ScanCompleted {
                recordings,
                now_unix_seconds,
            } => InjectedCommand::LibraryScanCompleted {
                recordings: recordings
                    .into_iter()
                    .map(|recording| InjectedScannedRecording {
                        relative_path: recording.relative_path,
                        name: recording.name,
                        size_bytes: recording.size_bytes,
                        modified_unix_seconds: recording.modified_unix_seconds,
                        indexed: recording.indexed,
                    })
                    .collect(),
                now_unix_seconds,
            },
            blueos_recorder_policy::LibraryCommand::RepairProgress {
                path,
                bytes_processed,
                total_bytes,
                now_unix_seconds,
            } => InjectedCommand::LibraryRepairProgress {
                path,
                bytes_processed,
                total_bytes,
                now_unix_seconds,
            },
            blueos_recorder_policy::LibraryCommand::OperationFinished {
                operation,
                path,
                output_path,
                succeeded,
                cancelled,
                error,
            } => InjectedCommand::LibraryOperationFinished {
                operation: match operation {
                    RecordingOperationKind::Repair => "repair",
                    RecordingOperationKind::Snapshot => "snapshot",
                    RecordingOperationKind::Delete => "delete",
                }
                .into(),
                path,
                output_path,
                succeeded,
                cancelled,
                error,
            },
            _ => return Err("not an injectable library command".into()),
        },
        RecorderCommand::InitializeLibrary => InjectedCommand::InitializeLibrary,
        RecorderCommand::CameraCaptureCommand {
            command,
            target_system,
            target_component,
            status_interval_hertz,
            now_millis,
        } => InjectedCommand::CameraCaptureCommand {
            command: match command {
                CaptureCommandKind::StartCapture => "start_capture",
                CaptureCommandKind::StopCapture => "stop_capture",
                CaptureCommandKind::RequestCaptureStatus => "request_capture_status",
            }
            .into(),
            target_system,
            target_component,
            status_interval_hertz,
            now_millis,
        },
        _ => return Err("not an injectable command".into()),
    };
    serde_json::to_vec(&injected).map_err(|error| error.to_string())
}

fn to_policy(system: InjectedSystem) -> PolicySystem {
    PolicySystem {
        system_id: system.system_id,
        component_id: system.component_id,
    }
}

fn from_policy(system: PolicySystem) -> InjectedSystem {
    InjectedSystem {
        system_id: system.system_id,
        component_id: system.component_id,
    }
}

#[allow(deprecated)]
pub fn fact_to_injected_command(
    fact: blueos_recorder_mavlink::MavlinkFact,
) -> Option<RecorderCommand> {
    use blueos_recorder_mavlink::MavlinkFact;
    use mavlink::dialects::ardupilotmega::MavCmd;

    let now_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    match fact {
        MavlinkFact::ArmedChanged(armed) => Some(RecorderCommand::ArmedChanged(armed)),
        MavlinkFact::CameraRecordingCapability {
            camera,
            capture_video,
        } => Some(RecorderCommand::SetCameraRecordingCapability {
            camera: PolicySystem {
                system_id: camera.system_id,
                component_id: camera.component_id,
            },
            capture_video,
        }),
        MavlinkFact::VideoStreamRegistered { topic, camera } => {
            Some(RecorderCommand::RegisterVideoStream {
                topic,
                camera: PolicySystem {
                    system_id: camera.system_id,
                    component_id: camera.component_id,
                },
            })
        }
        MavlinkFact::CameraCaptureCommand {
            command,
            target_system,
            target_component,
            status_interval_hertz,
        } => Some(RecorderCommand::CameraCaptureCommand {
            command: match command {
                MavCmd::MAV_CMD_VIDEO_START_CAPTURE => CaptureCommandKind::StartCapture,
                MavCmd::MAV_CMD_VIDEO_STOP_CAPTURE => CaptureCommandKind::StopCapture,
                MavCmd::MAV_CMD_REQUEST_CAMERA_CAPTURE_STATUS => {
                    CaptureCommandKind::RequestCaptureStatus
                }
                _ => CaptureCommandKind::RequestCaptureStatus,
            },
            target_system,
            target_component,
            status_interval_hertz,
            now_millis,
        }),
        MavlinkFact::CameraHeartbeat(_) => None,
    }
}
