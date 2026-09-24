// @generated

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

