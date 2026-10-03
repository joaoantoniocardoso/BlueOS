// @generated
export const SCHEMAS: Record<string, string> = {
  "blueos_example_msgs/msg/EmptyRequest": `# blueos_example_msgs/msg/EmptyRequest
# Command payload with no semantics (StartSelfTest, CancelSelfTest). Declared empty, with no placeholder field.`,
  "blueos_example_msgs/msg/LevelQueryResponse": `# blueos_example_msgs/msg/LevelQueryResponse
# Reply for blueos/v1/example/query/Level.

uint8 level
uint8 max_level`,
  "blueos_example_msgs/msg/PumpState": `# blueos_example_msgs/msg/PumpState
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
  "blueos_example_msgs/msg/SelfTestCompleted": `# blueos_example_msgs/msg/SelfTestCompleted
# Event on blueos/v1/example/event/SelfTestCompleted.

bool passed
string detail`,
  "blueos_example_msgs/msg/SetLevelRequest": `# blueos_example_msgs/msg/SetLevelRequest
# Payload for blueos/v1/example/command/SetLevel.

uint8 level`,
  "blueos_msgs/msg/CommandAck": `# blueos_msgs/msg/CommandAck
# Reply to a Command, which submits a Job or controls one (D-10, D-36).

# The STATUS_ values of blueos_msgs/JobStatus. STATUS_UNKNOWN when the Command named no Job.
uint8 STATUS_UNKNOWN=0
uint8 STATUS_ACCEPTED=1
uint8 STATUS_EXECUTING=2
uint8 STATUS_CANCELING=3
uint8 STATUS_SUCCEEDED=4
uint8 STATUS_CANCELED=5
uint8 STATUS_ABORTED=6
uint8 STATUS_WAITING_FOR_PERMISSION=7
uint8 STATUS_WAITING_FOR_RESOURCE=8
uint8 STATUS_PAUSED=9

bool accepted
# The UUID the client generated for the Job, as text.
string job_id
# The status of the Job after the Command was applied, so a Job that ended in that step returns its final status.
uint8 status
string reason`,
  "blueos_msgs/msg/EndpointInfo": `# blueos_msgs/msg/EndpointInfo
# One key a service serves, listed in ServiceInfo.endpoints so clients can discover the API (D-12, D-24).

# One of: command, query, io_query, state, event.
string kind
string name
string key
# Schema of the payload the client sends (command, query, io_query). Empty when the endpoint takes none.
string request_schema
# Schema of the reply (command: CommandAck; query, io_query) or of the published sample (state, event).
string response_schema`,
  "blueos_msgs/msg/JobFeedback": `# blueos_msgs/msg/JobFeedback
# The latest Feedback of one active Job, in the jobs/<JobType>/feedback State (D-12, D-36), like the feedback of a ROS 2
# action.

# The UUID the client generated for the Job, as text.
string job_id
# The Job type's Feedback message, CDR with its encapsulation header.
uint8[] feedback`,
  "blueos_msgs/msg/JobFeedbackList": `# blueos_msgs/msg/JobFeedbackList
# State published on blueos/v1/<service>/jobs/<JobType>/feedback: the latest Feedback of each active Job of the type,
# in the order the Jobs were submitted. A Job leaves it when it ends.

blueos_msgs/JobFeedback[] jobs
================================================================================
MSG: blueos_msgs/JobFeedback
# blueos_msgs/msg/JobFeedback
# The latest Feedback of one active Job, in the jobs/<JobType>/feedback State (D-12, D-36), like the feedback of a ROS 2
# action.

# The UUID the client generated for the Job, as text.
string job_id
# The Job type's Feedback message, CDR with its encapsulation header.
uint8[] feedback`,
  "blueos_msgs/msg/JobList": `# blueos_msgs/msg/JobList
# Snapshot published on blueos/v1/<service>/jobs.

blueos_msgs/JobStatus[] jobs
================================================================================
MSG: blueos_msgs/JobStatus
# blueos_msgs/msg/JobStatus
# One Job in the jobs State (D-12, D-36). STATUS_ values 0 to 6 are those of ROS 2 action_msgs/GoalStatus; a ROS 2
# client sees the two waiting statuses as STATUS_ACCEPTED, and STATUS_PAUSED as STATUS_EXECUTING (D-38).

uint8 STATUS_UNKNOWN=0
uint8 STATUS_ACCEPTED=1
uint8 STATUS_EXECUTING=2
uint8 STATUS_CANCELING=3
uint8 STATUS_SUCCEEDED=4
uint8 STATUS_CANCELED=5
uint8 STATUS_ABORTED=6
uint8 STATUS_WAITING_FOR_PERMISSION=7
uint8 STATUS_WAITING_FOR_RESOURCE=8
uint8 STATUS_PAUSED=9

# The UUID the client generated for the Job, as text.
string job_id
# The Job type, the name of its submit endpoint.
string job_type
uint8 status
# Why the Job was canceled or aborted, when the Kernel ended it.
string reason`,
  "blueos_msgs/msg/JobResult": `# blueos_msgs/msg/JobResult
# Event published on blueos/v1/<service>/jobs/<JobType>/result when a Job ends (D-12, D-36), like the result of a ROS 2
# action.

# The Job as it ended: Succeeded, Canceled or Aborted, with its reason.
blueos_msgs/JobStatus job
# The Job type's Job result message, CDR with its encapsulation header. Empty when the Job type declares none.
uint8[] result
================================================================================
MSG: blueos_msgs/JobStatus
# blueos_msgs/msg/JobStatus
# One Job in the jobs State (D-12, D-36). STATUS_ values 0 to 6 are those of ROS 2 action_msgs/GoalStatus; a ROS 2
# client sees the two waiting statuses as STATUS_ACCEPTED, and STATUS_PAUSED as STATUS_EXECUTING (D-38).

uint8 STATUS_UNKNOWN=0
uint8 STATUS_ACCEPTED=1
uint8 STATUS_EXECUTING=2
uint8 STATUS_CANCELING=3
uint8 STATUS_SUCCEEDED=4
uint8 STATUS_CANCELED=5
uint8 STATUS_ABORTED=6
uint8 STATUS_WAITING_FOR_PERMISSION=7
uint8 STATUS_WAITING_FOR_RESOURCE=8
uint8 STATUS_PAUSED=9

# The UUID the client generated for the Job, as text.
string job_id
# The Job type, the name of its submit endpoint.
string job_type
uint8 status
# Why the Job was canceled or aborted, when the Kernel ended it.
string reason`,
  "blueos_msgs/msg/JobStatus": `# blueos_msgs/msg/JobStatus
# One Job in the jobs State (D-12, D-36). STATUS_ values 0 to 6 are those of ROS 2 action_msgs/GoalStatus; a ROS 2
# client sees the two waiting statuses as STATUS_ACCEPTED, and STATUS_PAUSED as STATUS_EXECUTING (D-38).

uint8 STATUS_UNKNOWN=0
uint8 STATUS_ACCEPTED=1
uint8 STATUS_EXECUTING=2
uint8 STATUS_CANCELING=3
uint8 STATUS_SUCCEEDED=4
uint8 STATUS_CANCELED=5
uint8 STATUS_ABORTED=6
uint8 STATUS_WAITING_FOR_PERMISSION=7
uint8 STATUS_WAITING_FOR_RESOURCE=8
uint8 STATUS_PAUSED=9

# The UUID the client generated for the Job, as text.
string job_id
# The Job type, the name of its submit endpoint.
string job_type
uint8 status
# Why the Job was canceled or aborted, when the Kernel ended it.
string reason`,
  "blueos_msgs/msg/PermissionAnswer": `# blueos_msgs/msg/PermissionAnswer
# The body of command/AnswerPermission, whose attachment names the Job waiting for permission (D-36).

# True lets the Job execute; false cancels it.
bool granted`,
  "blueos_msgs/msg/RestartRequired": `# blueos_msgs/msg/RestartRequired
# Event listing settings fields that need a service restart (D-11).

string[] fields`,
  "blueos_msgs/msg/ServiceInfo": `# blueos_msgs/msg/ServiceInfo
# Metadata exposed on blueos/v1/<service>/info (D-12).

string name
string version
string build
string[] capabilities
blueos_msgs/EndpointInfo[] endpoints
================================================================================
MSG: blueos_msgs/EndpointInfo
# blueos_msgs/msg/EndpointInfo
# One key a service serves, listed in ServiceInfo.endpoints so clients can discover the API (D-12, D-24).

# One of: command, query, io_query, state, event.
string kind
string name
string key
# Schema of the payload the client sends (command, query, io_query). Empty when the endpoint takes none.
string request_schema
# Schema of the reply (command: CommandAck; query, io_query) or of the published sample (state, event).
string response_schema`,
  "blueos_msgs/msg/ServiceStatus": `# blueos_msgs/msg/ServiceStatus
# High-level service health on the status state key (D-12).

uint8 STATUS_UNKNOWN=0
uint8 STATUS_STARTING=1
uint8 STATUS_READY=2
uint8 STATUS_DEGRADED=3
uint8 STATUS_STOPPING=4

uint8 status
string detail`,
  "blueos_msgs/msg/SettingField": `# blueos_msgs/msg/SettingField
# Restart hint for one settings field (D-11).

string path
bool restart_required`,
  "blueos_msgs/msg/SettingsEnvelope": `# blueos_msgs/msg/SettingsEnvelope
# JSON settings document plus per-field restart flags (D-11).

string document_json
blueos_msgs/SettingField[] fields
================================================================================
MSG: blueos_msgs/SettingField
# blueos_msgs/msg/SettingField
# Restart hint for one settings field (D-11).

string path
bool restart_required`,
  "blueos_recorder_msgs/msg/ChannelMessageCount": `# blueos_recorder_msgs/msg/ChannelMessageCount

uint16 channel_id
uint64 count`,
  "blueos_recorder_msgs/msg/ChunkIndexEntry": `# blueos_recorder_msgs/msg/ChunkIndexEntry

uint64 start_time
uint64 end_time
# Offset and length of the whole Chunk record, header included.
uint64 offset
uint64 length
string compression
uint64 compressed_size
uint64 uncompressed_size
uint16[] channel_ids
# Bytes of the MessageIndex records that follow the chunk.
uint64 message_index_length`,
  "blueos_recorder_msgs/msg/DeleteRecordingCommand": `# blueos_recorder_msgs/msg/DeleteRecordingCommand
# Rejected while the recording is being written or repaired.

string path`,
  "blueos_recorder_msgs/msg/RecordingFile": `# blueos_recorder_msgs/msg/RecordingFile
# One MCAP recording in the recorder folder, as listed in RecordingLibrary.

# Being written by the recorder; bytes are readable but the file has no summary yet.
uint8 STATE_RECORDING=0
# Finished and indexed; seekable through HTTP ranges on /userdata/recorder/<path>.
uint8 STATE_READY=1
# Finished without a summary (power loss, crash); RepairRecording gives it back.
uint8 STATE_NEEDS_REPAIR=2
uint8 STATE_REPAIRING=3

# Relative to the recorder folder, forward slashes. Identifies the recording in every command.
string path
string name
uint64 size_bytes
# From the timestamp embedded in the file name, falling back to the file time.
builtin_interfaces/Time created
uint8 state
# Repair progress while STATE_REPAIRING; zero otherwise.
uint64 repair_bytes_processed
uint64 repair_total_bytes
float64 repair_bytes_per_second
# Reason the last repair failed; empty when it did not fail. Cleared by the next repair.
string repair_error
# Command endpoint names the library will accept for this file (for example DeleteRecording).
string[] allowed_operations
# The Job id of the repair while STATE_REPAIRING, which CancelJob names; empty otherwise.
string repair_job_id
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9).
uint32 nanosec`,
  "blueos_recorder_msgs/msg/RecordingIndex": `# blueos_recorder_msgs/msg/RecordingIndex
# Reply of the index query: one page of a walk over record headers, so the browser can fetch chunk
# bodies with HTTP ranges even when the file has no summary yet (still recording, needs repair).

uint64 size
# Where the next page starts.
uint64 offset
# A DataEnd or Footer record was reached: the walk is complete.
bool closed
blueos_recorder_msgs/ChunkIndexEntry[] chunks
blueos_recorder_msgs/ChannelMessageCount[] message_counts
# Raw Header, Schema, Channel and Metadata records met in this page, in file order.
uint8[] records
================================================================================
MSG: blueos_recorder_msgs/ChannelMessageCount
# blueos_recorder_msgs/msg/ChannelMessageCount

uint16 channel_id
uint64 count
================================================================================
MSG: blueos_recorder_msgs/ChunkIndexEntry
# blueos_recorder_msgs/msg/ChunkIndexEntry

uint64 start_time
uint64 end_time
# Offset and length of the whole Chunk record, header included.
uint64 offset
uint64 length
string compression
uint64 compressed_size
uint64 uncompressed_size
uint16[] channel_ids
# Bytes of the MessageIndex records that follow the chunk.
uint64 message_index_length`,
  "blueos_recorder_msgs/msg/RecordingIndexRequest": `# blueos_recorder_msgs/msg/RecordingIndexRequest
# Payload of the blueos/v1/recorder/query/index query.

string path
# 0 starts at the file magic; otherwise the \`offset\` of a previous RecordingIndex reply.
uint64 from_offset
# Maximum chunks in the reply (1..=20000).
uint32 limit`,
  "blueos_recorder_msgs/msg/RecordingLibrary": `# blueos_recorder_msgs/msg/RecordingLibrary
# Published on blueos/v1/recorder/state/library, newest recording first.

blueos_recorder_msgs/RecordingFile[] files
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9).
uint32 nanosec
================================================================================
MSG: blueos_recorder_msgs/RecordingFile
# blueos_recorder_msgs/msg/RecordingFile
# One MCAP recording in the recorder folder, as listed in RecordingLibrary.

# Being written by the recorder; bytes are readable but the file has no summary yet.
uint8 STATE_RECORDING=0
# Finished and indexed; seekable through HTTP ranges on /userdata/recorder/<path>.
uint8 STATE_READY=1
# Finished without a summary (power loss, crash); RepairRecording gives it back.
uint8 STATE_NEEDS_REPAIR=2
uint8 STATE_REPAIRING=3

# Relative to the recorder folder, forward slashes. Identifies the recording in every command.
string path
string name
uint64 size_bytes
# From the timestamp embedded in the file name, falling back to the file time.
builtin_interfaces/Time created
uint8 state
# Repair progress while STATE_REPAIRING; zero otherwise.
uint64 repair_bytes_processed
uint64 repair_total_bytes
float64 repair_bytes_per_second
# Reason the last repair failed; empty when it did not fail. Cleared by the next repair.
string repair_error
# Command endpoint names the library will accept for this file (for example DeleteRecording).
string[] allowed_operations
# The Job id of the repair while STATE_REPAIRING, which CancelJob names; empty otherwise.
string repair_job_id`,
  "blueos_recorder_msgs/msg/RecordingOperation": `# blueos_recorder_msgs/msg/RecordingOperation
# Published on blueos/v1/recorder/event/operation when a repair, snapshot or delete ends.

uint8 OPERATION_REPAIR=0
uint8 OPERATION_SNAPSHOT=1
uint8 OPERATION_DELETE=2

uint8 operation
string path
# The snapshot copy; empty for other operations.
string output_path
bool succeeded
bool cancelled
string error`,
  "blueos_recorder_msgs/msg/RecordingPolicy": `# blueos_recorder_msgs/msg/RecordingPolicy
# Persisted recorder settings (D-11).

bool record_mavlink_only_when_armed
bool auto_start_recording`,
  "blueos_recorder_msgs/msg/RecordingState": `# blueos_recorder_msgs/msg/RecordingState
# Published on blueos/v1/recorder/state/recording.

bool armed
bool session_active
string current_file
uint64 session_bytes_written
string[] recording_video_topics`,
  "blueos_recorder_msgs/msg/RepairRecordingCommand": `# blueos_recorder_msgs/msg/RepairRecordingCommand
# Rewrites a STATE_NEEDS_REPAIR recording so it has a summary again. Progress is on the library state.

string path`,
  "blueos_recorder_msgs/msg/SetPolicyCommand": `# blueos_recorder_msgs/msg/SetPolicyCommand

blueos_recorder_msgs/RecordingPolicy policy
================================================================================
MSG: blueos_recorder_msgs/RecordingPolicy
# blueos_recorder_msgs/msg/RecordingPolicy
# Persisted recorder settings (D-11).

bool record_mavlink_only_when_armed
bool auto_start_recording`,
  "blueos_recorder_msgs/msg/SnapshotRecordingCommand": `# blueos_recorder_msgs/msg/SnapshotRecordingCommand
# Writes an indexed copy of a recording (typically the one being written) next to it, named
# <stem>.snapshot-<UTC ISO time>Z.mcap. The copy is announced by a RecordingOperation event.

string path`,
  "blueos_recorder_msgs/msg/StartRecordingCommand": `# blueos_recorder_msgs/msg/StartRecordingCommand
# Opens a new MCAP session (rotate if one is already active).

bool rotate_if_active`,
  "blueos_recorder_msgs/msg/StopRecordingCommand": `# blueos_recorder_msgs/msg/StopRecordingCommand
# Finishes the current MCAP session; samples are dropped until StartRecording.

uint8 reserved`,
  "builtin_interfaces/msg/Duration": `# This message communicates ROS Duration.

int32 sec
uint32 nanosec`,
  "builtin_interfaces/msg/Time": `# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9).
uint32 nanosec`,
  "foxglove_msgs/msg/Log": `# foxglove_msgs/msg/Log
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
uint32 line
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9).
uint32 nanosec`,
  "std_msgs/msg/Header": `# Standard metadata for higher-level stamped data types.

builtin_interfaces/Time stamp
string frame_id
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9).
uint32 nanosec`,
};
