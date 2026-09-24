use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use blueos_comms::{Sample, Session};
use blueos_recorder_mavlink::{MavlinkIngressState, facts_from_frame, is_handled_message};
use blueos_recorder_mcap::{McapWriterHandle, channel_descriptor_for_sample};
use blueos_recorder_policy::{RAW_MAVLINK_OUT_TOPIC_PREFIX, TapPolicy};
use futures::StreamExt;
use mavlink_codec::PacketRef;
use tokio::sync::{mpsc, watch};
use tracing::{error, warn};

use crate::schema::embedded_ros_schema;

const FLUSH_POLL_SECONDS: u64 = 1;

pub async fn run_data_plane(
    session: Session,
    policy_watch: watch::Receiver<TapPolicy>,
    writer: Arc<McapWriterHandle>,
    schema_path: Option<std::path::PathBuf>,
    fact_sender: mpsc::Sender<Vec<blueos_recorder_mavlink::MavlinkFact>>,
) {
    let mut ingress_state = MavlinkIngressState::default();
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
                    tokio::time::sleep(std::time::Duration::from_secs(backoff_secs)).await;
                    backoff_secs = backoff_secs.saturating_mul(2).min(30);
                }
            }
        }
    };

    let mut flush_interval =
        tokio::time::interval(std::time::Duration::from_secs(FLUSH_POLL_SECONDS));
    flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = flush_interval.tick() => {
                if let Err(error) = writer.flush() {
                    error!(%error, "MCAP periodic flush failed");
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
                );
            }
        }
    }
    error!("Recorder data-plane tap exited");
}

fn handle_sample(
    sample: &Sample,
    policy_watch: &watch::Receiver<TapPolicy>,
    writer: &McapWriterHandle,
    schema_path: Option<&Path>,
    ingress_state: &mut MavlinkIngressState,
    fact_sender: &mpsc::Sender<Vec<blueos_recorder_mavlink::MavlinkFact>>,
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

    let lookup = |name: &str| embedded_ros_schema(name);
    let descriptor = channel_descriptor_for_sample(
        topic,
        &sample.encoding,
        &sample.payload,
        &lookup,
        schema_path,
    );

    if let Err(error) = writer.write_message(
        topic,
        log_time,
        publish_time,
        sample.payload.clone(),
        descriptor,
    ) {
        error!(%error, "Failed to write MCAP message");
    }
}
