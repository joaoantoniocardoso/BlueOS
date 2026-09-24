use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::{
    CommandAck, JobList, JobStatus, RestartRequired, ServiceInfo, ServiceStatus, SettingField,
    SettingsEnvelope,
};
use blueos_idl::msg::blueos_recorder_msgs::{RecordingPolicy, RecordingState};
use blueos_idl::msg::foxglove_msgs::Log;

pub fn embedded_ros_schema(schema_name: &str) -> Option<String> {
    [
        (CommandAck::SCHEMA_NAME, CommandAck::SCHEMA),
        (JobList::SCHEMA_NAME, JobList::SCHEMA),
        (JobStatus::SCHEMA_NAME, JobStatus::SCHEMA),
        (RestartRequired::SCHEMA_NAME, RestartRequired::SCHEMA),
        (ServiceInfo::SCHEMA_NAME, ServiceInfo::SCHEMA),
        (ServiceStatus::SCHEMA_NAME, ServiceStatus::SCHEMA),
        (SettingField::SCHEMA_NAME, SettingField::SCHEMA),
        (SettingsEnvelope::SCHEMA_NAME, SettingsEnvelope::SCHEMA),
        (RecordingPolicy::SCHEMA_NAME, RecordingPolicy::SCHEMA),
        (RecordingState::SCHEMA_NAME, RecordingState::SCHEMA),
        (Log::SCHEMA_NAME, Log::SCHEMA),
    ]
    .iter()
    .find(|(name, _)| *name == schema_name)
    .map(|(_, schema)| (*schema).to_string())
}
