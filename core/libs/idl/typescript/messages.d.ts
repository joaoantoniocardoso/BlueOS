// @generated

export interface EmptyRequest {
  padding: number;
}

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
  job_id: number;
  reason: string;
}

export interface JobList {
  jobs: JobStatus[];
}

export interface JobStatus {
  job_id: number;
  parent_job_id: number;
  status: number;
  name: string;
}

export interface RestartRequired {
  fields: string[];
}

export interface ServiceInfo {
  name: string;
  version: string;
  build: string;
  capabilities: string[];
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

