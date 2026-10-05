use blueos_idl::{
    Message,
    msg::{
        blueos_example_msgs::LevelRequest,
        blueos_msgs::{CommandAck, ServiceInfo, ServiceMetrics, SettingField},
        blueos_recorder_msgs::{RecordingFile, RecordingLibrary},
        foxglove_msgs::Log,
    },
};

use super::fixture::encode_hex;

pub(crate) fn encode_json_message(
    schema_name: &str,
    message: &serde_json::Value,
) -> Option<String> {
    let encoded = match schema_name {
        LevelRequest::SCHEMA_NAME => {
            let message: LevelRequest =
                serde_json::from_value(message.clone()).expect("LevelRequest from json");
            message.encode().ok()
        }
        Log::SCHEMA_NAME => {
            let message: Log = serde_json::from_value(message.clone()).expect("Log from json");
            message.encode().ok()
        }
        CommandAck::SCHEMA_NAME => {
            let message: CommandAck =
                serde_json::from_value(message.clone()).expect("CommandAck from json");
            message.encode().ok()
        }
        ServiceInfo::SCHEMA_NAME => {
            let message: ServiceInfo =
                serde_json::from_value(message.clone()).expect("ServiceInfo from json");
            message.encode().ok()
        }
        ServiceMetrics::SCHEMA_NAME => {
            let message: ServiceMetrics =
                serde_json::from_value(message.clone()).expect("ServiceMetrics from json");
            message.encode().ok()
        }
        RecordingFile::SCHEMA_NAME => {
            let message: RecordingFile =
                serde_json::from_value(message.clone()).expect("RecordingFile from json");
            message.encode().ok()
        }
        RecordingLibrary::SCHEMA_NAME => {
            let message: RecordingLibrary =
                serde_json::from_value(message.clone()).expect("RecordingLibrary from json");
            message.encode().ok()
        }
        SettingField::SCHEMA_NAME => {
            let message: SettingField =
                serde_json::from_value(message.clone()).expect("SettingField from json");
            message.encode().ok()
        }
        _ => None,
    };
    encoded.map(|payload| encode_hex(&payload))
}
