// @generated
export const SCHEMAS: Record<string, string> = {
  "blueos_example_msgs/msg/EmptyRequest": `MSG: blueos_example_msgs/msg/EmptyRequest
# blueos_example_msgs/msg/EmptyRequest
# Command payload with no semantics (StartSelfTest, CancelSelfTest). Padding keeps CDR codegen happy.

uint8 padding`,
  "blueos_example_msgs/msg/LevelQueryResponse": `MSG: blueos_example_msgs/msg/LevelQueryResponse
# blueos_example_msgs/msg/LevelQueryResponse
# Reply for blueos/v1/example/query/Level.

uint8 level
uint8 max_level`,
  "blueos_example_msgs/msg/PumpState": `MSG: blueos_example_msgs/msg/PumpState
# blueos_example_msgs/msg/PumpState
# Published on blueos/v1/example/state/pump (teaching example, D-20).

uint8 SELF_TEST_IDLE=0
uint8 SELF_TEST_RUNNING=1
uint8 SELF_TEST_PASSED=2
uint8 SELF_TEST_FAILED=3
uint8 SELF_TEST_CANCELLED=4

uint8 level
uint8 max_level
uint8 self_test_phase
bool self_test_active`,
  "blueos_example_msgs/msg/SelfTestCompleted": `MSG: blueos_example_msgs/msg/SelfTestCompleted
# blueos_example_msgs/msg/SelfTestCompleted
# Event on blueos/v1/example/event/SelfTestCompleted.

bool passed
string detail`,
  "blueos_example_msgs/msg/SetLevelRequest": `MSG: blueos_example_msgs/msg/SetLevelRequest
# blueos_example_msgs/msg/SetLevelRequest
# Payload for blueos/v1/example/command/SetLevel.

uint8 level`,
  "blueos_msgs/msg/CommandAck": `MSG: blueos_msgs/msg/CommandAck
# blueos_msgs/msg/CommandAck
# Reply to a Zenoh command query (D-10).

bool accepted
uint64 job_id
string reason`,
  "blueos_msgs/msg/JobList": `MSG: blueos_msgs/msg/JobStatus
# blueos_msgs/msg/JobStatus
# One job entry; status values mirror blueos_jobs (D-12).

uint8 STATUS_QUEUED=0
uint8 STATUS_RUNNING=1
uint8 STATUS_CANCELLING=2
uint8 STATUS_SUCCEEDED=3
uint8 STATUS_FAILED=4
uint8 STATUS_CANCELLED=5

uint64 job_id
uint64 parent_job_id
uint8 status
string name
================================================================================
MSG: blueos_msgs/msg/JobList
# blueos_msgs/msg/JobList
# Snapshot published on blueos/v1/<service>/jobs.

blueos_msgs/JobStatus[] jobs`,
  "blueos_msgs/msg/JobStatus": `MSG: blueos_msgs/msg/JobStatus
# blueos_msgs/msg/JobStatus
# One job entry; status values mirror blueos_jobs (D-12).

uint8 STATUS_QUEUED=0
uint8 STATUS_RUNNING=1
uint8 STATUS_CANCELLING=2
uint8 STATUS_SUCCEEDED=3
uint8 STATUS_FAILED=4
uint8 STATUS_CANCELLED=5

uint64 job_id
uint64 parent_job_id
uint8 status
string name`,
  "blueos_msgs/msg/RestartRequired": `MSG: blueos_msgs/msg/RestartRequired
# blueos_msgs/msg/RestartRequired
# Event listing settings fields that need a service restart (D-11).

string[] fields`,
  "blueos_msgs/msg/ServiceInfo": `MSG: blueos_msgs/msg/ServiceInfo
# blueos_msgs/msg/ServiceInfo
# Metadata exposed on blueos/v1/<service>/info (D-12).

string name
string version
string build
string[] capabilities`,
  "blueos_msgs/msg/ServiceStatus": `MSG: blueos_msgs/msg/ServiceStatus
# blueos_msgs/msg/ServiceStatus
# High-level service health on the status state key (D-12).

uint8 STATUS_UNKNOWN=0
uint8 STATUS_STARTING=1
uint8 STATUS_READY=2
uint8 STATUS_DEGRADED=3
uint8 STATUS_STOPPING=4

uint8 status
string detail`,
  "blueos_msgs/msg/SettingField": `MSG: blueos_msgs/msg/SettingField
# blueos_msgs/msg/SettingField
# Restart hint for one settings field (D-11).

string path
bool restart_required`,
  "blueos_msgs/msg/SettingsEnvelope": `MSG: blueos_msgs/msg/SettingField
# blueos_msgs/msg/SettingField
# Restart hint for one settings field (D-11).

string path
bool restart_required
================================================================================
MSG: blueos_msgs/msg/SettingsEnvelope
# blueos_msgs/msg/SettingsEnvelope
# JSON settings document plus per-field restart flags (D-11).

string document_json
blueos_msgs/SettingField[] fields`,
  "blueos_recorder_msgs/msg/RecordingPolicy": `MSG: blueos_recorder_msgs/msg/RecordingPolicy
# blueos_recorder_msgs/msg/RecordingPolicy
# Persisted recorder settings (D-11).

bool record_mavlink_only_when_armed
bool auto_start_recording`,
  "blueos_recorder_msgs/msg/RecordingState": `MSG: blueos_recorder_msgs/msg/RecordingState
# blueos_recorder_msgs/msg/RecordingState
# Published on blueos/v1/recorder/state/recording.

bool armed
bool session_active
string current_file
uint64 session_bytes_written
string[] recording_video_topics`,
  "blueos_recorder_msgs/msg/SetPolicyCommand": `MSG: blueos_recorder_msgs/msg/RecordingPolicy
# blueos_recorder_msgs/msg/RecordingPolicy
# Persisted recorder settings (D-11).

bool record_mavlink_only_when_armed
bool auto_start_recording
================================================================================
MSG: blueos_recorder_msgs/msg/SetPolicyCommand
# blueos_recorder_msgs/msg/SetPolicyCommand

blueos_recorder_msgs/RecordingPolicy policy`,
  "blueos_recorder_msgs/msg/StartRecordingCommand": `MSG: blueos_recorder_msgs/msg/StartRecordingCommand
# blueos_recorder_msgs/msg/StartRecordingCommand
# Opens a new MCAP session (rotate if one is already active).

bool rotate_if_active`,
  "blueos_recorder_msgs/msg/StopRecordingCommand": `MSG: blueos_recorder_msgs/msg/StopRecordingCommand
# blueos_recorder_msgs/msg/StopRecordingCommand
# Finishes the current MCAP session; samples are dropped until StartRecording.

uint8 reserved`,
  "builtin_interfaces/msg/Duration": `MSG: builtin_interfaces/msg/Duration
# This message communicates ROS Duration.

int32 sec
uint32 nanosec`,
  "builtin_interfaces/msg/Time": `MSG: builtin_interfaces/msg/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9).
uint32 nanosec`,
  "foxglove_msgs/msg/Log": `MSG: builtin_interfaces/msg/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9).
uint32 nanosec
================================================================================
MSG: foxglove_msgs/msg/Log
# foxglove_msgs/msg/Log
# A log message for service tracing output (D-13).

builtin_interfaces/Time timestamp

uint8 UNKNOWN=0
uint8 DEBUG=1
uint8 INFO=2
uint8 WARNING=3
uint8 ERROR=4
uint8 FATAL=5

uint8 level
string message
string name
string file
uint32 line`,
  "std_msgs/msg/Header": `MSG: builtin_interfaces/msg/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9).
uint32 nanosec
================================================================================
MSG: std_msgs/msg/Header
# Standard metadata for higher-level stamped data types.

builtin_interfaces/Time stamp
string frame_id`,
};
