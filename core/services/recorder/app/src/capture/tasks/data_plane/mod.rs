//! Data plane Task: owns the MCAP writer actor, follows [`RecordGate`], reports Observed facts.

mod sample_plan;

use core::time::Duration;
use std::{collections::BTreeMap, collections::BTreeSet, sync::Arc};

use tokio::{
    task::JoinSet,
    time::{MissedTickBehavior, interval},
};
use tracing::warn;

use blueos_comms::{LivelinessEvent, Payload, Sample};
use blueos_domain::Command;
use blueos_recorder_capture::{Capture, CaptureObservedFact, RecordGate};
use blueos_recorder_domain::{RecorderDomain, RecorderObservedFact};
use blueos_recorder_mcap::{
    ChannelDescriptor, ChannelRoute, McapWriterHandle, ros2_lane_descriptor,
};
use blueos_recorder_schema_gate::{
    Ros2ddsGate, Ros2ddsGateInput, Ros2ddsGateOutput, held::HeldSample,
};
use blueos_recorder_storage::RecordingsFolder;
use blueos_ros2_names::{parse_ros2dds_liveliness_token, ros2dds_liveliness_token_to_data_key};
use blueos_service::{CommandSender, TaskContext, TaskFailed};

use crate::context::RecorderContext;

use self::sample_plan::{SampleWritePlan, plan_sample_write};

const BYTES_REPORT_INTERVAL: Duration = Duration::from_secs(1);
// ponytail: fixed 100 ms tick scans awaiting topics; sleep-until-earliest-deadline would skip idle wakeups.
const GATE_TICK_INTERVAL: Duration = Duration::from_millis(100);
const LIVELINESS_GET_TIMEOUT: Duration = Duration::from_millis(500);

type LivelinessGetOutcome = Result<Vec<String>, blueos_comms::CommsError>;

/// Metadata for the file the writer actor has open.
struct OpenRecording {
    file_generation: u64,
}

/// Local state the Domain does not own.
struct DataPlaneLocal {
    open: Option<OpenRecording>,
    descriptors: BTreeMap<ChannelRoute, Arc<ChannelDescriptor>>,
    last_reported_bytes: u64,
    ros2_gate: Ros2ddsGate<Payload>,
    pending_liveliness_gets: BTreeSet<String>,
}

/// Runs until shutdown, reconciling [`RecordGate`] and recording backbone samples.
pub(crate) async fn run_data_plane(
    task_context: TaskContext<RecorderDomain, RecorderContext>,
) -> Result<(), TaskFailed> {
    let writer = Arc::new(McapWriterHandle::spawn_with_queue_capacity(
        task_context.context.mcap_writer_queue_capacity,
    ));
    let mut gate = task_context.context.record_gate.subscribe();
    let mut local = DataPlaneLocal {
        open: None,
        descriptors: BTreeMap::new(),
        last_reported_bytes: 0,
        ros2_gate: Ros2ddsGate::default(),
        pending_liveliness_gets: BTreeSet::new(),
    };
    let folder = Arc::clone(&task_context.context.recordings_folder);
    let mut subscriber = task_context
        .session
        .subscribe("**")
        .await
        .map_err(|error| {
            warn!(%error, "failed to subscribe to the backbone");
            TaskFailed
        })?;
    let mut liveliness = task_context
        .session
        .subscribe_liveliness("@/*/@ros2_lv/**")
        .await
        .map_err(|error| {
            warn!(%error, "failed to subscribe to ros2dds liveliness");
            TaskFailed
        })?;
    let mut liveliness_gets = JoinSet::new();
    let mut bytes_timer = interval(BYTES_REPORT_INTERVAL);
    bytes_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut gate_timer = interval(GATE_TICK_INTERVAL);
    gate_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    {
        let gate_snapshot = gate.borrow().clone();
        reconcile(
            &gate_snapshot,
            &mut local,
            &folder,
            &writer,
            &task_context,
            &mut liveliness_gets,
        )
        .await?;
    }

    loop {
        tokio::select! {
            () = task_context.shutdown.cancelled() => {
                cleanup(&mut local, &writer, &task_context.commands, &mut liveliness_gets).await;
                break;
            }
            changed = gate.changed() => {
                if changed.is_err() {
                    cleanup(&mut local, &writer, &task_context.commands, &mut liveliness_gets).await;
                    break;
                }
                {
                    let gate_snapshot = gate.borrow().clone();
                    reconcile(
                        &gate_snapshot,
                        &mut local,
                        &folder,
                        &writer,
                        &task_context,
                        &mut liveliness_gets,
                    )
                    .await?;
                }
            }
            sample = subscriber.recv() => {
                let Some(sample) = sample else {
                    cleanup(&mut local, &writer, &task_context.commands, &mut liveliness_gets).await;
                    break;
                };
                if let Err(error) = handle_sample(
                    &sample,
                    &mut gate,
                    &mut local,
                    &folder,
                    &writer,
                    &task_context,
                    &mut liveliness_gets,
                )
                .await
                {
                    cleanup(&mut local, &writer, &task_context.commands, &mut liveliness_gets).await;
                    return Err(error);
                }
            }
            event = liveliness.recv() => {
                let Some(event) = event else {
                    warn!("ros2dds liveliness stream ended");
                    continue;
                };
                handle_liveliness_event(
                    event,
                    &mut local,
                    &writer,
                    &task_context,
                    &mut liveliness_gets,
                )
                .await;
            }
            Some(join_result) = liveliness_gets.join_next(), if !liveliness_gets.is_empty() => {
                match join_result {
                    Ok((topic, get_result)) => {
                        local.pending_liveliness_gets.remove(&topic);
                        let token_keys = match get_result {
                            Ok(token_keys) => token_keys,
                            Err(error) => {
                                warn!(%error, topic = %topic, "ros2dds liveliness get failed");
                                Vec::new()
                            }
                        };
                        let outputs = local.ros2_gate.on_input(
                            &topic,
                            Ros2ddsGateInput::LivelinessGetResult { token_keys },
                        );
                        apply_gate_outputs(
                            &topic,
                            outputs,
                            &mut local,
                            &writer,
                            &task_context,
                            &mut liveliness_gets,
                        )
                        .await;
                    }
                    Err(error) => {
                        warn!(%error, "ros2dds liveliness get task failed");
                    }
                }
            }
            _ = gate_timer.tick() => {
                let now_monotonic_millis = monotonic_millis(&task_context);
                for topic in local.ros2_gate.topics_awaiting_timer() {
                    let outputs = local.ros2_gate.on_input(
                        &topic,
                        Ros2ddsGateInput::TimerTick { now_monotonic_millis },
                    );
                    apply_gate_outputs(
                        &topic,
                        outputs,
                        &mut local,
                        &writer,
                        &task_context,
                        &mut liveliness_gets,
                    )
                    .await;
                }
            }
            _ = bytes_timer.tick() => {
                report_bytes_if_due(&mut local, &writer, &task_context.commands).await;
            }
        }
    }
    Ok(())
}

async fn handle_liveliness_event(
    event: LivelinessEvent,
    local: &mut DataPlaneLocal,
    writer: &Arc<McapWriterHandle>,
    task_context: &TaskContext<RecorderDomain, RecorderContext>,
    liveliness_gets: &mut JoinSet<(String, LivelinessGetOutcome)>,
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
                apply_gate_outputs(
                    &data_key,
                    outputs,
                    local,
                    writer,
                    task_context,
                    liveliness_gets,
                )
                .await;
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

async fn handle_sample(
    sample: &Sample,
    gate: &mut tokio::sync::watch::Receiver<RecordGate>,
    local: &mut DataPlaneLocal,
    folder: &Arc<RecordingsFolder>,
    writer: &Arc<McapWriterHandle>,
    task_context: &TaskContext<RecorderDomain, RecorderContext>,
    liveliness_gets: &mut JoinSet<(String, LivelinessGetOutcome)>,
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
        reconcile(
            &gate_snapshot,
            local,
            folder,
            writer,
            task_context,
            liveliness_gets,
        )
        .await?;
    }
    if local.open.is_none() {
        return Ok(());
    }

    let topic = sample.key();
    let wall = task_context.clock.now().wall;
    let log_time = sample_log_time_nanos(sample, wall);
    let publish_time = log_time;

    match plan_sample_write(
        topic,
        sample.encoding(),
        sample.payload(),
        &local.ros2_gate,
        &mut local.descriptors,
    ) {
        SampleWritePlan::Ready { route, descriptor } => {
            writer.try_write_sample(
                topic.to_owned(),
                route,
                log_time,
                publish_time,
                sample.payload().clone(),
                descriptor,
            );
        }
        SampleWritePlan::Skip => {}
        SampleWritePlan::NeedsRos2Gate => {
            let outputs = local.ros2_gate.on_input(
                topic,
                Ros2ddsGateInput::Sample {
                    now_monotonic_millis: monotonic_millis(task_context),
                    log_time,
                    publish_time,
                    payload: sample.payload().clone(),
                },
            );
            apply_gate_outputs(topic, outputs, local, writer, task_context, liveliness_gets).await;
        }
    }
    Ok(())
}

async fn apply_gate_outputs(
    topic: &str,
    outputs: Vec<Ros2ddsGateOutput>,
    local: &mut DataPlaneLocal,
    writer: &Arc<McapWriterHandle>,
    task_context: &TaskContext<RecorderDomain, RecorderContext>,
    liveliness_gets: &mut JoinSet<(String, LivelinessGetOutcome)>,
) {
    if local.open.is_none() {
        return;
    }
    for output in outputs {
        match output {
            Ros2ddsGateOutput::QueryLiveliness { pattern } => {
                if local.pending_liveliness_gets.insert(topic.to_owned()) {
                    let session = Arc::clone(&task_context.session);
                    let topic_owned = topic.to_owned();
                    liveliness_gets.spawn(async move {
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
                write_held_samples(writer, topic, route, descriptor, samples);
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
    for held in samples {
        writer.try_write_sample(
            topic.to_owned(),
            route.clone(),
            held.log_time,
            held.publish_time,
            held.payload,
            Arc::clone(&descriptor),
        );
    }
}

fn monotonic_millis(task_context: &TaskContext<RecorderDomain, RecorderContext>) -> u64 {
    task_context
        .clock
        .now()
        .monotonic
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

async fn reconcile(
    gate: &RecordGate,
    local: &mut DataPlaneLocal,
    folder: &Arc<RecordingsFolder>,
    writer: &Arc<McapWriterHandle>,
    task_context: &TaskContext<RecorderDomain, RecorderContext>,
    liveliness_gets: &mut JoinSet<(String, LivelinessGetOutcome)>,
) -> Result<(), TaskFailed> {
    if !gate.recording_requested {
        finish_open_file(local, writer, &task_context.commands, liveliness_gets).await;
        return Ok(());
    }

    let desired = gate.desired_file_generation;
    if local
        .open
        .as_ref()
        .is_some_and(|open| open.file_generation == desired)
    {
        return Ok(());
    }

    finish_open_file(local, writer, &task_context.commands, liveliness_gets).await;

    let wall = task_context.clock.now().wall;
    let stamp = blueos_recorder_paths::recorder_wall_clock_file_stamp(wall);
    let (path, file_name) = folder.allocate_new_recording(&stamp).map_err(|error| {
        warn!(%error, "failed to allocate recording path");
        TaskFailed
    })?;
    if let Err(error) = writer.open(path, file_name.clone()).await {
        warn!(%error, "failed to open MCAP file");
        return Ok(());
    }
    send_observed(
        &task_context.commands,
        CaptureObservedFact::McapFileOpened {
            file_generation: desired,
            file_name: file_name.clone(),
        },
    )
    .await?;
    local.open = Some(OpenRecording {
        file_generation: desired,
    });
    local.last_reported_bytes = 0;
    Ok(())
}

async fn cleanup(
    local: &mut DataPlaneLocal,
    writer: &Arc<McapWriterHandle>,
    commands: &CommandSender<RecorderDomain>,
    liveliness_gets: &mut JoinSet<(String, LivelinessGetOutcome)>,
) {
    finish_open_file(local, writer, commands, liveliness_gets).await;
}

async fn finish_open_file(
    local: &mut DataPlaneLocal,
    writer: &Arc<McapWriterHandle>,
    commands: &CommandSender<RecorderDomain>,
    liveliness_gets: &mut JoinSet<(String, LivelinessGetOutcome)>,
) {
    liveliness_gets.abort_all();
    local.pending_liveliness_gets.clear();
    local.ros2_gate.clear();
    let Some(open) = local.open.take() else {
        return;
    };
    finish_file(open, writer, commands).await;
}

async fn finish_file(
    open: OpenRecording,
    writer: &Arc<McapWriterHandle>,
    commands: &CommandSender<RecorderDomain>,
) {
    let bytes = match writer.finish().await {
        Ok(bytes) => bytes,
        Err(error) => {
            warn!(%error, "failed to finish MCAP file");
            0
        }
    };
    if send_observed(
        commands,
        CaptureObservedFact::McapFileFinished {
            file_generation: open.file_generation,
        },
    )
    .await
    .is_err()
    {
        return;
    }
    let _ = send_observed(
        commands,
        CaptureObservedFact::RecordingBytesWritten {
            file_generation: open.file_generation,
            bytes,
        },
    )
    .await;
}

async fn report_bytes_if_due(
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

async fn send_observed(
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
