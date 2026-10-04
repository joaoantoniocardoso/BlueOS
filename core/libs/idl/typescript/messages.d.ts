// @generated

export interface SetLevelFeedback {
  level: number;
}

export interface SetLevelGoal {
  level: number;
}

export interface SetLevelResult {
  level: number;
}

export interface PumpState {
  level: number;
  max_level: number;
  self_test_phase: number;
  self_test_active: boolean;
}

export interface SelfTestCompleted {
  passed: boolean;
  detail: string;
}

export interface LevelRequest {}

export interface LevelResponse {
  level: number;
  max_level: number;
}

export interface UpdateSettingsFeedback {}

export interface UpdateSettingsGoal {
  envelope: SettingsEnvelope;
}

export interface UpdateSettingsResult {}

export interface CommandAck {
  accepted: boolean;
  job_id: string;
  status: number;
  reason: string;
}

export interface EndpointInfo {
  kind: string;
  name: string;
  key: string;
  interface_type: string;
  schema: string;
}

export interface JobFeedback {
  job_id: string;
  feedback: Uint8Array;
}

export interface JobFeedbackList {
  jobs: JobFeedback[];
}

export interface JobList {
  jobs: JobStatus[];
}

export interface JobResult {
  job: JobStatus;
  result: Uint8Array;
}

export interface JobStatus {
  job_id: string;
  job_type: string;
  status: number;
  reason: string;
}

export interface PermissionAnswer {
  granted: boolean;
}

export interface RestartRequired {
  fields: string[];
}

export interface ServiceInfo {
  name: string;
  version: string;
  build: string;
  capabilities: string[];
  endpoints: EndpointInfo[];
}

export interface ServiceStatus {
  status: number;
  detail: string;
}

export interface SettingField {
  path: string;
  restart_required: boolean;
}

export interface SettingsEnvelope {
  document_json: string;
  fields: SettingField[];
}

export interface DeleteRecordingFeedback {}

export interface DeleteRecordingGoal {
  path: string;
}

export interface DeleteRecordingResult {
  path: string;
}

export interface RepairRecordingFeedback {
  bytes_processed: number;
  total_bytes: number;
}

export interface RepairRecordingGoal {
  path: string;
}

export interface RepairRecordingResult {
  path: string;
}

export interface SnapshotRecordingFeedback {
  output_path: string;
}

export interface SnapshotRecordingGoal {
  path: string;
}

export interface SnapshotRecordingResult {
  path: string;
  output_path: string;
}

export interface StartRecordingFeedback {}

export interface StartRecordingGoal {
  rotate_if_active: boolean;
}

export interface StartRecordingResult {}

export interface StopRecordingFeedback {}

export interface StopRecordingGoal {}

export interface StopRecordingResult {}

export interface ChannelMessageCount {
  channel_id: number;
  count: number;
}

export interface ChunkIndexEntry {
  start_time: number;
  end_time: number;
  offset: number;
  length: number;
  compression: string;
  compressed_size: number;
  uncompressed_size: number;
  channel_ids: number[];
  message_index_length: number;
}

export interface RecordingContents {
  path: string;
  duration: Duration;
  video_topics: string[];
  other_topic_count: number;
}

export interface RecordingFile {
  path: string;
  name: string;
  size_bytes: number;
  created: Time;
  state: number;
  repair_bytes_processed: number;
  repair_total_bytes: number;
  repair_bytes_per_second: number;
  repair_error: string;
  allowed_operations: string[];
  repair_job_id: string;
}

export interface RecordingLibrary {
  files: RecordingFile[];
  contents: RecordingContents[];
}

export interface RecordingState {
  armed: boolean;
  session_active: boolean;
  current_file: string;
  session_bytes_written: number;
  recording_video_topics: string[];
  samples_dropped: number;
}

export interface RecordingBytesRequest {
  path: string;
  offset: number;
  length: number;
  from_end: boolean;
}

export interface RecordingBytesResponse {
  size: number;
  data: Uint8Array;
}

export interface RecordingIndexRequest {
  path: string;
  from_offset: number;
  limit: number;
}

export interface RecordingIndexResponse {
  size: number;
  offset: number;
  closed: boolean;
  chunks: ChunkIndexEntry[];
  message_counts: ChannelMessageCount[];
  records: Uint8Array;
}

export interface Duration {
  sec: number;
  nanosec: number;
}

export interface Time {
  sec: number;
  nanosec: number;
}

export interface Log {
  timestamp: Time;
  level: number;
  message: string;
  name: string;
  file: string;
  line: number;
}

export interface Empty {}

export interface Header {
  stamp: Time;
  frame_id: string;
}

export interface MessageBySchema {
  "blueos_example_msgs/action/SetLevel_Feedback": SetLevelFeedback;
  "blueos_example_msgs/action/SetLevel_Goal": SetLevelGoal;
  "blueos_example_msgs/action/SetLevel_Result": SetLevelResult;
  "blueos_example_msgs/msg/PumpState": PumpState;
  "blueos_example_msgs/msg/SelfTestCompleted": SelfTestCompleted;
  "blueos_example_msgs/srv/Level_Request": LevelRequest;
  "blueos_example_msgs/srv/Level_Response": LevelResponse;
  "blueos_msgs/action/UpdateSettings_Feedback": UpdateSettingsFeedback;
  "blueos_msgs/action/UpdateSettings_Goal": UpdateSettingsGoal;
  "blueos_msgs/action/UpdateSettings_Result": UpdateSettingsResult;
  "blueos_msgs/msg/CommandAck": CommandAck;
  "blueos_msgs/msg/EndpointInfo": EndpointInfo;
  "blueos_msgs/msg/JobFeedback": JobFeedback;
  "blueos_msgs/msg/JobFeedbackList": JobFeedbackList;
  "blueos_msgs/msg/JobList": JobList;
  "blueos_msgs/msg/JobResult": JobResult;
  "blueos_msgs/msg/JobStatus": JobStatus;
  "blueos_msgs/msg/PermissionAnswer": PermissionAnswer;
  "blueos_msgs/msg/RestartRequired": RestartRequired;
  "blueos_msgs/msg/ServiceInfo": ServiceInfo;
  "blueos_msgs/msg/ServiceStatus": ServiceStatus;
  "blueos_msgs/msg/SettingField": SettingField;
  "blueos_msgs/msg/SettingsEnvelope": SettingsEnvelope;
  "blueos_recorder_msgs/action/DeleteRecording_Feedback": DeleteRecordingFeedback;
  "blueos_recorder_msgs/action/DeleteRecording_Goal": DeleteRecordingGoal;
  "blueos_recorder_msgs/action/DeleteRecording_Result": DeleteRecordingResult;
  "blueos_recorder_msgs/action/RepairRecording_Feedback": RepairRecordingFeedback;
  "blueos_recorder_msgs/action/RepairRecording_Goal": RepairRecordingGoal;
  "blueos_recorder_msgs/action/RepairRecording_Result": RepairRecordingResult;
  "blueos_recorder_msgs/action/SnapshotRecording_Feedback": SnapshotRecordingFeedback;
  "blueos_recorder_msgs/action/SnapshotRecording_Goal": SnapshotRecordingGoal;
  "blueos_recorder_msgs/action/SnapshotRecording_Result": SnapshotRecordingResult;
  "blueos_recorder_msgs/action/StartRecording_Feedback": StartRecordingFeedback;
  "blueos_recorder_msgs/action/StartRecording_Goal": StartRecordingGoal;
  "blueos_recorder_msgs/action/StartRecording_Result": StartRecordingResult;
  "blueos_recorder_msgs/action/StopRecording_Feedback": StopRecordingFeedback;
  "blueos_recorder_msgs/action/StopRecording_Goal": StopRecordingGoal;
  "blueos_recorder_msgs/action/StopRecording_Result": StopRecordingResult;
  "blueos_recorder_msgs/msg/ChannelMessageCount": ChannelMessageCount;
  "blueos_recorder_msgs/msg/ChunkIndexEntry": ChunkIndexEntry;
  "blueos_recorder_msgs/msg/RecordingContents": RecordingContents;
  "blueos_recorder_msgs/msg/RecordingFile": RecordingFile;
  "blueos_recorder_msgs/msg/RecordingLibrary": RecordingLibrary;
  "blueos_recorder_msgs/msg/RecordingState": RecordingState;
  "blueos_recorder_msgs/srv/RecordingBytes_Request": RecordingBytesRequest;
  "blueos_recorder_msgs/srv/RecordingBytes_Response": RecordingBytesResponse;
  "blueos_recorder_msgs/srv/RecordingIndex_Request": RecordingIndexRequest;
  "blueos_recorder_msgs/srv/RecordingIndex_Response": RecordingIndexResponse;
  "builtin_interfaces/msg/Duration": Duration;
  "builtin_interfaces/msg/Time": Time;
  "foxglove_msgs/msg/Log": Log;
  "std_msgs/msg/Empty": Empty;
  "std_msgs/msg/Header": Header;
}
