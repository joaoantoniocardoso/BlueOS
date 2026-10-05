//! Recording file reconcile and teardown.

use std::sync::Arc;

use tokio::task::JoinSet;
use tracing::{info, warn};

use blueos_recorder_capture::{CaptureObservedFact, RecordGate};
use blueos_recorder_domain::RecorderDomain;
use blueos_recorder_mcap::McapWriterHandle;
use blueos_service::{CommandSender, TaskFailed};

use super::{DataPlaneHandle, DataPlaneLocal, LivelinessGetOutcome, OpenRecording, send_observed};

fn log_recording_video_topic_changes(gate: &RecordGate, local: &mut DataPlaneLocal) {
    for topic in gate
        .recording_video_topics
        .difference(&local.recording_video_topics)
    {
        info!(topic, "Started recording a video stream");
    }
    for topic in local
        .recording_video_topics
        .difference(&gate.recording_video_topics)
    {
        info!(topic, "Stopped recording a video stream");
    }
    local.recording_video_topics = gate.recording_video_topics.clone();
}

pub(super) async fn reconcile(
    gate: &RecordGate,
    local: &mut DataPlaneLocal,
    handle: &mut DataPlaneHandle<'_>,
) -> Result<(), TaskFailed> {
    log_recording_video_topic_changes(gate, local);

    if !gate.recording_requested {
        finish_open_file(
            local,
            handle.writer,
            &handle.task_context.commands,
            handle.liveliness_gets,
        )
        .await;
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

    finish_open_file(
        local,
        handle.writer,
        &handle.task_context.commands,
        handle.liveliness_gets,
    )
    .await;

    open_desired_recording(local, handle, desired).await
}

async fn open_desired_recording(
    local: &mut DataPlaneLocal,
    handle: &mut DataPlaneHandle<'_>,
    desired: u64,
) -> Result<(), TaskFailed> {
    let wall = handle.task_context.clock.now().wall;
    let stamp = blueos_recorder_paths::recorder_wall_clock_file_stamp(wall);
    let (path, file_name) = handle
        .folder
        .allocate_new_recording(&stamp)
        .map_err(|error| {
            warn!(%error, "failed to allocate recording path");
            TaskFailed
        })?;
    if let Err(error) = handle.writer.open(path, file_name.clone()).await {
        warn!(%error, "failed to open MCAP file");
        return Ok(());
    }
    info!(file_name, "Opened a recording");
    send_observed(
        &handle.task_context.commands,
        CaptureObservedFact::McapFileOpened {
            file_generation: desired,
            file_name: file_name.clone(),
        },
    )
    .await?;
    local.open = Some(OpenRecording {
        file_generation: desired,
        file_name,
    });
    local.last_reported_bytes = 0;
    local.samples_dropped = 0;
    Ok(())
}

pub(super) async fn cleanup(
    local: &mut DataPlaneLocal,
    writer: &Arc<McapWriterHandle>,
    commands: &CommandSender<RecorderDomain>,
    liveliness_gets: &mut JoinSet<(String, LivelinessGetOutcome)>,
) {
    finish_open_file(local, writer, commands, liveliness_gets).await;
}

pub(super) async fn finish_open_file(
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
    info!(file_name = open.file_name, bytes, "Finished a recording");
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
