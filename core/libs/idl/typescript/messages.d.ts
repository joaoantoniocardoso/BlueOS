// @generated

export interface EmptyRequest {}

export interface LevelQueryResponse {
  level: number;
  max_level: number;
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

export interface SetLevelRequest {
  level: number;
}

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
  request_schema: string;
  response_schema: string;
}

export interface JobFeedback {
  job_id: string;
  feedback: number[];
}

export interface JobFeedbackList {
  jobs: JobFeedback[];
}

export interface JobList {
  jobs: JobStatus[];
}

export interface JobResult {
  job: JobStatus;
  result: number[];
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

export interface CancelRepairCommand {
  path: string;
}

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

export interface DeleteRecordingCommand {
  path: string;
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
}

export interface RecordingIndex {
  size: number;
  offset: number;
  closed: boolean;
  chunks: ChunkIndexEntry[];
  message_counts: ChannelMessageCount[];
  records: number[];
}

export interface RecordingIndexRequest {
  path: string;
  from_offset: number;
  limit: number;
}

export interface RecordingLibrary {
  files: RecordingFile[];
}

export interface RecordingOperation {
  operation: number;
  path: string;
  output_path: string;
  succeeded: boolean;
  cancelled: boolean;
  error: string;
}

export interface RecordingPolicy {
  record_mavlink_only_when_armed: boolean;
  auto_start_recording: boolean;
}

export interface RecordingState {
  armed: boolean;
  session_active: boolean;
  current_file: string;
  session_bytes_written: number;
  recording_video_topics: string[];
}

export interface RepairRecordingCommand {
  path: string;
}

export interface SetPolicyCommand {
  policy: RecordingPolicy;
}

export interface SnapshotRecordingCommand {
  path: string;
}

export interface StartRecordingCommand {
  rotate_if_active: boolean;
}

export interface StopRecordingCommand {
  reserved: number;
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

export interface Header {
  stamp: Time;
  frame_id: string;
}

export interface MessageBySchema {
  "blueos_example_msgs/msg/EmptyRequest": EmptyRequest;
  "blueos_example_msgs/msg/LevelQueryResponse": LevelQueryResponse;
  "blueos_example_msgs/msg/PumpState": PumpState;
  "blueos_example_msgs/msg/SelfTestCompleted": SelfTestCompleted;
  "blueos_example_msgs/msg/SetLevelRequest": SetLevelRequest;
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
  "blueos_recorder_msgs/msg/CancelRepairCommand": CancelRepairCommand;
  "blueos_recorder_msgs/msg/ChannelMessageCount": ChannelMessageCount;
  "blueos_recorder_msgs/msg/ChunkIndexEntry": ChunkIndexEntry;
  "blueos_recorder_msgs/msg/DeleteRecordingCommand": DeleteRecordingCommand;
  "blueos_recorder_msgs/msg/RecordingFile": RecordingFile;
  "blueos_recorder_msgs/msg/RecordingIndex": RecordingIndex;
  "blueos_recorder_msgs/msg/RecordingIndexRequest": RecordingIndexRequest;
  "blueos_recorder_msgs/msg/RecordingLibrary": RecordingLibrary;
  "blueos_recorder_msgs/msg/RecordingOperation": RecordingOperation;
  "blueos_recorder_msgs/msg/RecordingPolicy": RecordingPolicy;
  "blueos_recorder_msgs/msg/RecordingState": RecordingState;
  "blueos_recorder_msgs/msg/RepairRecordingCommand": RepairRecordingCommand;
  "blueos_recorder_msgs/msg/SetPolicyCommand": SetPolicyCommand;
  "blueos_recorder_msgs/msg/SnapshotRecordingCommand": SnapshotRecordingCommand;
  "blueos_recorder_msgs/msg/StartRecordingCommand": StartRecordingCommand;
  "blueos_recorder_msgs/msg/StopRecordingCommand": StopRecordingCommand;
  "builtin_interfaces/msg/Duration": Duration;
  "builtin_interfaces/msg/Time": Time;
  "foxglove_msgs/msg/Log": Log;
  "std_msgs/msg/Header": Header;
}
