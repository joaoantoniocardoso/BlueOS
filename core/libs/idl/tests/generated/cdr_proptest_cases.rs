// @generated
proptest! {
    #[test]
    fn blueos_example_msgs_set_level_feedback_round_trips(message in blueos_example_msgs_set_level_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_example_msgs::SetLevelFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_example_msgs_set_level_goal_round_trips(message in blueos_example_msgs_set_level_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_example_msgs::SetLevelGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_example_msgs_set_level_result_round_trips(message in blueos_example_msgs_set_level_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_example_msgs::SetLevelResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_example_msgs_pump_state_round_trips(message in blueos_example_msgs_pump_state_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_example_msgs::PumpState::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_example_msgs_self_test_completed_round_trips(message in blueos_example_msgs_self_test_completed_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_example_msgs::SelfTestCompleted::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_example_msgs_level_request_round_trips(message in blueos_example_msgs_level_request_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_example_msgs::LevelRequest::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_example_msgs_level_response_round_trips(message in blueos_example_msgs_level_response_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_example_msgs::LevelResponse::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_update_settings_feedback_round_trips(message in blueos_msgs_update_settings_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::UpdateSettingsFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_setting_field_round_trips(message in blueos_msgs_setting_field_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::SettingField::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_settings_envelope_round_trips(message in blueos_msgs_settings_envelope_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::SettingsEnvelope::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_update_settings_goal_round_trips(message in blueos_msgs_update_settings_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::UpdateSettingsGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_update_settings_result_round_trips(message in blueos_msgs_update_settings_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::UpdateSettingsResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_command_ack_round_trips(message in blueos_msgs_command_ack_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::CommandAck::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_endpoint_info_round_trips(message in blueos_msgs_endpoint_info_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::EndpointInfo::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_job_feedback_round_trips(message in blueos_msgs_job_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::JobFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_job_feedback_list_round_trips(message in blueos_msgs_job_feedback_list_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::JobFeedbackList::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_job_status_round_trips(message in blueos_msgs_job_status_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::JobStatus::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_job_list_round_trips(message in blueos_msgs_job_list_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::JobList::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_job_result_round_trips(message in blueos_msgs_job_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::JobResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_metric_label_round_trips(message in blueos_msgs_metric_label_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::MetricLabel::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_metric_counter_round_trips(message in blueos_msgs_metric_counter_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::MetricCounter::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_metric_gauge_round_trips(message in blueos_msgs_metric_gauge_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::MetricGauge::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_metric_histogram_round_trips(message in blueos_msgs_metric_histogram_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::MetricHistogram::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_permission_answer_round_trips(message in blueos_msgs_permission_answer_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::PermissionAnswer::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_restart_required_round_trips(message in blueos_msgs_restart_required_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::RestartRequired::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_service_info_round_trips(message in blueos_msgs_service_info_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::ServiceInfo::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_service_metrics_round_trips(message in blueos_msgs_service_metrics_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::ServiceMetrics::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_msgs_service_status_round_trips(message in blueos_msgs_service_status_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_msgs::ServiceStatus::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_delete_recording_feedback_round_trips(message in blueos_recorder_msgs_delete_recording_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_delete_recording_goal_round_trips(message in blueos_recorder_msgs_delete_recording_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_delete_recording_result_round_trips(message in blueos_recorder_msgs_delete_recording_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::DeleteRecordingResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_repair_recording_feedback_round_trips(message in blueos_recorder_msgs_repair_recording_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RepairRecordingFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_repair_recording_goal_round_trips(message in blueos_recorder_msgs_repair_recording_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RepairRecordingGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_repair_recording_result_round_trips(message in blueos_recorder_msgs_repair_recording_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RepairRecordingResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_snapshot_recording_feedback_round_trips(message in blueos_recorder_msgs_snapshot_recording_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_snapshot_recording_goal_round_trips(message in blueos_recorder_msgs_snapshot_recording_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_snapshot_recording_result_round_trips(message in blueos_recorder_msgs_snapshot_recording_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::SnapshotRecordingResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_start_recording_feedback_round_trips(message in blueos_recorder_msgs_start_recording_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::StartRecordingFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_start_recording_goal_round_trips(message in blueos_recorder_msgs_start_recording_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::StartRecordingGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_start_recording_result_round_trips(message in blueos_recorder_msgs_start_recording_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::StartRecordingResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_stop_recording_feedback_round_trips(message in blueos_recorder_msgs_stop_recording_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::StopRecordingFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_stop_recording_goal_round_trips(message in blueos_recorder_msgs_stop_recording_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::StopRecordingGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_stop_recording_result_round_trips(message in blueos_recorder_msgs_stop_recording_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::StopRecordingResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_channel_message_count_round_trips(message in blueos_recorder_msgs_channel_message_count_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::ChannelMessageCount::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_chunk_index_entry_round_trips(message in blueos_recorder_msgs_chunk_index_entry_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::ChunkIndexEntry::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn builtin_interfaces_duration_round_trips(message in builtin_interfaces_duration_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::builtin_interfaces::Duration::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_recording_contents_round_trips(message in blueos_recorder_msgs_recording_contents_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingContents::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn builtin_interfaces_time_round_trips(message in builtin_interfaces_time_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::builtin_interfaces::Time::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_recording_file_round_trips(message in blueos_recorder_msgs_recording_file_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingFile::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_recording_library_round_trips(message in blueos_recorder_msgs_recording_library_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingLibrary::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_recording_state_round_trips(message in blueos_recorder_msgs_recording_state_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingState::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_recording_bytes_request_round_trips(message in blueos_recorder_msgs_recording_bytes_request_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingBytesRequest::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_recording_bytes_response_round_trips(message in blueos_recorder_msgs_recording_bytes_response_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingBytesResponse::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_recording_index_request_round_trips(message in blueos_recorder_msgs_recording_index_request_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingIndexRequest::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn blueos_recorder_msgs_recording_index_response_round_trips(message in blueos_recorder_msgs_recording_index_response_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingIndexResponse::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn foxglove_msgs_log_round_trips(message in foxglove_msgs_log_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::foxglove_msgs::Log::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn std_msgs_empty_round_trips(message in std_msgs_empty_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::std_msgs::Empty::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn std_msgs_header_round_trips(message in std_msgs_header_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::std_msgs::Header::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }
}
