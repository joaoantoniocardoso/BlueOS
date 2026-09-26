use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use blueos_comms::{LivelinessEvent, Sample, Session};
use blueos_recorder_mavlink::{MavlinkIngressState, facts_from_frame, is_handled_message};
use blueos_recorder_mcap::{
    McapWriterHandle, channel_descriptor_cdr_fallback, channel_descriptor_for_ros2_type,
    channel_descriptor_for_sample,
};
use blueos_recorder_policy::{
    RAW_MAVLINK_OUT_TOPIC_PREFIX, Ros2ddsGate, Ros2ddsGateInput, Ros2ddsGateOutput, TapPolicy,
    is_ros2_schema_candidate,
};
use blueos_ros2_names::{
    parse_rmw_zenoh_data_key, parse_ros2dds_liveliness_token, ros2dds_liveliness_token_to_data_key,
};
use futures::StreamExt;
use mavlink_codec::PacketRef;
use tokio::sync::{mpsc, watch};
use tracing::{error, info, warn};

const FLUSH_POLL_SECONDS: u64 = 1;
const LIVELINESS_GET_TIMEOUT: Duration = Duration::from_millis(500);
const GATE_TICK_MS: u64 = 100;

struct HeldSample {
    log_time: u64,
    publish_time: u64,
    payload: blueos_comms::Payload,
}

struct TapRos2State {
    ros2dds_types: HashMap<String, String>,
    gate: Ros2ddsGate,
    held: HashMap<String, Vec<HeldSample>>,
    pending_gets: HashSet<String>,
    second_channel_logged: HashSet<String>,
    // ponytail: one verdict per topic from the first sample, never evicted
    schema_candidate_cache: HashMap<String, bool>,
}

impl TapRos2State {
    fn new() -> Self {
        Self {
            ros2dds_types: HashMap::new(),
            gate: Ros2ddsGate::default(),
            held: HashMap::new(),
            pending_gets: HashSet::new(),
            second_channel_logged: HashSet::new(),
            schema_candidate_cache: HashMap::new(),
        }
    }

    fn schema_candidate_cached(&mut self, topic: &str, encoding: &str, payload: &[u8]) -> bool {
        if let Some(verdict) = self.schema_candidate_cache.get(topic) {
            return *verdict;
        }
        let verdict = is_ros2_schema_candidate(topic, encoding, payload);
        self.schema_candidate_cache
            .insert(topic.to_string(), verdict);
        verdict
    }

    fn known_ros2dds_type(&self, topic: &str) -> Option<String> {
        self.ros2dds_types.get(topic).cloned()
    }

    fn apply_liveliness_put(&mut self, key: &str) {
        if let Some(data_key) = ros2dds_liveliness_token_to_data_key(key)
            && let Some(info) = parse_ros2dds_liveliness_token(key)
        {
            self.ros2dds_types.insert(data_key, info.type_name);
        }
    }

    fn apply_liveliness_delete(&mut self, key: &str) {
        if let Some(data_key) = ros2dds_liveliness_token_to_data_key(key) {
            self.ros2dds_types.remove(&data_key);
        }
    }
}

pub async fn run_data_plane(
    session: Session,
    policy_watch: watch::Receiver<TapPolicy>,
    writer: Arc<McapWriterHandle>,
    schema_path: Option<std::path::PathBuf>,
    fact_sender: mpsc::Sender<Vec<blueos_recorder_mavlink::MavlinkFact>>,
) {
    let mut ingress_state = MavlinkIngressState::default();
    let mut ros2_state = TapRos2State::new();
    let (get_result_sender, mut get_result_receiver) = mpsc::channel(32);

    let mut subscription = {
        let mut backoff_secs = 1u64;
        loop {
            match session.subscribe("**").await {
                Ok(subscription) => break subscription,
                Err(error) => {
                    error!(
                        %error,
                        retry_secs = backoff_secs,
                        "Global Zenoh subscribe failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    backoff_secs = backoff_secs.saturating_mul(2).min(30);
                }
            }
        }
    };

    let mut liveliness_stream = {
        let mut backoff_secs = 1u64;
        loop {
            match session.subscribe_liveliness("@/*/@ros2_lv/**").await {
                Ok(stream) => break stream,
                Err(error) => {
                    error!(
                        %error,
                        retry_secs = backoff_secs,
                        "ros2dds liveliness subscribe failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                    backoff_secs = backoff_secs.saturating_mul(2).min(30);
                }
            }
        }
    };

    let mut flush_interval = tokio::time::interval(Duration::from_secs(FLUSH_POLL_SECONDS));
    flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut gate_tick = tokio::time::interval(Duration::from_millis(GATE_TICK_MS));
    gate_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut liveliness_open = true;

    loop {
        tokio::select! {
            _ = flush_interval.tick() => {
                if let Err(error) = writer.flush() {
                    error!(%error, "MCAP periodic flush failed");
                }
            }
            _ = gate_tick.tick() => {
                let now_millis = now_millis();
                let topics: Vec<String> = ros2_state.held.keys().cloned().collect();
                for topic in topics {
                    let outputs = ros2_state.gate.on_input(
                        &topic,
                        Ros2ddsGateInput::TimerTick { now_millis },
                    );
                    apply_gate_outputs(
                        &topic,
                        outputs,
                        &mut ros2_state,
                        &writer,
                        schema_path.as_deref(),
                        &session,
                        &get_result_sender,
                    );
                }
            }
            event = liveliness_stream.next(), if liveliness_open => {
                let Some(event) = event else {
                    error!("ros2dds liveliness stream ended; late ROS 2 schemas will no longer resolve");
                    liveliness_open = false;
                    continue;
                };
                match event {
                    LivelinessEvent::Put { key } => {
                        ros2_state.apply_liveliness_put(&key);
                        if let Some(data_key) = ros2dds_liveliness_token_to_data_key(&key) {
                            let type_name = ros2_state.ros2dds_types.get(&data_key).cloned();
                            if let Some(type_name) = type_name {
                                let outputs = ros2_state.gate.on_input(
                                    &data_key,
                                    Ros2ddsGateInput::LivelinessTypePut { type_name },
                                );
                                apply_gate_outputs(
                                    &data_key,
                                    outputs,
                                    &mut ros2_state,
                                    &writer,
                                    schema_path.as_deref(),
                                    &session,
                                    &get_result_sender,
                                );
                            }
                        }
                    }
                    LivelinessEvent::Delete { key } => {
                        if let Some(data_key) = ros2dds_liveliness_token_to_data_key(&key) {
                            ros2_state.apply_liveliness_delete(&key);
                            let _outputs = ros2_state.gate.on_input(
                                &data_key,
                                Ros2ddsGateInput::LivelinessTypeDelete,
                            );
                        }
                    }
                }
            }
            result = get_result_receiver.recv() => {
                if let Some((topic, token_keys)) = result {
                    ros2_state.pending_gets.remove(&topic);
                    let outputs = ros2_state.gate.on_input(
                        &topic,
                        Ros2ddsGateInput::LivelinessGetResult { token_keys },
                    );
                    apply_gate_outputs(
                        &topic,
                        outputs,
                        &mut ros2_state,
                        &writer,
                        schema_path.as_deref(),
                        &session,
                        &get_result_sender,
                    );
                }
            }
            sample = subscription.next() => {
                let Some(sample) = sample else {
                    break;
                };
                handle_sample(
                    &sample,
                    &policy_watch,
                    &writer,
                    schema_path.as_deref(),
                    &mut ingress_state,
                    &fact_sender,
                    &mut ros2_state,
                    &session,
                    &get_result_sender,
                );
            }
        }
    }
    error!("Recorder data-plane tap exited");
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn schema_lookup() -> impl Fn(&str) -> Option<String> + Clone {
    |name: &str| {
        blueos_idl::schema(name)
            .or_else(|| blueos_idl::catalog::schema(name))
            .map(str::to_string)
    }
}

fn write_with_descriptor(
    writer: &McapWriterHandle,
    topic: &str,
    schema_lane: Option<&str>,
    log_time: u64,
    publish_time: u64,
    payload: blueos_comms::Payload,
    descriptor: Option<blueos_recorder_mcap::ChannelDescriptor>,
) {
    if let Err(error) = writer.write_message(
        topic,
        schema_lane,
        log_time,
        publish_time,
        payload,
        descriptor,
    ) {
        error!(%error, "Failed to write MCAP message");
    }
}

fn resolve_descriptor(
    topic: &str,
    encoding: &str,
    payload: &blueos_comms::Payload,
    schema_path: Option<&Path>,
    ros2_state: &TapRos2State,
) -> Option<(
    Option<String>,
    Option<blueos_recorder_mcap::ChannelDescriptor>,
)> {
    let lookup = schema_lookup();
    if let Some(descriptor) =
        channel_descriptor_for_sample(topic, encoding, payload, &lookup, schema_path)
    {
        return Some((None, Some(descriptor)));
    }
    if let Some(parsed) = parse_rmw_zenoh_data_key(topic) {
        return Some(ros2_lane_and_descriptor(
            topic,
            &parsed.type_name,
            schema_path,
        ));
    }
    if let Some(type_name) = ros2_state.known_ros2dds_type(topic) {
        return Some(ros2_lane_and_descriptor(topic, &type_name, schema_path));
    }
    None
}

/// The typed lane for a ROS 2 type, or the schema-less CDR fallback lane when no schema text is known.
fn ros2_lane_and_descriptor(
    topic: &str,
    type_name: &str,
    schema_path: Option<&Path>,
) -> (
    Option<String>,
    Option<blueos_recorder_mcap::ChannelDescriptor>,
) {
    match channel_descriptor_for_ros2_type(topic, type_name, &schema_lookup(), schema_path) {
        Some(descriptor) => (Some(type_name.to_string()), Some(descriptor)),
        None => (None, Some(channel_descriptor_cdr_fallback(topic))),
    }
}

fn flush_held_topic(
    topic: &str,
    ros2_state: &mut TapRos2State,
    writer: &McapWriterHandle,
    schema_path: Option<&Path>,
    schema_lane: Option<&str>,
    use_fallback: bool,
) {
    let held = ros2_state.held.remove(topic).unwrap_or_default();
    for sample in held {
        let (lane, descriptor) = match schema_lane {
            Some(type_name) if !use_fallback => {
                ros2_lane_and_descriptor(topic, type_name, schema_path)
            }
            _ => (None, Some(channel_descriptor_cdr_fallback(topic))),
        };
        write_with_descriptor(
            writer,
            topic,
            lane.as_deref(),
            sample.log_time,
            sample.publish_time,
            sample.payload,
            descriptor,
        );
    }
}

fn apply_gate_outputs(
    topic: &str,
    outputs: Vec<Ros2ddsGateOutput>,
    ros2_state: &mut TapRos2State,
    writer: &McapWriterHandle,
    schema_path: Option<&Path>,
    session: &Session,
    get_result_sender: &mpsc::Sender<(String, Vec<String>)>,
) {
    for output in outputs {
        match output {
            Ros2ddsGateOutput::QueryLiveliness { pattern } => {
                if ros2_state.pending_gets.insert(topic.to_string()) {
                    let session = session.clone();
                    let topic_owned = topic.to_string();
                    let sender = get_result_sender.clone();
                    tokio::spawn(async move {
                        let token_keys = session
                            .get_liveliness(&pattern, LIVELINESS_GET_TIMEOUT)
                            .await
                            .unwrap_or_default();
                        let _ = sender.send((topic_owned, token_keys)).await;
                    });
                }
            }
            Ros2ddsGateOutput::HoldSample => {}
            Ros2ddsGateOutput::WriteFallback => {
                flush_held_topic(topic, ros2_state, writer, schema_path, None, true);
            }
            Ros2ddsGateOutput::WriteWithSchema { type_name } => {
                flush_held_topic(
                    topic,
                    ros2_state,
                    writer,
                    schema_path,
                    Some(type_name.as_str()),
                    false,
                );
            }
            Ros2ddsGateOutput::OpenSchemaChannel { type_name } => {
                if ros2_state.second_channel_logged.insert(topic.to_string()) {
                    info!(
                        topic,
                        type_name, "Opened second MCAP channel with ROS 2 schema after fallback"
                    );
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_sample(
    sample: &Sample,
    policy_watch: &watch::Receiver<TapPolicy>,
    writer: &McapWriterHandle,
    schema_path: Option<&Path>,
    ingress_state: &mut MavlinkIngressState,
    fact_sender: &mpsc::Sender<Vec<blueos_recorder_mavlink::MavlinkFact>>,
    ros2_state: &mut TapRos2State,
    session: &Session,
    get_result_sender: &mpsc::Sender<(String, Vec<String>)>,
) {
    let topic = sample.key.as_str();

    let policy = policy_watch.borrow().clone();

    if topic.starts_with(RAW_MAVLINK_OUT_TOPIC_PREFIX) {
        let bytes = sample.payload.to_vec();
        if let Some(packet) = PacketRef::new(bytes.as_slice())
            && is_handled_message(packet.message_id())
        {
            let facts = facts_from_frame(ingress_state, bytes.as_slice());
            if !facts.is_empty()
                && let Err(error) = fact_sender.try_send(facts)
            {
                warn!(%error, "Dropped MAVLink facts (fact channel full)");
            }
        }
    }

    if !policy.should_record_topic(topic) {
        return;
    }

    let now = SystemTime::now();
    let log_time = now
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let publish_time = sample.timestamp.unwrap_or(log_time);

    if let Some((schema_lane, descriptor)) = resolve_descriptor(
        topic,
        &sample.encoding,
        &sample.payload,
        schema_path,
        ros2_state,
    ) {
        write_with_descriptor(
            writer,
            topic,
            schema_lane.as_deref(),
            log_time,
            publish_time,
            sample.payload.clone(),
            descriptor,
        );
        return;
    }

    let is_schema_candidate =
        ros2_state.schema_candidate_cached(topic, &sample.encoding, &sample.payload.as_slice());
    if !is_schema_candidate {
        write_with_descriptor(
            writer,
            topic,
            None,
            log_time,
            publish_time,
            sample.payload.clone(),
            None,
        );
        return;
    }

    let known_type = ros2_state.known_ros2dds_type(topic);
    let outputs = ros2_state.gate.on_input(
        topic,
        Ros2ddsGateInput::Sample {
            now_millis: now_millis(),
            known_type,
        },
    );

    ros2_state
        .held
        .entry(topic.to_string())
        .or_default()
        .push(HeldSample {
            log_time,
            publish_time,
            payload: sample.payload.clone(),
        });

    apply_gate_outputs(
        topic,
        outputs,
        ros2_state,
        writer,
        schema_path,
        session,
        get_result_sender,
    );
}
