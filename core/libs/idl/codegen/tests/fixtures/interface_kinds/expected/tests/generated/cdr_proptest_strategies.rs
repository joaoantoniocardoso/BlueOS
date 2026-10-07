// @generated
use proptest::{collection::vec, prelude::*, prop_compose};

use blueos_idl::Message;

use super::cdr_proptest_helpers::{bounded_bytes, bounded_string, bounded_vec};

prop_compose! {
    pub(crate) fn fixture_msgs_progress_strategy()(done in any::<u64>(), total in any::<u64>()) -> blueos_idl::msg::fixture_msgs::Progress {
        blueos_idl::msg::fixture_msgs::Progress { done, total }
    }
}

prop_compose! {
    pub(crate) fn fixture_msgs_drain_feedback_strategy()(progress in fixture_msgs_progress_strategy()) -> blueos_idl::msg::fixture_msgs::DrainFeedback {
        blueos_idl::msg::fixture_msgs::DrainFeedback { progress }
    }
}

pub(crate) fn fixture_msgs_drain_goal_strategy() -> impl Strategy<Value = blueos_idl::msg::fixture_msgs::DrainGoal> {
    Just(blueos_idl::msg::fixture_msgs::DrainGoal::default())
}

prop_compose! {
    pub(crate) fn fixture_msgs_drain_result_strategy()(drained in any::<f32>()) -> blueos_idl::msg::fixture_msgs::DrainResult {
        blueos_idl::msg::fixture_msgs::DrainResult { drained }
    }
}

prop_compose! {
    pub(crate) fn fixture_msgs_fill_feedback_strategy()(progress in fixture_msgs_progress_strategy()) -> blueos_idl::msg::fixture_msgs::FillFeedback {
        blueos_idl::msg::fixture_msgs::FillFeedback { progress }
    }
}

prop_compose! {
    pub(crate) fn fixture_msgs_fill_goal_strategy()(level in any::<f32>(), rate in any::<f32>()) -> blueos_idl::msg::fixture_msgs::FillGoal {
        blueos_idl::msg::fixture_msgs::FillGoal { level, rate }
    }
}

prop_compose! {
    pub(crate) fn fixture_msgs_fill_result_strategy()(reached in any::<bool>()) -> blueos_idl::msg::fixture_msgs::FillResult {
        blueos_idl::msg::fixture_msgs::FillResult { reached }
    }
}

prop_compose! {
    pub(crate) fn fixture_msgs_samples_strategy()(samples in bounded_bytes(), tag in vec(any::<u8>(), 4).prop_map(|values| {
            let mut array = [Default::default(); 4];
            for (index, value) in values.into_iter().enumerate() {
                array[index] = value;
            }
            array
        }), counts in bounded_vec(any::<u16>())) -> blueos_idl::msg::fixture_msgs::Samples {
        blueos_idl::msg::fixture_msgs::Samples { samples, tag, counts }
    }
}

prop_compose! {
    pub(crate) fn fixture_msgs_measure_request_strategy()(probe in bounded_string()) -> blueos_idl::msg::fixture_msgs::MeasureRequest {
        blueos_idl::msg::fixture_msgs::MeasureRequest { probe }
    }
}

prop_compose! {
    pub(crate) fn fixture_msgs_measure_response_strategy()(level in any::<f32>(), progress in fixture_msgs_progress_strategy()) -> blueos_idl::msg::fixture_msgs::MeasureResponse {
        blueos_idl::msg::fixture_msgs::MeasureResponse { level, progress }
    }
}
