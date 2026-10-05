//! Data plane main loop.

use std::{collections::BTreeMap, collections::BTreeSet, sync::Arc};

use tokio::{
    task::JoinSet,
    time::{MissedTickBehavior, interval},
};
use tracing::warn;

use blueos_comms::LivelinessSubscriber;
use blueos_recorder_capture::RecordGate;
use blueos_recorder_domain::RecorderDomain;
use blueos_recorder_mcap::McapWriterHandle;
use blueos_recorder_schema_gate::{Ros2ddsGate, Ros2ddsGateInput};
use blueos_service::{Projection, TaskContext, TaskFailed};

use crate::context::RecorderContext;

use super::{
    BYTES_REPORT_INTERVAL, DataPlaneHandle, DataPlaneLocal, GATE_TICK_INTERVAL,
    LivelinessGetOutcome, apply_gate_outputs, cleanup, handle_liveliness_event, handle_sample,
    monotonic_millis, reconcile, report_bytes_if_due,
};

struct DataPlaneLoop<'a> {
    task_context: &'a TaskContext<RecorderDomain, RecorderContext>,
    gate: &'a mut tokio::sync::watch::Receiver<RecordGate>,
    subscriber: &'a mut blueos_comms::Subscriber,
    liveliness: &'a mut LivelinessSubscriber,
    bytes_timer: &'a mut tokio::time::Interval,
    gate_timer: &'a mut tokio::time::Interval,
    local: &'a mut DataPlaneLocal,
    writer: &'a Arc<McapWriterHandle>,
    handle: &'a mut DataPlaneHandle<'a>,
}

pub(crate) async fn run_data_plane(
    task_context: TaskContext<RecorderDomain, RecorderContext>,
    record_gate: Projection<RecordGate>,
) -> Result<(), TaskFailed> {
    let writer = Arc::new(McapWriterHandle::spawn_with_queue_bytes(
        task_context.context.mcap_writer_queue_bytes,
    ));
    let mut gate = record_gate.subscribe();
    let mut local = DataPlaneLocal {
        open: None,
        recording_video_topics: BTreeSet::new(),
        descriptors: BTreeMap::new(),
        last_reported_bytes: 0,
        samples_dropped: 0,
        ros2_gate: Ros2ddsGate::new(),
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

    let mut handle = DataPlaneHandle {
        folder: &folder,
        writer: &writer,
        task_context: &task_context,
        liveliness_gets: &mut liveliness_gets,
    };
    let current_gate = gate.borrow().clone();
    reconcile(&current_gate, &mut local, &mut handle).await?;

    let mut loop_state = DataPlaneLoop {
        task_context: &task_context,
        gate: &mut gate,
        subscriber: &mut subscriber,
        liveliness: &mut liveliness,
        bytes_timer: &mut bytes_timer,
        gate_timer: &mut gate_timer,
        local: &mut local,
        writer: &writer,
        handle: &mut handle,
    };
    run_data_plane_loop(&mut loop_state).await
}

async fn run_data_plane_loop(loop_state: &mut DataPlaneLoop<'_>) -> Result<(), TaskFailed> {
    loop {
        let shutdown = loop_state.task_context.shutdown.cancelled();
        tokio::select! {
            () = shutdown => {
                cleanup(loop_state.local, loop_state.writer, &loop_state.task_context.commands, loop_state.handle.liveliness_gets).await;
                break;
            }
            changed = loop_state.gate.changed() => {
                if changed.is_err() {
                    cleanup(loop_state.local, loop_state.writer, &loop_state.task_context.commands, loop_state.handle.liveliness_gets).await;
                    break;
                }
                let gate_snapshot = loop_state.gate.borrow().clone();
                reconcile(&gate_snapshot, loop_state.local, loop_state.handle).await?;
            }
            sample = loop_state.subscriber.recv() => {
                let Some(sample) = sample else {
                    cleanup(loop_state.local, loop_state.writer, &loop_state.task_context.commands, loop_state.handle.liveliness_gets).await;
                    break;
                };
                if let Err(error) = handle_sample(&sample, loop_state.gate, loop_state.local, loop_state.handle).await {
                    cleanup(loop_state.local, loop_state.writer, &loop_state.task_context.commands, loop_state.handle.liveliness_gets).await;
                    return Err(error);
                }
            }
            event = loop_state.liveliness.recv() => {
                let Some(event) = event else {
                    warn!("ros2dds liveliness stream ended");
                    continue;
                };
                handle_liveliness_event(event, loop_state.local, loop_state.handle).await;
            }
            Some(join_result) = loop_state.handle.liveliness_gets.join_next(), if !loop_state.handle.liveliness_gets.is_empty() => {
                on_liveliness_get_finished(join_result, loop_state.local, loop_state.handle).await;
            }
            _ = loop_state.gate_timer.tick() => {
                on_gate_timer_tick(loop_state.local, loop_state.handle).await;
            }
            _ = loop_state.bytes_timer.tick() => {
                report_bytes_if_due(loop_state.local, loop_state.writer, &loop_state.task_context.commands).await;
            }
        }
    }
    Ok(())
}

async fn on_liveliness_get_finished(
    join_result: Result<(String, LivelinessGetOutcome), tokio::task::JoinError>,
    local: &mut DataPlaneLocal,
    handle: &mut DataPlaneHandle<'_>,
) {
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
            let outputs = local
                .ros2_gate
                .on_input(&topic, Ros2ddsGateInput::LivelinessGetResult { token_keys });
            apply_gate_outputs(&topic, outputs, local, handle).await;
        }
        Err(error) => {
            warn!(%error, "ros2dds liveliness get task failed");
        }
    }
}

async fn on_gate_timer_tick(local: &mut DataPlaneLocal, handle: &mut DataPlaneHandle<'_>) {
    let now_monotonic_millis = monotonic_millis(handle.task_context);
    for topic in local.ros2_gate.topics_awaiting_timer() {
        let outputs = local.ros2_gate.on_input(
            &topic,
            Ros2ddsGateInput::TimerTick {
                now_monotonic_millis,
            },
        );
        apply_gate_outputs(&topic, outputs, local, handle).await;
    }
}
