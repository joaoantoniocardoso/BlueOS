// @generated
use blueos_idl::Message;

pub fn encode_default(schema_name: &str) -> Option<Vec<u8>> {
    match schema_name {
        "fixture_msgs/action/Drain_Feedback" => blueos_idl::msg::fixture_msgs::DrainFeedback::default().encode().ok(),
        "fixture_msgs/action/Drain_Goal" => blueos_idl::msg::fixture_msgs::DrainGoal::default().encode().ok(),
        "fixture_msgs/action/Drain_Result" => blueos_idl::msg::fixture_msgs::DrainResult::default().encode().ok(),
        "fixture_msgs/action/Fill_Feedback" => blueos_idl::msg::fixture_msgs::FillFeedback::default().encode().ok(),
        "fixture_msgs/action/Fill_Goal" => blueos_idl::msg::fixture_msgs::FillGoal::default().encode().ok(),
        "fixture_msgs/action/Fill_Result" => blueos_idl::msg::fixture_msgs::FillResult::default().encode().ok(),
        "fixture_msgs/msg/Progress" => blueos_idl::msg::fixture_msgs::Progress::default().encode().ok(),
        "fixture_msgs/srv/Measure_Request" => blueos_idl::msg::fixture_msgs::MeasureRequest::default().encode().ok(),
        "fixture_msgs/srv/Measure_Response" => blueos_idl::msg::fixture_msgs::MeasureResponse::default().encode().ok(),
        _ => None,
    }
}

pub fn decode_to_json(schema_name: &str, payload: &[u8]) -> Option<serde_json::Value> {
    match schema_name {
        "fixture_msgs/action/Drain_Feedback" => blueos_idl::msg::fixture_msgs::DrainFeedback::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        "fixture_msgs/action/Drain_Goal" => blueos_idl::msg::fixture_msgs::DrainGoal::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        "fixture_msgs/action/Drain_Result" => blueos_idl::msg::fixture_msgs::DrainResult::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        "fixture_msgs/action/Fill_Feedback" => blueos_idl::msg::fixture_msgs::FillFeedback::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        "fixture_msgs/action/Fill_Goal" => blueos_idl::msg::fixture_msgs::FillGoal::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        "fixture_msgs/action/Fill_Result" => blueos_idl::msg::fixture_msgs::FillResult::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        "fixture_msgs/msg/Progress" => blueos_idl::msg::fixture_msgs::Progress::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        "fixture_msgs/srv/Measure_Request" => blueos_idl::msg::fixture_msgs::MeasureRequest::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        "fixture_msgs/srv/Measure_Response" => blueos_idl::msg::fixture_msgs::MeasureResponse::decode(payload).ok().and_then(|message| serde_json::to_value(message).ok()),
        _ => None,
    }
}
