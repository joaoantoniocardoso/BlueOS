#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "generated from ROS .msg sources"
)]
pub mod msg;
use crate::message::Message;
/// ROS 2 `.msg` text of any IDL message, keyed by its `SCHEMA_NAME`.
pub fn schema(schema_name: &str) -> Option<&'static str> {
    match schema_name {
        "fixture_msgs/action/Drain_Feedback" => Some(msg::fixture_msgs::DrainFeedback::SCHEMA),
        "fixture_msgs/action/Drain_Goal" => Some(msg::fixture_msgs::DrainGoal::SCHEMA),
        "fixture_msgs/action/Drain_Result" => Some(msg::fixture_msgs::DrainResult::SCHEMA),
        "fixture_msgs/action/Fill_Feedback" => Some(msg::fixture_msgs::FillFeedback::SCHEMA),
        "fixture_msgs/action/Fill_Goal" => Some(msg::fixture_msgs::FillGoal::SCHEMA),
        "fixture_msgs/action/Fill_Result" => Some(msg::fixture_msgs::FillResult::SCHEMA),
        "fixture_msgs/msg/Progress" => Some(msg::fixture_msgs::Progress::SCHEMA),
        "fixture_msgs/srv/Measure_Request" => Some(msg::fixture_msgs::MeasureRequest::SCHEMA),
        "fixture_msgs/srv/Measure_Response" => Some(msg::fixture_msgs::MeasureResponse::SCHEMA),
        "fixture_msgs/action/Drain" => Some(
            "# fixture_msgs/action/Drain\n# An empty Goal: draining needs no input.\n---\nfloat32 drained\n---\nProgress progress\n================================================================================\nMSG: fixture_msgs/Progress\n# fixture_msgs/msg/Progress\nuint64 done\nuint64 total",
        ),
        "fixture_msgs/action/Fill" => Some(
            "# fixture_msgs/action/Fill\nfloat32 level\nfloat32 rate\n---\nbool reached\n---\nProgress progress\n================================================================================\nMSG: fixture_msgs/Progress\n# fixture_msgs/msg/Progress\nuint64 done\nuint64 total",
        ),
        "fixture_msgs/srv/Measure" => Some(
            "# fixture_msgs/srv/Measure\nstring probe\n---\nfloat32 level\nProgress progress\n================================================================================\nMSG: fixture_msgs/Progress\n# fixture_msgs/msg/Progress\nuint64 done\nuint64 total",
        ),
        _ => None,
    }
}
