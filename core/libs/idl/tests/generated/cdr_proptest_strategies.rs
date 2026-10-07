// @generated
use proptest::{collection::vec, prelude::*, prop_compose};

use blueos_idl::Message;

use super::cdr_proptest_helpers::{bounded_bytes, bounded_string, bounded_vec};

prop_compose! {
    pub(crate) fn blueos_example_msgs_set_level_feedback_strategy()(level in any::<u8>()) -> blueos_idl::msg::blueos_example_msgs::SetLevelFeedback {
        blueos_idl::msg::blueos_example_msgs::SetLevelFeedback { level }
    }
}

prop_compose! {
    pub(crate) fn blueos_example_msgs_set_level_goal_strategy()(level in any::<u8>()) -> blueos_idl::msg::blueos_example_msgs::SetLevelGoal {
        blueos_idl::msg::blueos_example_msgs::SetLevelGoal { level }
    }
}

prop_compose! {
    pub(crate) fn blueos_example_msgs_set_level_result_strategy()(level in any::<u8>()) -> blueos_idl::msg::blueos_example_msgs::SetLevelResult {
        blueos_idl::msg::blueos_example_msgs::SetLevelResult { level }
    }
}

prop_compose! {
    pub(crate) fn blueos_example_msgs_pump_state_strategy()(level in any::<u8>(), max_level in any::<u8>(), self_test_phase in any::<u8>().prop_map(blueos_idl::msg::blueos_example_msgs::PumpStateSelfTestPhase::from_raw), self_test_active in any::<bool>()) -> blueos_idl::msg::blueos_example_msgs::PumpState {
        blueos_idl::msg::blueos_example_msgs::PumpState { level, max_level, self_test_phase, self_test_active }
    }
}

prop_compose! {
    pub(crate) fn blueos_example_msgs_self_test_completed_strategy()(passed in any::<bool>(), detail in bounded_string(), checks in vec(any::<bool>(), 3).prop_map(|values| {
            let mut array = [Default::default(); 3];
            for (index, value) in values.into_iter().enumerate() {
                array[index] = value;
            }
            array
        })) -> blueos_idl::msg::blueos_example_msgs::SelfTestCompleted {
        blueos_idl::msg::blueos_example_msgs::SelfTestCompleted { passed, detail, checks }
    }
}

pub(crate) fn blueos_example_msgs_level_request_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_example_msgs::LevelRequest> {
    Just(blueos_idl::msg::blueos_example_msgs::LevelRequest::default())
}

prop_compose! {
    pub(crate) fn blueos_example_msgs_level_response_strategy()(level in any::<u8>(), max_level in any::<u8>()) -> blueos_idl::msg::blueos_example_msgs::LevelResponse {
        blueos_idl::msg::blueos_example_msgs::LevelResponse { level, max_level }
    }
}

pub(crate) fn blueos_msgs_update_settings_feedback_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_msgs::UpdateSettingsFeedback> {
    Just(blueos_idl::msg::blueos_msgs::UpdateSettingsFeedback::default())
}

prop_compose! {
    pub(crate) fn blueos_msgs_setting_field_strategy()(path in bounded_string(), restart_required in any::<bool>()) -> blueos_idl::msg::blueos_msgs::SettingField {
        blueos_idl::msg::blueos_msgs::SettingField { path, restart_required }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_settings_envelope_strategy()(document_json in bounded_string(), fields in bounded_vec(blueos_msgs_setting_field_strategy())) -> blueos_idl::msg::blueos_msgs::SettingsEnvelope {
        blueos_idl::msg::blueos_msgs::SettingsEnvelope { document_json, fields }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_update_settings_goal_strategy()(envelope in blueos_msgs_settings_envelope_strategy()) -> blueos_idl::msg::blueos_msgs::UpdateSettingsGoal {
        blueos_idl::msg::blueos_msgs::UpdateSettingsGoal { envelope }
    }
}

pub(crate) fn blueos_msgs_update_settings_result_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_msgs::UpdateSettingsResult> {
    Just(blueos_idl::msg::blueos_msgs::UpdateSettingsResult::default())
}

prop_compose! {
    pub(crate) fn blueos_msgs_command_ack_strategy()(accepted in any::<bool>(), job_id in bounded_string(), status in any::<u8>().prop_map(blueos_idl::msg::blueos_msgs::CommandAckStatus::from_raw), reason in bounded_string()) -> blueos_idl::msg::blueos_msgs::CommandAck {
        blueos_idl::msg::blueos_msgs::CommandAck { accepted, job_id, status, reason }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_endpoint_info_strategy()(kind in bounded_string(), name in bounded_string(), key in bounded_string(), interface_type in bounded_string(), schema in bounded_string()) -> blueos_idl::msg::blueos_msgs::EndpointInfo {
        blueos_idl::msg::blueos_msgs::EndpointInfo { kind, name, key, interface_type, schema }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_job_feedback_strategy()(job_id in bounded_string(), feedback in bounded_bytes()) -> blueos_idl::msg::blueos_msgs::JobFeedback {
        blueos_idl::msg::blueos_msgs::JobFeedback { job_id, feedback }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_job_feedback_list_strategy()(jobs in bounded_vec(blueos_msgs_job_feedback_strategy())) -> blueos_idl::msg::blueos_msgs::JobFeedbackList {
        blueos_idl::msg::blueos_msgs::JobFeedbackList { jobs }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_job_status_strategy()(job_id in bounded_string(), job_type in bounded_string(), status in any::<u8>().prop_map(blueos_idl::msg::blueos_msgs::JobStatusStatus::from_raw), reason in bounded_string()) -> blueos_idl::msg::blueos_msgs::JobStatus {
        blueos_idl::msg::blueos_msgs::JobStatus { job_id, job_type, status, reason }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_job_list_strategy()(jobs in bounded_vec(blueos_msgs_job_status_strategy())) -> blueos_idl::msg::blueos_msgs::JobList {
        blueos_idl::msg::blueos_msgs::JobList { jobs }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_job_result_strategy()(job in blueos_msgs_job_status_strategy(), result in bounded_bytes()) -> blueos_idl::msg::blueos_msgs::JobResult {
        blueos_idl::msg::blueos_msgs::JobResult { job, result }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_metric_label_strategy()(name in bounded_string(), value in bounded_string()) -> blueos_idl::msg::blueos_msgs::MetricLabel {
        blueos_idl::msg::blueos_msgs::MetricLabel { name, value }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_metric_counter_strategy()(name in bounded_string(), labels in bounded_vec(blueos_msgs_metric_label_strategy()), value in any::<u64>()) -> blueos_idl::msg::blueos_msgs::MetricCounter {
        blueos_idl::msg::blueos_msgs::MetricCounter { name, labels, value }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_metric_gauge_strategy()(name in bounded_string(), labels in bounded_vec(blueos_msgs_metric_label_strategy()), value in any::<f64>()) -> blueos_idl::msg::blueos_msgs::MetricGauge {
        blueos_idl::msg::blueos_msgs::MetricGauge { name, labels, value }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_metric_histogram_strategy()(name in bounded_string(), labels in bounded_vec(blueos_msgs_metric_label_strategy()), count in any::<u64>(), sum in any::<f64>(), bucket_bounds in bounded_vec(any::<f64>()), bucket_counts in bounded_vec(any::<u64>())) -> blueos_idl::msg::blueos_msgs::MetricHistogram {
        blueos_idl::msg::blueos_msgs::MetricHistogram { name, labels, count, sum, bucket_bounds, bucket_counts }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_permission_answer_strategy()(granted in any::<bool>()) -> blueos_idl::msg::blueos_msgs::PermissionAnswer {
        blueos_idl::msg::blueos_msgs::PermissionAnswer { granted }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_restart_required_strategy()(fields in bounded_vec(bounded_string())) -> blueos_idl::msg::blueos_msgs::RestartRequired {
        blueos_idl::msg::blueos_msgs::RestartRequired { fields }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_service_info_strategy()(name in bounded_string(), version in bounded_string(), build in bounded_string(), capabilities in bounded_vec(bounded_string()), endpoints in bounded_vec(blueos_msgs_endpoint_info_strategy())) -> blueos_idl::msg::blueos_msgs::ServiceInfo {
        blueos_idl::msg::blueos_msgs::ServiceInfo { name, version, build, capabilities, endpoints }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_service_metrics_strategy()(counters in bounded_vec(blueos_msgs_metric_counter_strategy()), gauges in bounded_vec(blueos_msgs_metric_gauge_strategy()), histograms in bounded_vec(blueos_msgs_metric_histogram_strategy())) -> blueos_idl::msg::blueos_msgs::ServiceMetrics {
        blueos_idl::msg::blueos_msgs::ServiceMetrics { counters, gauges, histograms }
    }
}

prop_compose! {
    pub(crate) fn blueos_msgs_service_status_strategy()(status in any::<u8>().prop_map(blueos_idl::msg::blueos_msgs::ServiceStatusStatus::from_raw), detail in bounded_string()) -> blueos_idl::msg::blueos_msgs::ServiceStatus {
        blueos_idl::msg::blueos_msgs::ServiceStatus { status, detail }
    }
}

pub(crate) fn blueos_recorder_msgs_delete_recording_feedback_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingFeedback> {
    Just(blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingFeedback::default())
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_delete_recording_goal_strategy()(path in bounded_string()) -> blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingGoal {
        blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingGoal { path }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_delete_recording_result_strategy()(path in bounded_string()) -> blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingResult {
        blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingResult { path }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_repair_recording_feedback_strategy()(bytes_processed in any::<u64>(), total_bytes in any::<u64>()) -> blueos_idl::msg::blueos_recorder_msgs::RepairRecordingFeedback {
        blueos_idl::msg::blueos_recorder_msgs::RepairRecordingFeedback { bytes_processed, total_bytes }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_repair_recording_goal_strategy()(path in bounded_string()) -> blueos_idl::msg::blueos_recorder_msgs::RepairRecordingGoal {
        blueos_idl::msg::blueos_recorder_msgs::RepairRecordingGoal { path }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_repair_recording_result_strategy()(path in bounded_string()) -> blueos_idl::msg::blueos_recorder_msgs::RepairRecordingResult {
        blueos_idl::msg::blueos_recorder_msgs::RepairRecordingResult { path }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_snapshot_recording_feedback_strategy()(output_path in bounded_string()) -> blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingFeedback {
        blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingFeedback { output_path }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_snapshot_recording_goal_strategy()(path in bounded_string()) -> blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingGoal {
        blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingGoal { path }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_snapshot_recording_result_strategy()(path in bounded_string(), output_path in bounded_string()) -> blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingResult {
        blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingResult { path, output_path }
    }
}

pub(crate) fn blueos_recorder_msgs_start_recording_feedback_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_recorder_msgs::StartRecordingFeedback> {
    Just(blueos_idl::msg::blueos_recorder_msgs::StartRecordingFeedback::default())
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_start_recording_goal_strategy()(rotate_if_active in any::<bool>()) -> blueos_idl::msg::blueos_recorder_msgs::StartRecordingGoal {
        blueos_idl::msg::blueos_recorder_msgs::StartRecordingGoal { rotate_if_active }
    }
}

pub(crate) fn blueos_recorder_msgs_start_recording_result_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_recorder_msgs::StartRecordingResult> {
    Just(blueos_idl::msg::blueos_recorder_msgs::StartRecordingResult::default())
}

pub(crate) fn blueos_recorder_msgs_stop_recording_feedback_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_recorder_msgs::StopRecordingFeedback> {
    Just(blueos_idl::msg::blueos_recorder_msgs::StopRecordingFeedback::default())
}

pub(crate) fn blueos_recorder_msgs_stop_recording_goal_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_recorder_msgs::StopRecordingGoal> {
    Just(blueos_idl::msg::blueos_recorder_msgs::StopRecordingGoal::default())
}

pub(crate) fn blueos_recorder_msgs_stop_recording_result_strategy() -> impl Strategy<Value = blueos_idl::msg::blueos_recorder_msgs::StopRecordingResult> {
    Just(blueos_idl::msg::blueos_recorder_msgs::StopRecordingResult::default())
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_channel_message_count_strategy()(channel_id in any::<u16>(), count in any::<u64>()) -> blueos_idl::msg::blueos_recorder_msgs::ChannelMessageCount {
        blueos_idl::msg::blueos_recorder_msgs::ChannelMessageCount { channel_id, count }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_chunk_index_entry_strategy()(start_time in any::<u64>(), end_time in any::<u64>(), offset in any::<u64>(), length in any::<u64>(), compression in bounded_string(), compressed_size in any::<u64>(), uncompressed_size in any::<u64>(), channel_ids in bounded_vec(any::<u16>()), message_index_length in any::<u64>()) -> blueos_idl::msg::blueos_recorder_msgs::ChunkIndexEntry {
        blueos_idl::msg::blueos_recorder_msgs::ChunkIndexEntry { start_time, end_time, offset, length, compression, compressed_size, uncompressed_size, channel_ids, message_index_length }
    }
}

prop_compose! {
    pub(crate) fn builtin_interfaces_duration_strategy()(sec in any::<i32>(), nanosec in any::<u32>()) -> blueos_idl::msg::builtin_interfaces::Duration {
        blueos_idl::msg::builtin_interfaces::Duration { sec, nanosec }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_recording_contents_strategy()(path in bounded_string(), duration in builtin_interfaces_duration_strategy(), video_topics in bounded_vec(bounded_string()), other_topic_count in any::<u32>()) -> blueos_idl::msg::blueos_recorder_msgs::RecordingContents {
        blueos_idl::msg::blueos_recorder_msgs::RecordingContents { path, duration, video_topics, other_topic_count }
    }
}

prop_compose! {
    pub(crate) fn builtin_interfaces_time_strategy()(sec in any::<i32>(), nanosec in any::<u32>()) -> blueos_idl::msg::builtin_interfaces::Time {
        blueos_idl::msg::builtin_interfaces::Time { sec, nanosec }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_recording_file_strategy()(path in bounded_string(), name in bounded_string(), size_bytes in any::<u64>(), created in builtin_interfaces_time_strategy(), state in any::<u8>().prop_map(blueos_idl::msg::blueos_recorder_msgs::RecordingFileState::from_raw), repair_bytes_processed in any::<u64>(), repair_total_bytes in any::<u64>(), repair_bytes_per_second in any::<f64>(), repair_error in bounded_string(), allowed_operations in bounded_vec(bounded_string()), repair_job_id in bounded_string()) -> blueos_idl::msg::blueos_recorder_msgs::RecordingFile {
        blueos_idl::msg::blueos_recorder_msgs::RecordingFile { path, name, size_bytes, created, state, repair_bytes_processed, repair_total_bytes, repair_bytes_per_second, repair_error, allowed_operations, repair_job_id }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_recording_library_strategy()(files in bounded_vec(blueos_recorder_msgs_recording_file_strategy()), contents in bounded_vec(blueos_recorder_msgs_recording_contents_strategy())) -> blueos_idl::msg::blueos_recorder_msgs::RecordingLibrary {
        blueos_idl::msg::blueos_recorder_msgs::RecordingLibrary { files, contents }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_recording_state_strategy()(armed in any::<bool>(), session_active in any::<bool>(), current_file in bounded_string(), session_bytes_written in any::<u64>(), recording_video_topics in bounded_vec(bounded_string()), samples_dropped in any::<u64>()) -> blueos_idl::msg::blueos_recorder_msgs::RecordingState {
        blueos_idl::msg::blueos_recorder_msgs::RecordingState { armed, session_active, current_file, session_bytes_written, recording_video_topics, samples_dropped }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_recording_bytes_request_strategy()(path in bounded_string(), offset in any::<u64>(), length in any::<u32>(), from_end in any::<bool>()) -> blueos_idl::msg::blueos_recorder_msgs::RecordingBytesRequest {
        blueos_idl::msg::blueos_recorder_msgs::RecordingBytesRequest { path, offset, length, from_end }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_recording_bytes_response_strategy()(size in any::<u64>(), data in bounded_bytes()) -> blueos_idl::msg::blueos_recorder_msgs::RecordingBytesResponse {
        blueos_idl::msg::blueos_recorder_msgs::RecordingBytesResponse { size, data }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_recording_index_request_strategy()(path in bounded_string(), from_offset in any::<u64>(), limit in any::<u32>()) -> blueos_idl::msg::blueos_recorder_msgs::RecordingIndexRequest {
        blueos_idl::msg::blueos_recorder_msgs::RecordingIndexRequest { path, from_offset, limit }
    }
}

prop_compose! {
    pub(crate) fn blueos_recorder_msgs_recording_index_response_strategy()(size in any::<u64>(), offset in any::<u64>(), closed in any::<bool>(), chunks in bounded_vec(blueos_recorder_msgs_chunk_index_entry_strategy()), message_counts in bounded_vec(blueos_recorder_msgs_channel_message_count_strategy()), records in bounded_bytes()) -> blueos_idl::msg::blueos_recorder_msgs::RecordingIndexResponse {
        blueos_idl::msg::blueos_recorder_msgs::RecordingIndexResponse { size, offset, closed, chunks, message_counts, records }
    }
}

prop_compose! {
    pub(crate) fn foxglove_msgs_log_strategy()(timestamp in builtin_interfaces_time_strategy(), level in any::<u8>(), message in bounded_string(), name in bounded_string(), file in bounded_string(), line in any::<u32>()) -> blueos_idl::msg::foxglove_msgs::Log {
        blueos_idl::msg::foxglove_msgs::Log { timestamp, level, message, name, file, line }
    }
}

pub(crate) fn std_msgs_empty_strategy() -> impl Strategy<Value = blueos_idl::msg::std_msgs::Empty> {
    Just(blueos_idl::msg::std_msgs::Empty::default())
}

prop_compose! {
    pub(crate) fn std_msgs_header_strategy()(stamp in builtin_interfaces_time_strategy(), frame_id in bounded_string()) -> blueos_idl::msg::std_msgs::Header {
        blueos_idl::msg::std_msgs::Header { stamp, frame_id }
    }
}
