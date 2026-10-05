// @generated

export interface DrainFeedback {
  progress: Progress;
}

export interface DrainGoal {}

export interface DrainResult {
  drained: number;
}

export interface FillFeedback {
  progress: Progress;
}

export interface FillGoal {
  level: number;
  rate: number;
}

export interface FillResult {
  reached: boolean;
}

export interface Progress {
  done: number;
  total: number;
}

export interface Samples {
  samples: Uint8Array;
  tag: Uint8Array;
  counts: number[];
}

export interface MeasureRequest {
  probe: string;
}

export interface MeasureResponse {
  level: number;
  progress: Progress;
}

export interface MessageBySchema {
  "fixture_msgs/action/Drain_Feedback": DrainFeedback;
  "fixture_msgs/action/Drain_Goal": DrainGoal;
  "fixture_msgs/action/Drain_Result": DrainResult;
  "fixture_msgs/action/Fill_Feedback": FillFeedback;
  "fixture_msgs/action/Fill_Goal": FillGoal;
  "fixture_msgs/action/Fill_Result": FillResult;
  "fixture_msgs/msg/Progress": Progress;
  "fixture_msgs/msg/Samples": Samples;
  "fixture_msgs/srv/Measure_Request": MeasureRequest;
  "fixture_msgs/srv/Measure_Response": MeasureResponse;
}
