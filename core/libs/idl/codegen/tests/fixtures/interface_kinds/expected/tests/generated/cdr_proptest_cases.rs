// @generated
proptest! {
    #[test]
    fn fixture_msgs_progress_round_trips(message in fixture_msgs_progress_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::Progress::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_drain_feedback_round_trips(message in fixture_msgs_drain_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::DrainFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_drain_goal_round_trips(message in fixture_msgs_drain_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::DrainGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_drain_result_round_trips(message in fixture_msgs_drain_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::DrainResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_fill_feedback_round_trips(message in fixture_msgs_fill_feedback_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::FillFeedback::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_fill_goal_round_trips(message in fixture_msgs_fill_goal_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::FillGoal::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_fill_result_round_trips(message in fixture_msgs_fill_result_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::FillResult::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_samples_round_trips(message in fixture_msgs_samples_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::Samples::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_measure_request_round_trips(message in fixture_msgs_measure_request_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::MeasureRequest::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }

    #[test]
    fn fixture_msgs_measure_response_round_trips(message in fixture_msgs_measure_response_strategy()) {
        let encoded = message.encode().expect("encode");
        let decoded = blueos_idl::msg::fixture_msgs::MeasureResponse::decode(&encoded).expect("decode");
        prop_assert_eq!(message, decoded);
    }
}
