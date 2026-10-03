#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "generated from ROS .msg sources"
)]
pub mod msg;
use crate::message::Message;
/// ROS 2 `.msg` text of any IDL message, keyed by its `SCHEMA_NAME`.
pub fn schema(schema_name: &str) -> Option<&'static str> {
    match schema_name {
        "blueos_example_msgs/msg/EmptyRequest" => {
            Some(msg::blueos_example_msgs::EmptyRequest::SCHEMA)
        }
        "blueos_example_msgs/msg/LevelQueryResponse" => {
            Some(msg::blueos_example_msgs::LevelQueryResponse::SCHEMA)
        }
        "blueos_example_msgs/msg/PumpState" => Some(msg::blueos_example_msgs::PumpState::SCHEMA),
        "blueos_example_msgs/msg/SelfTestCompleted" => {
            Some(msg::blueos_example_msgs::SelfTestCompleted::SCHEMA)
        }
        "blueos_example_msgs/msg/SetLevelRequest" => {
            Some(msg::blueos_example_msgs::SetLevelRequest::SCHEMA)
        }
        "blueos_msgs/msg/CommandAck" => Some(msg::blueos_msgs::CommandAck::SCHEMA),
        "blueos_msgs/msg/EndpointInfo" => Some(msg::blueos_msgs::EndpointInfo::SCHEMA),
        "blueos_msgs/msg/JobFeedback" => Some(msg::blueos_msgs::JobFeedback::SCHEMA),
        "blueos_msgs/msg/JobFeedbackList" => Some(msg::blueos_msgs::JobFeedbackList::SCHEMA),
        "blueos_msgs/msg/JobList" => Some(msg::blueos_msgs::JobList::SCHEMA),
        "blueos_msgs/msg/JobResult" => Some(msg::blueos_msgs::JobResult::SCHEMA),
        "blueos_msgs/msg/JobStatus" => Some(msg::blueos_msgs::JobStatus::SCHEMA),
        "blueos_msgs/msg/PermissionAnswer" => Some(msg::blueos_msgs::PermissionAnswer::SCHEMA),
        "blueos_msgs/msg/RestartRequired" => Some(msg::blueos_msgs::RestartRequired::SCHEMA),
        "blueos_msgs/msg/ServiceInfo" => Some(msg::blueos_msgs::ServiceInfo::SCHEMA),
        "blueos_msgs/msg/ServiceStatus" => Some(msg::blueos_msgs::ServiceStatus::SCHEMA),
        "blueos_msgs/msg/SettingField" => Some(msg::blueos_msgs::SettingField::SCHEMA),
        "blueos_msgs/msg/SettingsEnvelope" => Some(msg::blueos_msgs::SettingsEnvelope::SCHEMA),
        "blueos_recorder_msgs/msg/ChannelMessageCount" => {
            Some(msg::blueos_recorder_msgs::ChannelMessageCount::SCHEMA)
        }
        "blueos_recorder_msgs/msg/ChunkIndexEntry" => {
            Some(msg::blueos_recorder_msgs::ChunkIndexEntry::SCHEMA)
        }
        "blueos_recorder_msgs/msg/DeleteRecordingCommand" => {
            Some(msg::blueos_recorder_msgs::DeleteRecordingCommand::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingFile" => {
            Some(msg::blueos_recorder_msgs::RecordingFile::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingIndex" => {
            Some(msg::blueos_recorder_msgs::RecordingIndex::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingIndexRequest" => {
            Some(msg::blueos_recorder_msgs::RecordingIndexRequest::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingLibrary" => {
            Some(msg::blueos_recorder_msgs::RecordingLibrary::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingOperation" => {
            Some(msg::blueos_recorder_msgs::RecordingOperation::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingPolicy" => {
            Some(msg::blueos_recorder_msgs::RecordingPolicy::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingState" => {
            Some(msg::blueos_recorder_msgs::RecordingState::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RepairRecordingCommand" => {
            Some(msg::blueos_recorder_msgs::RepairRecordingCommand::SCHEMA)
        }
        "blueos_recorder_msgs/msg/SetPolicyCommand" => {
            Some(msg::blueos_recorder_msgs::SetPolicyCommand::SCHEMA)
        }
        "blueos_recorder_msgs/msg/SnapshotRecordingCommand" => {
            Some(msg::blueos_recorder_msgs::SnapshotRecordingCommand::SCHEMA)
        }
        "blueos_recorder_msgs/msg/StartRecordingCommand" => {
            Some(msg::blueos_recorder_msgs::StartRecordingCommand::SCHEMA)
        }
        "blueos_recorder_msgs/msg/StopRecordingCommand" => {
            Some(msg::blueos_recorder_msgs::StopRecordingCommand::SCHEMA)
        }
        "builtin_interfaces/msg/Duration" => Some(msg::builtin_interfaces::Duration::SCHEMA),
        "builtin_interfaces/msg/Time" => Some(msg::builtin_interfaces::Time::SCHEMA),
        "foxglove_msgs/msg/Log" => Some(msg::foxglove_msgs::Log::SCHEMA),
        "std_msgs/msg/Header" => Some(msg::std_msgs::Header::SCHEMA),
        _ => None,
    }
}
