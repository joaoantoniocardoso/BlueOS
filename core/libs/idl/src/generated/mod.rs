#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "generated from ROS .msg sources"
)]
pub mod msg;
use crate::message::Message;
/// ROS 2 `.msg` text of any IDL message, keyed by its `SCHEMA_NAME`.
pub fn schema(schema_name: &str) -> Option<&'static str> {
    match schema_name {
        "blueos_example_msgs/action/SetLevel_Feedback" => {
            Some(msg::blueos_example_msgs::SetLevelFeedback::SCHEMA)
        }
        "blueos_example_msgs/action/SetLevel_Goal" => {
            Some(msg::blueos_example_msgs::SetLevelGoal::SCHEMA)
        }
        "blueos_example_msgs/action/SetLevel_Result" => {
            Some(msg::blueos_example_msgs::SetLevelResult::SCHEMA)
        }
        "blueos_example_msgs/msg/PumpState" => Some(msg::blueos_example_msgs::PumpState::SCHEMA),
        "blueos_example_msgs/msg/SelfTestCompleted" => {
            Some(msg::blueos_example_msgs::SelfTestCompleted::SCHEMA)
        }
        "blueos_example_msgs/srv/Level_Request" => {
            Some(msg::blueos_example_msgs::LevelRequest::SCHEMA)
        }
        "blueos_example_msgs/srv/Level_Response" => {
            Some(msg::blueos_example_msgs::LevelResponse::SCHEMA)
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
        "blueos_recorder_msgs/action/DeleteRecording_Feedback" => {
            Some(msg::blueos_recorder_msgs::DeleteRecordingFeedback::SCHEMA)
        }
        "blueos_recorder_msgs/action/DeleteRecording_Goal" => {
            Some(msg::blueos_recorder_msgs::DeleteRecordingGoal::SCHEMA)
        }
        "blueos_recorder_msgs/action/DeleteRecording_Result" => {
            Some(msg::blueos_recorder_msgs::DeleteRecordingResult::SCHEMA)
        }
        "blueos_recorder_msgs/action/RepairRecording_Feedback" => {
            Some(msg::blueos_recorder_msgs::RepairRecordingFeedback::SCHEMA)
        }
        "blueos_recorder_msgs/action/RepairRecording_Goal" => {
            Some(msg::blueos_recorder_msgs::RepairRecordingGoal::SCHEMA)
        }
        "blueos_recorder_msgs/action/RepairRecording_Result" => {
            Some(msg::blueos_recorder_msgs::RepairRecordingResult::SCHEMA)
        }
        "blueos_recorder_msgs/action/SnapshotRecording_Feedback" => {
            Some(msg::blueos_recorder_msgs::SnapshotRecordingFeedback::SCHEMA)
        }
        "blueos_recorder_msgs/action/SnapshotRecording_Goal" => {
            Some(msg::blueos_recorder_msgs::SnapshotRecordingGoal::SCHEMA)
        }
        "blueos_recorder_msgs/action/SnapshotRecording_Result" => {
            Some(msg::blueos_recorder_msgs::SnapshotRecordingResult::SCHEMA)
        }
        "blueos_recorder_msgs/action/StartRecording_Feedback" => {
            Some(msg::blueos_recorder_msgs::StartRecordingFeedback::SCHEMA)
        }
        "blueos_recorder_msgs/action/StartRecording_Goal" => {
            Some(msg::blueos_recorder_msgs::StartRecordingGoal::SCHEMA)
        }
        "blueos_recorder_msgs/action/StartRecording_Result" => {
            Some(msg::blueos_recorder_msgs::StartRecordingResult::SCHEMA)
        }
        "blueos_recorder_msgs/action/StopRecording_Feedback" => {
            Some(msg::blueos_recorder_msgs::StopRecordingFeedback::SCHEMA)
        }
        "blueos_recorder_msgs/action/StopRecording_Goal" => {
            Some(msg::blueos_recorder_msgs::StopRecordingGoal::SCHEMA)
        }
        "blueos_recorder_msgs/action/StopRecording_Result" => {
            Some(msg::blueos_recorder_msgs::StopRecordingResult::SCHEMA)
        }
        "blueos_recorder_msgs/msg/ChannelMessageCount" => {
            Some(msg::blueos_recorder_msgs::ChannelMessageCount::SCHEMA)
        }
        "blueos_recorder_msgs/msg/ChunkIndexEntry" => {
            Some(msg::blueos_recorder_msgs::ChunkIndexEntry::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingFile" => {
            Some(msg::blueos_recorder_msgs::RecordingFile::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingLibrary" => {
            Some(msg::blueos_recorder_msgs::RecordingLibrary::SCHEMA)
        }
        "blueos_recorder_msgs/msg/RecordingState" => {
            Some(msg::blueos_recorder_msgs::RecordingState::SCHEMA)
        }
        "blueos_recorder_msgs/srv/RecordingIndex_Request" => {
            Some(msg::blueos_recorder_msgs::RecordingIndexRequest::SCHEMA)
        }
        "blueos_recorder_msgs/srv/RecordingIndex_Response" => {
            Some(msg::blueos_recorder_msgs::RecordingIndexResponse::SCHEMA)
        }
        "builtin_interfaces/msg/Duration" => Some(msg::builtin_interfaces::Duration::SCHEMA),
        "builtin_interfaces/msg/Time" => Some(msg::builtin_interfaces::Time::SCHEMA),
        "foxglove_msgs/msg/Log" => Some(msg::foxglove_msgs::Log::SCHEMA),
        "std_msgs/msg/Header" => Some(msg::std_msgs::Header::SCHEMA),
        "blueos_example_msgs/action/SetLevel" => Some(
            "# blueos_example_msgs/action/SetLevel\n# The Job type on blueos/v1/example/command/SetLevel: the pump moves to a level one step at a time.\n\n# Goal: the level to reach, at most PumpState.max_level.\nuint8 level\n---\n# Job result: the level the pump reached.\nuint8 level\n---\n# Feedback: the level the pump is at.\nuint8 level",
        ),
        "blueos_example_msgs/srv/Level" => Some(
            "# blueos_example_msgs/srv/Level\n# The Query on blueos/v1/example/query/Level. Its request is empty.\n---\nuint8 level\nuint8 max_level",
        ),
        "blueos_recorder_msgs/action/DeleteRecording" => Some(
            "# blueos_recorder_msgs/action/DeleteRecording\n# The Job type on blueos/v1/recorder/command/DeleteRecording. Rejected while the recording is being written or\n# repaired.\n\nstring path\n---\n---",
        ),
        "blueos_recorder_msgs/action/RepairRecording" => Some(
            "# blueos_recorder_msgs/action/RepairRecording\n# The Job type on blueos/v1/recorder/command/RepairRecording: rewrites a STATE_NEEDS_REPAIR recording so it has a\n# summary again. CancelJob stops it and leaves the recording untouched.\n\nstring path\n---\n# Job result: the recording the Job repaired. Why it failed is the reason of the Job.\nstring path\n---\n# Feedback: how far the repair has read into the recording.\nuint64 bytes_processed\nuint64 total_bytes",
        ),
        "blueos_recorder_msgs/action/SnapshotRecording" => Some(
            "# blueos_recorder_msgs/action/SnapshotRecording\n# The Job type on blueos/v1/recorder/command/SnapshotRecording: writes an indexed copy of a recording (typically the\n# one being written) next to it, named <stem>.snapshot-<UTC ISO time>Z.mcap.\n\nstring path\n---\n# Job result: the recording the Job copied, and the snapshot, which exists when the Job succeeded.\nstring path\nstring output_path\n---\n# Feedback: the snapshot the Job is writing.\nstring output_path",
        ),
        "blueos_recorder_msgs/action/StartRecording" => Some(
            "# blueos_recorder_msgs/action/StartRecording\n# The Job type on blueos/v1/recorder/command/Start: opens a new MCAP session (rotate if one is already active).\n\nbool rotate_if_active\n---\n---",
        ),
        "blueos_recorder_msgs/action/StopRecording" => Some(
            "# blueos_recorder_msgs/action/StopRecording\n# The Job type on blueos/v1/recorder/command/Stop: finishes the current MCAP session; samples are dropped until\n# Start.\n---\n---",
        ),
        "blueos_recorder_msgs/srv/RecordingIndex" => Some(
            "# blueos_recorder_msgs/srv/RecordingIndex\n# The Query on blueos/v1/recorder/query/index. Its response is one page of a walk over record headers, so the\n# browser can fetch chunk bodies with HTTP ranges even when the file has no summary yet (still recording, needs\n# repair).\n\nstring path\n# 0 starts at the file magic; otherwise the `offset` of a previous response.\nuint64 from_offset\n# Maximum chunks in the response (1..=20000).\nuint32 limit\n---\nuint64 size\n# Where the next page starts.\nuint64 offset\n# A DataEnd or Footer record was reached: the walk is complete.\nbool closed\nblueos_recorder_msgs/ChunkIndexEntry[] chunks\nblueos_recorder_msgs/ChannelMessageCount[] message_counts\n# Raw Header, Schema, Channel and Metadata records met in this page, in file order.\nuint8[] records\n================================================================================\nMSG: blueos_recorder_msgs/ChannelMessageCount\n# blueos_recorder_msgs/msg/ChannelMessageCount\n\nuint16 channel_id\nuint64 count\n================================================================================\nMSG: blueos_recorder_msgs/ChunkIndexEntry\n# blueos_recorder_msgs/msg/ChunkIndexEntry\n\nuint64 start_time\nuint64 end_time\n# Offset and length of the whole Chunk record, header included.\nuint64 offset\nuint64 length\nstring compression\nuint64 compressed_size\nuint64 uncompressed_size\nuint16[] channel_ids\n# Bytes of the MessageIndex records that follow the chunk.\nuint64 message_index_length",
        ),
        _ => None,
    }
}
