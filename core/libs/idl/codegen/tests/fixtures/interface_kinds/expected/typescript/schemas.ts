// @generated
export const SCHEMAS: Record<string, string> = {
  "fixture_msgs/action/Drain_Feedback": `Progress progress
================================================================================
MSG: fixture_msgs/Progress
# fixture_msgs/msg/Progress
uint64 done
uint64 total`,
  "fixture_msgs/action/Drain_Goal": `# fixture_msgs/action/Drain
# An empty Goal: draining needs no input.`,
  "fixture_msgs/action/Drain_Result": `float32 drained`,
  "fixture_msgs/action/Fill_Feedback": `Progress progress
================================================================================
MSG: fixture_msgs/Progress
# fixture_msgs/msg/Progress
uint64 done
uint64 total`,
  "fixture_msgs/action/Fill_Goal": `# fixture_msgs/action/Fill
float32 level
float32 rate`,
  "fixture_msgs/action/Fill_Result": `bool reached`,
  "fixture_msgs/msg/Progress": `# fixture_msgs/msg/Progress
uint64 done
uint64 total`,
  "fixture_msgs/srv/Measure_Request": `# fixture_msgs/srv/Measure
string probe`,
  "fixture_msgs/srv/Measure_Response": `float32 level
Progress progress
================================================================================
MSG: fixture_msgs/Progress
# fixture_msgs/msg/Progress
uint64 done
uint64 total`,
};
