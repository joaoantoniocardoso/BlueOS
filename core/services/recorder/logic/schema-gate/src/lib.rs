//! Sans-IO gate for ros2dds topics whose schema arrives after the first sample (D-24, D-15).

#![no_std]

extern crate alloc;

pub mod held;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use blueos_ros2_names::parse_ros2dds_liveliness_token;

use held::HeldSample;

/// Maximum samples held per topic while waiting for a liveliness token.
pub const ROS2DDS_HOLD_MAX_SAMPLES: usize = 64;

/// Maximum time to hold samples per topic while waiting for a liveliness token.
pub const ROS2DDS_HOLD_TIMEOUT_MS: u64 = 2_000;

const MAVLINK_RAW_TOPIC_PREFIX: &str = "mavlink_raw/";
const MAVLINK_TOPIC_PREFIX: &str = "mavlink/";
const VIDEO_TOPIC_PREFIX: &str = "video/";

/// Effects the data plane should run after one gate input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ros2ddsGateOutput {
    /// Issue a liveliness `get` on the router for `pattern`.
    QueryLiveliness {
        /// Zenoh key expression for the get.
        pattern: String,
    },
    /// Drain held samples and write them (`type_name` None uses the fallback lane).
    ReleaseHeld {
        /// Resolved ROS type, when known.
        type_name: Option<String>,
    },
}

/// Inputs that drive the gate state machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ros2ddsGateInput<Payload> {
    /// A ros2dds candidate sample (caller filters with [`is_ros2_schema_candidate`]).
    Sample {
        /// Monotonic millis from the service injected clock.
        now_monotonic_millis: u64,
        /// MCAP log time in nanoseconds.
        log_time: u64,
        /// MCAP publish time in nanoseconds.
        publish_time: u64,
        /// Sample bytes (held by reference).
        payload: Payload,
    },
    /// Result of a liveliness `get` for this data key.
    LivelinessGetResult {
        /// Alive token keys returned by the router.
        token_keys: Vec<String>,
    },
    /// A ros2dds publisher token was declared for this data key.
    LivelinessTypePut {
        /// ROS 2 type name from the token.
        type_name: String,
    },
    /// The ros2dds publisher token for this data key was dropped.
    LivelinessTypeDelete,
    /// Periodic tick from the data plane (paused clock in tests).
    TimerTick {
        /// Monotonic millis from the injected clock.
        now_monotonic_millis: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TopicGateState {
    AwaitingGet {
        started_monotonic_millis: u64,
        held_samples: usize,
    },
    Typed {
        type_name: String,
    },
    Fallback,
}

/// Per-topic ros2dds schema gate with a bounded sample queue (payloads by reference).
pub struct Ros2ddsGate<Payload> {
    topics: BTreeMap<String, TopicGateState>,
    held: BTreeMap<String, Vec<HeldSample<Payload>>>,
}

impl<Payload> Default for Ros2ddsGate<Payload> {
    fn default() -> Self {
        Self {
            topics: BTreeMap::new(),
            held: BTreeMap::new(),
        }
    }
}

impl<Payload> Ros2ddsGate<Payload> {
    /// Topics currently waiting for a liveliness token (for timeout ticks).
    pub fn topics_awaiting_timer(&self) -> Vec<String> {
        self.topics
            .iter()
            .filter_map(|(topic, state)| {
                if matches!(state, TopicGateState::AwaitingGet { .. }) {
                    Some(topic.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Resolved ROS 2 type name when the gate has left the awaiting/fallback-only path.
    pub fn type_name(&self, topic: &str) -> Option<&str> {
        match self.topics.get(topic) {
            Some(TopicGateState::Typed { type_name }) => Some(type_name.as_str()),
            _ => None,
        }
    }

    /// Whether this topic is already in the gate state machine (awaiting, typed, or fallback).
    pub fn tracks_topic(&self, topic: &str) -> bool {
        self.topics.contains_key(topic)
    }

    /// Drops held samples and per-topic state (call when a recording file closes).
    pub fn clear(&mut self) {
        self.topics.clear();
        self.held.clear();
    }

    /// Applies one input for `topic` and returns outputs for the data plane.
    pub fn on_input(
        &mut self,
        topic: &str,
        input: Ros2ddsGateInput<Payload>,
    ) -> Vec<Ros2ddsGateOutput> {
        match input {
            Ros2ddsGateInput::Sample {
                now_monotonic_millis,
                log_time,
                publish_time,
                payload,
            } => self.on_sample(
                topic,
                now_monotonic_millis,
                HeldSample {
                    log_time,
                    publish_time,
                    payload,
                },
            ),
            Ros2ddsGateInput::LivelinessGetResult { token_keys } => {
                self.on_get_result(topic, token_keys)
            }
            Ros2ddsGateInput::LivelinessTypePut { type_name } => self.on_type_put(topic, type_name),
            Ros2ddsGateInput::LivelinessTypeDelete => {
                self.on_type_delete(topic);
                Vec::new()
            }
            Ros2ddsGateInput::TimerTick {
                now_monotonic_millis,
            } => self.on_timer(topic, now_monotonic_millis),
        }
    }

    fn on_sample(
        &mut self,
        topic: &str,
        now_monotonic_millis: u64,
        sample: HeldSample<Payload>,
    ) -> Vec<Ros2ddsGateOutput> {
        match self.topics.get(topic).cloned() {
            Some(TopicGateState::Typed { type_name }) => {
                self.held.entry(topic.to_string()).or_default().push(sample);
                vec![Ros2ddsGateOutput::ReleaseHeld {
                    type_name: Some(type_name),
                }]
            }
            Some(TopicGateState::Fallback) => {
                self.held.entry(topic.to_string()).or_default().push(sample);
                vec![Ros2ddsGateOutput::ReleaseHeld { type_name: None }]
            }
            Some(TopicGateState::AwaitingGet {
                started_monotonic_millis,
                held_samples,
            }) => {
                self.held.entry(topic.to_string()).or_default().push(sample);
                let held_samples = held_samples + 1;
                let timed_out = now_monotonic_millis.saturating_sub(started_monotonic_millis)
                    >= ROS2DDS_HOLD_TIMEOUT_MS;
                if held_samples >= ROS2DDS_HOLD_MAX_SAMPLES || timed_out {
                    self.topics
                        .insert(topic.to_string(), TopicGateState::Fallback);
                    vec![Ros2ddsGateOutput::ReleaseHeld { type_name: None }]
                } else {
                    self.topics.insert(
                        topic.to_string(),
                        TopicGateState::AwaitingGet {
                            started_monotonic_millis,
                            held_samples,
                        },
                    );
                    Vec::new()
                }
            }
            None => {
                self.held.entry(topic.to_string()).or_default().push(sample);
                let pattern = ros2dds_liveliness_query_pattern(topic);
                self.topics.insert(
                    topic.to_string(),
                    TopicGateState::AwaitingGet {
                        started_monotonic_millis: now_monotonic_millis,
                        held_samples: 1,
                    },
                );
                vec![Ros2ddsGateOutput::QueryLiveliness { pattern }]
            }
        }
    }

    fn on_get_result(&mut self, topic: &str, token_keys: Vec<String>) -> Vec<Ros2ddsGateOutput> {
        let Some(TopicGateState::AwaitingGet { .. }) = self.topics.get(topic) else {
            return Vec::new();
        };
        let type_name = token_keys
            .iter()
            .find_map(|key| parse_ros2dds_liveliness_token(key).map(|info| info.type_name));
        if let Some(type_name) = type_name {
            self.topics.insert(
                topic.to_string(),
                TopicGateState::Typed {
                    type_name: type_name.clone(),
                },
            );
            vec![Ros2ddsGateOutput::ReleaseHeld {
                type_name: Some(type_name),
            }]
        } else {
            self.topics
                .insert(topic.to_string(), TopicGateState::Fallback);
            vec![Ros2ddsGateOutput::ReleaseHeld { type_name: None }]
        }
    }

    fn on_type_put(&mut self, topic: &str, type_name: String) -> Vec<Ros2ddsGateOutput> {
        match self.topics.get(topic).cloned() {
            Some(TopicGateState::Fallback) | Some(TopicGateState::AwaitingGet { .. }) => {
                self.topics.insert(
                    topic.to_string(),
                    TopicGateState::Typed {
                        type_name: type_name.clone(),
                    },
                );
                if self.held.get(topic).is_some_and(|queue| !queue.is_empty()) {
                    return vec![Ros2ddsGateOutput::ReleaseHeld {
                        type_name: Some(type_name),
                    }];
                }
                Vec::new()
            }
            Some(TopicGateState::Typed { .. }) | None => {
                self.topics
                    .insert(topic.to_string(), TopicGateState::Typed { type_name });
                Vec::new()
            }
        }
    }

    fn on_type_delete(&mut self, topic: &str) {
        if matches!(self.topics.get(topic), Some(TopicGateState::Typed { .. })) {
            self.topics.remove(topic);
        }
    }

    fn on_timer(&mut self, topic: &str, now_monotonic_millis: u64) -> Vec<Ros2ddsGateOutput> {
        let Some(TopicGateState::AwaitingGet {
            started_monotonic_millis,
            ..
        }) = self.topics.get(topic).cloned()
        else {
            return Vec::new();
        };
        if now_monotonic_millis.saturating_sub(started_monotonic_millis) < ROS2DDS_HOLD_TIMEOUT_MS {
            return Vec::new();
        }
        self.topics
            .insert(topic.to_string(), TopicGateState::Fallback);
        vec![Ros2ddsGateOutput::ReleaseHeld { type_name: None }]
    }

    /// Removes and returns held samples for `topic` after a [`Ros2ddsGateOutput::ReleaseHeld`] output.
    pub fn drain_held(&mut self, topic: &str) -> Vec<HeldSample<Payload>> {
        self.held.remove(topic).unwrap_or_default()
    }
}

/// Whether schema resolution may consult ROS 2 transports for this sample (D-24).
pub fn is_ros2_schema_candidate(topic: &str, encoding: &str, payload_prefix: &[u8]) -> bool {
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
    has_cdr_encapsulation_header(payload_prefix)
}

/// Zenoh key expression to query ros2dds liveliness for a data key (slashes escaped as section sign).
pub fn ros2dds_liveliness_query_pattern(data_key: &str) -> String {
    let escaped = data_key.replace('/', "\u{a7}");
    format!("@/*/@ros2_lv/*/{escaped}/**")
}

fn has_cdr_encapsulation_header(payload: &[u8]) -> bool {
    payload.len() >= 4
        && payload[0] == 0x00
        && (payload[1] == 0x00 || payload[1] == 0x01)
        && payload[2] == 0x00
        && payload[3] == 0x00
}

fn ros2_wire_encoding_base(encoding: &str) -> bool {
    if encoding.is_empty() || encoding == "zenoh/bytes" {
        return true;
    }
    let mut parts = encoding.split(';');
    let base = parts.next().unwrap_or(encoding);
    base == "application/cdr" && parts.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CDR_HEADER: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

    fn sample_input(payload: u8) -> Ros2ddsGateInput<u8> {
        Ros2ddsGateInput::Sample {
            now_monotonic_millis: 0,
            log_time: 1,
            publish_time: 1,
            payload,
        }
    }

    #[test]
    fn candidate_accepts_zenoh_bytes_with_cdr_header() {
        assert!(is_ros2_schema_candidate(
            "chatter",
            "zenoh/bytes",
            &CDR_HEADER
        ));
    }

    #[test]
    fn sample_before_token_flushes_with_schema_when_token_arrives_within_two_seconds() {
        let mut gate = Ros2ddsGate::<u8>::default();
        let outputs = gate.on_input("chatter", sample_input(42));
        assert_eq!(outputs.len(), 1);
        assert!(matches!(
            outputs[0],
            Ros2ddsGateOutput::QueryLiveliness { .. }
        ));

        let schema_outputs = gate.on_input(
            "chatter",
            Ros2ddsGateInput::LivelinessTypePut {
                type_name: "std_msgs/msg/String".into(),
            },
        );
        assert_eq!(schema_outputs.len(), 1);
        match &schema_outputs[0] {
            Ros2ddsGateOutput::ReleaseHeld {
                type_name: Some(type_name),
            } => {
                assert_eq!(type_name, "std_msgs/msg/String");
                let samples = gate.drain_held("chatter");
                assert_eq!(samples.len(), 1);
                assert_eq!(samples[0].payload, 42);
            }
            other => panic!("expected ReleaseHeld, got {other:?}"),
        }
    }

    #[test]
    fn timeout_produces_fallback_then_typed_lane_on_later_token_and_sample() {
        let mut gate = Ros2ddsGate::<u8>::default();
        let _ = gate.on_input("chatter", sample_input(1));
        let outputs = gate.on_input(
            "chatter",
            Ros2ddsGateInput::TimerTick {
                now_monotonic_millis: ROS2DDS_HOLD_TIMEOUT_MS,
            },
        );
        assert!(matches!(
            outputs[0],
            Ros2ddsGateOutput::ReleaseHeld { type_name: None }
        ));
        let _ = gate.drain_held("chatter");

        let _ = gate.on_input(
            "chatter",
            Ros2ddsGateInput::LivelinessTypePut {
                type_name: "std_msgs/msg/String".into(),
            },
        );
        assert_eq!(gate.type_name("chatter"), Some("std_msgs/msg/String"));

        let flush = gate.on_input("chatter", sample_input(2));
        assert!(matches!(
            flush[0],
            Ros2ddsGateOutput::ReleaseHeld { type_name: Some(_) }
        ));
    }

    #[test]
    fn unknown_type_after_fallback_still_flushes_held_samples() {
        let mut gate = Ros2ddsGate::<u8>::default();
        let _ = gate.on_input("chatter", sample_input(1));
        let timeout_outputs = gate.on_input(
            "chatter",
            Ros2ddsGateInput::TimerTick {
                now_monotonic_millis: ROS2DDS_HOLD_TIMEOUT_MS,
            },
        );
        assert!(matches!(
            timeout_outputs[0],
            Ros2ddsGateOutput::ReleaseHeld { type_name: None }
        ));
        let _ = gate.drain_held("chatter");
        let outputs = gate.on_input("chatter", sample_input(9));
        match &outputs[0] {
            Ros2ddsGateOutput::ReleaseHeld { type_name: None } => {
                let samples = gate.drain_held("chatter");
                assert_eq!(samples.len(), 1);
                assert_eq!(samples[0].payload, 9);
            }
            other => panic!("expected ReleaseHeld, got {other:?}"),
        }
    }

    #[test]
    fn queue_holds_payload_by_reference() {
        let mut gate = Ros2ddsGate::<&'static u8>::default();
        let payload: &'static u8 = &7;
        let _ = gate.on_input(
            "chatter",
            Ros2ddsGateInput::Sample {
                now_monotonic_millis: 0,
                log_time: 0,
                publish_time: 0,
                payload,
            },
        );
        let outputs = gate.on_input(
            "chatter",
            Ros2ddsGateInput::LivelinessTypePut {
                type_name: "std_msgs/msg/String".into(),
            },
        );
        match &outputs[0] {
            Ros2ddsGateOutput::ReleaseHeld {
                type_name: Some(type_name),
            } => {
                assert_eq!(type_name, "std_msgs/msg/String");
                let samples = gate.drain_held("chatter");
                assert!(core::ptr::eq(samples[0].payload, payload));
            }
            other => panic!("expected ReleaseHeld, got {other:?}"),
        }
    }

    #[test]
    fn clear_drops_held_samples_and_state() {
        let mut gate = Ros2ddsGate::<u8>::default();
        let _ = gate.on_input("chatter", sample_input(1));
        assert!(gate.tracks_topic("chatter"));
        gate.clear();
        assert!(!gate.tracks_topic("chatter"));
        assert!(gate.drain_held("chatter").is_empty());
    }
}
