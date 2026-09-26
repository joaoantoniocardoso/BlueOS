//! Sans-IO gate for ros2dds topics that may arrive before their liveliness token (D-24).

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::{MAVLINK_RAW_TOPIC_PREFIX, MAVLINK_TOPIC_PREFIX, VIDEO_TOPIC_PREFIX};

pub const ROS2DDS_HOLD_MAX_SAMPLES: usize = 64;
pub const ROS2DDS_HOLD_TIMEOUT_MS: u64 = 2_000;

fn has_cdr_encapsulation_header(payload: &[u8]) -> bool {
    payload.len() >= 4
        && payload[0] == 0x00
        && (payload[1] == 0x00 || payload[1] == 0x01)
        && payload[2] == 0x00
        && payload[3] == 0x00
}

/// Whether schema resolution may consult ROS 2 transports for this sample.
pub fn is_ros2_schema_candidate(topic: &str, encoding: &str, payload: &[u8]) -> bool {
    if topic.starts_with("blueos/")
        || topic.starts_with(VIDEO_TOPIC_PREFIX)
        || topic.starts_with(MAVLINK_TOPIC_PREFIX)
        || topic.starts_with(MAVLINK_RAW_TOPIC_PREFIX)
    {
        return false;
    }
    if encoding.starts_with("application/json") || encoding.starts_with("text/") {
        return false;
    }
    if !ros2_wire_encoding_base(encoding) {
        return false;
    }
    has_cdr_encapsulation_header(payload)
}

fn ros2_wire_encoding_base(encoding: &str) -> bool {
    if encoding.is_empty() || encoding == "zenoh/bytes" {
        return true;
    }
    let mut parts = encoding.split(';');
    let base = parts.next().unwrap_or(encoding);
    base == "application/cdr" && parts.next().is_none()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ros2ddsGateOutput {
    QueryLiveliness { pattern: String },
    HoldSample,
    WriteWithSchema { type_name: String },
    WriteFallback,
    OpenSchemaChannel { type_name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ros2ddsGateInput {
    /// Only for samples that already passed [`is_ros2_schema_candidate`] at the tap.
    Sample {
        now_millis: u64,
        known_type: Option<String>,
    },
    LivelinessGetResult {
        token_keys: Vec<String>,
    },
    LivelinessTypePut {
        type_name: String,
    },
    LivelinessTypeDelete,
    TimerTick {
        now_millis: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TopicGateState {
    AwaitingGet {
        started_millis: u64,
        held_samples: usize,
    },
    Typed {
        type_name: String,
    },
    Fallback,
}

#[derive(Default)]
pub struct Ros2ddsGate {
    topics: BTreeMap<String, TopicGateState>,
}

impl Ros2ddsGate {
    /// `Sample` inputs must be samples the caller already filtered with [`is_ros2_schema_candidate`].
    pub fn on_input(&mut self, topic: &str, input: Ros2ddsGateInput) -> Vec<Ros2ddsGateOutput> {
        match input {
            Ros2ddsGateInput::Sample {
                now_millis,
                known_type,
            } => self.on_sample(topic, now_millis, known_type),
            Ros2ddsGateInput::LivelinessGetResult { token_keys } => {
                self.on_get_result(topic, token_keys)
            }
            Ros2ddsGateInput::LivelinessTypePut { type_name } => self.on_type_put(topic, type_name),
            Ros2ddsGateInput::LivelinessTypeDelete => self.on_type_delete(topic),
            Ros2ddsGateInput::TimerTick { now_millis } => self.on_timer(topic, now_millis),
        }
    }

    fn on_sample(
        &mut self,
        topic: &str,
        now_millis: u64,
        known_type: Option<String>,
    ) -> Vec<Ros2ddsGateOutput> {
        if let Some(type_name) = known_type {
            self.topics.insert(
                topic.to_string(),
                TopicGateState::Typed {
                    type_name: type_name.clone(),
                },
            );
            return vec![Ros2ddsGateOutput::WriteWithSchema { type_name }];
        }
        match self.topics.get(topic).cloned() {
            Some(TopicGateState::Typed { type_name }) => {
                vec![Ros2ddsGateOutput::WriteWithSchema { type_name }]
            }
            Some(TopicGateState::Fallback) => {
                vec![Ros2ddsGateOutput::WriteFallback]
            }
            Some(TopicGateState::AwaitingGet {
                started_millis,
                held_samples,
            }) => {
                let held_samples = held_samples + 1;
                let timed_out =
                    now_millis.saturating_sub(started_millis) >= ROS2DDS_HOLD_TIMEOUT_MS;
                if held_samples >= ROS2DDS_HOLD_MAX_SAMPLES || timed_out {
                    self.topics
                        .insert(topic.to_string(), TopicGateState::Fallback);
                    vec![Ros2ddsGateOutput::WriteFallback]
                } else {
                    self.topics.insert(
                        topic.to_string(),
                        TopicGateState::AwaitingGet {
                            started_millis,
                            held_samples,
                        },
                    );
                    vec![Ros2ddsGateOutput::HoldSample]
                }
            }
            None => {
                let pattern = ros2dds_liveliness_query_pattern(topic);
                self.topics.insert(
                    topic.to_string(),
                    TopicGateState::AwaitingGet {
                        started_millis: now_millis,
                        held_samples: 1,
                    },
                );
                vec![
                    Ros2ddsGateOutput::QueryLiveliness { pattern },
                    Ros2ddsGateOutput::HoldSample,
                ]
            }
        }
    }

    fn on_get_result(&mut self, topic: &str, token_keys: Vec<String>) -> Vec<Ros2ddsGateOutput> {
        let Some(TopicGateState::AwaitingGet { .. }) = self.topics.get(topic) else {
            return Vec::new();
        };
        let type_name = token_keys
            .iter()
            .find_map(|key| type_name_from_ros2dds_token(key));
        if let Some(type_name) = type_name {
            self.topics.insert(
                topic.to_string(),
                TopicGateState::Typed {
                    type_name: type_name.clone(),
                },
            );
            vec![Ros2ddsGateOutput::WriteWithSchema { type_name }]
        } else {
            self.topics
                .insert(topic.to_string(), TopicGateState::Fallback);
            vec![Ros2ddsGateOutput::WriteFallback]
        }
    }

    fn on_type_put(&mut self, topic: &str, type_name: String) -> Vec<Ros2ddsGateOutput> {
        match self.topics.get(topic).cloned() {
            Some(TopicGateState::Fallback) => {
                self.topics.insert(
                    topic.to_string(),
                    TopicGateState::Typed {
                        type_name: type_name.clone(),
                    },
                );
                vec![Ros2ddsGateOutput::OpenSchemaChannel { type_name }]
            }
            Some(TopicGateState::AwaitingGet { .. }) => {
                self.topics.insert(
                    topic.to_string(),
                    TopicGateState::Typed {
                        type_name: type_name.clone(),
                    },
                );
                vec![Ros2ddsGateOutput::WriteWithSchema { type_name }]
            }
            Some(TopicGateState::Typed { .. }) | None => {
                self.topics.insert(
                    topic.to_string(),
                    TopicGateState::Typed {
                        type_name: type_name.clone(),
                    },
                );
                Vec::new()
            }
        }
    }

    fn on_type_delete(&mut self, topic: &str) -> Vec<Ros2ddsGateOutput> {
        if matches!(self.topics.get(topic), Some(TopicGateState::Typed { .. })) {
            self.topics.remove(topic);
        }
        Vec::new()
    }

    fn on_timer(&mut self, topic: &str, now_millis: u64) -> Vec<Ros2ddsGateOutput> {
        let Some(TopicGateState::AwaitingGet {
            started_millis,
            held_samples,
        }) = self.topics.get(topic).cloned()
        else {
            return Vec::new();
        };
        if now_millis.saturating_sub(started_millis) < ROS2DDS_HOLD_TIMEOUT_MS {
            return Vec::new();
        }
        if held_samples == 0 {
            return Vec::new();
        }
        self.topics
            .insert(topic.to_string(), TopicGateState::Fallback);
        vec![Ros2ddsGateOutput::WriteFallback]
    }
}

/// Zenoh key expression to query ros2dds liveliness for a data key (slashes escaped as section sign).
pub fn ros2dds_liveliness_query_pattern(data_key: &str) -> String {
    let escaped = data_key.replace('/', "\u{a7}");
    format!("@/*/@ros2_lv/*/{escaped}/**")
}

fn type_name_from_ros2dds_token(token_key: &str) -> Option<String> {
    blueos_ros2_names::parse_ros2dds_liveliness_token(token_key).map(|info| info.type_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CDR_HEADER: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

    #[test]
    fn candidate_accepts_zenoh_bytes_with_cdr_header() {
        assert!(is_ros2_schema_candidate(
            "chatter",
            "zenoh/bytes",
            &CDR_HEADER
        ));
    }

    #[test]
    fn candidate_accepts_empty_encoding_with_cdr_header() {
        assert!(is_ros2_schema_candidate("chatter", "", &CDR_HEADER));
    }

    #[test]
    fn candidate_rejects_json_body_on_zenoh_bytes() {
        assert!(!is_ros2_schema_candidate(
            "chatter",
            "zenoh/bytes",
            b"{\"x\":1}"
        ));
    }

    #[test]
    fn candidate_rejects_blueos_prefix() {
        assert!(!is_ros2_schema_candidate(
            "blueos/v1/ping",
            "zenoh/bytes",
            &CDR_HEADER,
        ));
    }
}
