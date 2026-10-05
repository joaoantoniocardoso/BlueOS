//! Data plane Task: owns the MCAP writer actor, follows [`RecordGate`], reports Observed facts.

mod reconcile;
mod run;
mod sample_plan;

pub(crate) use run::run_data_plane;

use reconcile::{cleanup, reconcile};

use core::time::Duration;
use std::{collections::BTreeMap, collections::BTreeSet, sync::Arc};

use tokio::task::JoinSet;
use tracing::warn;

use blueos_comms::{LivelinessEvent, Payload, Sample};
use blueos_domain::Command;
use blueos_recorder_capture::{Capture, CaptureObservedFact, RecordGate};
use blueos_recorder_domain::{RecorderDomain, RecorderObservedFact};
use blueos_recorder_mcap::{
    ChannelDescriptor, ChannelRoute, McapWriterHandle, WriteSampleRequest, ros2_lane_descriptor,
};
use blueos_recorder_schema_gate::{
    Ros2ddsGate, Ros2ddsGateInput, Ros2ddsGateOutput, held::HeldSample,
};
use blueos_recorder_storage::RecordingsFolder;
use blueos_ros2_names::{parse_ros2dds_liveliness_token, ros2dds_liveliness_token_to_data_key};
use blueos_service::{CommandSender, TaskContext, TaskFailed};

use crate::context::RecorderContext;

use self::sample_plan::{SampleWritePlan, plan_sample_write};

pub(super) const BYTES_REPORT_INTERVAL: Duration = Duration::from_secs(1);
// ponytail: fixed 100 ms tick scans awaiting topics; sleep-until-earliest-deadline would skip idle wakeups.
pub(super) const GATE_TICK_INTERVAL: Duration = Duration::from_millis(100);
const LIVELINESS_GET_TIMEOUT: Duration = Duration::from_millis(500);

pub(super) type LivelinessGetOutcome = Result<Vec<String>, blueos_comms::CommsError>;

/// Metadata for the file the writer actor has open.
pub(super) struct OpenRecording {
    file_generation: u64,
    file_name: String,
}

/// Local state the Domain does not own.
pub(super) struct DataPlaneLocal {
    open: Option<OpenRecording>,
    recording_video_topics: BTreeSet<String>,
    descriptors: BTreeMap<ChannelRoute, Arc<ChannelDescriptor>>,
    last_reported_bytes: u64,
    /// Samples the writer dropped from the open file so far.
    samples_dropped: u64,
    ros2_gate: Ros2ddsGate<Payload>,
    pending_liveliness_gets: BTreeSet<String>,
}

pub(super) struct DataPlaneHandle<'a> {
    folder: &'a Arc<RecordingsFolder>,
    writer: &'a Arc<McapWriterHandle>,
    task_context: &'a TaskContext<RecorderDomain, RecorderContext>,
    liveliness_gets: &'a mut JoinSet<(String, LivelinessGetOutcome)>,
}

pub(super) async fn handle_liveliness_event(
    event: LivelinessEvent,
    local: &mut DataPlaneLocal,
    handle: &mut DataPlaneHandle<'_>,
) {
    match event {
        LivelinessEvent::Put { key } => {
            if let Some(data_key) = ros2dds_liveliness_token_to_data_key(&key)
                && let Some(info) = parse_ros2dds_liveliness_token(&key)
            {
                let outputs = local.ros2_gate.on_input(
                    &data_key,
                    Ros2ddsGateInput::LivelinessTypePut {
                        type_name: info.type_name,
                    },
                );
                apply_gate_outputs(&data_key, outputs, local, handle).await;
            }
        }
        LivelinessEvent::Delete { key } => {
            if let Some(data_key) = ros2dds_liveliness_token_to_data_key(&key) {
                local
                    .ros2_gate
                    .on_input(&data_key, Ros2ddsGateInput::LivelinessTypeDelete);
            }
        }
    }
}

pub(super) async fn handle_sample(
    sample: &Sample,
    gate: &mut tokio::sync::watch::Receiver<RecordGate>,
    local: &mut DataPlaneLocal,
    handle: &mut DataPlaneHandle<'_>,
) -> Result<(), TaskFailed> {
    let gate_snapshot = gate.borrow().clone();
    if !gate_snapshot.recording_requested {
        return Ok(());
    }
    if !Capture::should_record_sample(sample.key(), &gate_snapshot) {
        return Ok(());
    }
    if local
        .open
        .as_ref()
        .is_some_and(|open| open.file_generation != gate_snapshot.desired_file_generation)
    {
        reconcile(&gate_snapshot, local, handle).await?;
    }
    if local.open.is_none() {
        return Ok(());
    }

    let topic = sample.key();
    let wall = handle.task_context.clock.now().wall;
    let log_time = sample_log_time_nanos(sample, wall);
    let plan = plan_sample_write(
        topic,
        sample.encoding(),
        sample.payload(),
        &local.ros2_gate,
        &mut local.descriptors,
    );
    apply_sample_write_plan(sample, log_time, plan, local, handle).await;
    Ok(())
}

async fn apply_sample_write_plan(
    sample: &Sample,
    log_time: u64,
    plan: SampleWritePlan,
    local: &mut DataPlaneLocal,
    handle: &mut DataPlaneHandle<'_>,
) {
    let topic = sample.key();
    match plan {
        SampleWritePlan::Ready { route, descriptor } => {
            handle.writer.try_write_sample(WriteSampleRequest {
                topic: topic.to_owned(),
                route,
                log_time,
                publish_time: log_time,
                payload: sample.payload().clone(),
                descriptor,
            });
        }
        SampleWritePlan::Skip => {}
        SampleWritePlan::NeedsRos2Gate => {
            let outputs = local.ros2_gate.on_input(
                topic,
                Ros2ddsGateInput::Sample {
                    now_monotonic_millis: monotonic_millis(handle.task_context),
                    log_time,
                    publish_time: log_time,
                    payload: sample.payload().clone(),
                },
            );
            apply_gate_outputs(topic, outputs, local, handle).await;
        }
    }
}

pub(super) async fn apply_gate_outputs(
    topic: &str,
    outputs: Vec<Ros2ddsGateOutput>,
    local: &mut DataPlaneLocal,
    handle: &mut DataPlaneHandle<'_>,
) {
    if local.open.is_none() {
        return;
    }
    for output in outputs {
        match output {
            Ros2ddsGateOutput::QueryLiveliness { pattern } => {
                if local.pending_liveliness_gets.insert(topic.to_owned()) {
                    let session = Arc::clone(&handle.task_context.session);
                    let topic_owned = topic.to_owned();
                    handle.liveliness_gets.spawn(async move {
                        let result = session
                            .get_liveliness(&pattern, LIVELINESS_GET_TIMEOUT)
                            .await;
                        (topic_owned, result)
                    });
                }
            }
            Ros2ddsGateOutput::ReleaseHeld { type_name } => {
                let samples = local.ros2_gate.drain_held(topic);
                let (route, descriptor) =
                    ros2_lane_descriptor(topic, type_name.as_deref(), &mut local.descriptors);
                write_held_samples(handle.writer, topic, route, descriptor, samples);
            }
        }
    }
}

fn write_held_samples(
    writer: &McapWriterHandle,
    topic: &str,
    route: ChannelRoute,
    descriptor: Arc<ChannelDescriptor>,
    samples: Vec<HeldSample<Payload>>,
) {
    samples.into_iter().for_each(|held| {
        writer.try_write_sample(WriteSampleRequest {
            topic: topic.to_owned(),
            route: route.clone(),
            log_time: held.log_time,
            publish_time: held.publish_time,
            payload: held.payload,
            descriptor: Arc::clone(&descriptor),
        });
    });
}

pub(super) fn monotonic_millis(task_context: &TaskContext<RecorderDomain, RecorderContext>) -> u64 {
    task_context
        .clock
        .now()
        .monotonic
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

pub(super) async fn report_bytes_if_due(
    local: &mut DataPlaneLocal,
    writer: &Arc<McapWriterHandle>,
    commands: &CommandSender<RecorderDomain>,
) {
    let dropped = writer.take_dropped_samples();
    if dropped > 0 {
        warn!(
            dropped,
            "MCAP writer queue dropped samples under back pressure"
        );
        local.samples_dropped = local.samples_dropped.saturating_add(dropped);
        if let Some(open) = &local.open {
            let _ = send_observed(
                commands,
                CaptureObservedFact::RecordingSamplesDropped {
                    file_generation: open.file_generation,
                    samples: local.samples_dropped,
                },
            )
            .await;
        }
    }
    let Some(open) = &local.open else {
        return;
    };
    let bytes = writer.bytes_written();
    if bytes == local.last_reported_bytes {
        return;
    }
    local.last_reported_bytes = bytes;
    let _ = send_observed(
        commands,
        CaptureObservedFact::RecordingBytesWritten {
            file_generation: open.file_generation,
            bytes,
        },
    )
    .await;
}

pub(super) async fn send_observed(
    commands: &CommandSender<RecorderDomain>,
    fact: CaptureObservedFact,
) -> Result<(), TaskFailed> {
    commands
        .send(Command::ObservedFact(RecorderObservedFact::Capture(fact)))
        .await
        .map_err(|error| {
            warn!(%error, "failed to deliver observed fact to the Inbox");
            TaskFailed
        })
}

fn sample_log_time_nanos(sample: &Sample, wall: Duration) -> u64 {
    sample
        .timestamp()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or_else(|| wall.as_nanos() as u64)
}
