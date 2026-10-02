//! Data plane Task: owns the MCAP writer thread, follows [`RecordGate`], reports Observed facts.

use core::time::Duration;
use std::{collections::BTreeMap, sync::Arc};

use tokio::time::{MissedTickBehavior, interval};
use tracing::warn;

use blueos_comms::Sample;
use blueos_domain::Command;
use blueos_recorder_capture::{CaptureObservedFact, RecordGate};
use blueos_recorder_domain::{RecorderDomain, RecorderObservedFact};
use blueos_recorder_mcap::{
    ChannelDescriptor, McapWriterHandle, descriptor_for_sample, should_record_topic,
};
use blueos_recorder_storage::RecordingsFolder;
use blueos_service::{CommandSender, TaskContext, TaskFailed};

use crate::context::RecorderContext;

const BYTES_REPORT_INTERVAL: Duration = Duration::from_secs(1);

/// Metadata for the file the writer thread has open.
struct OpenRecording {
    file_generation: u64,
}

/// Local state the Domain does not own.
struct DataPlaneLocal {
    open: Option<OpenRecording>,
    descriptors: BTreeMap<String, Arc<ChannelDescriptor>>,
    last_reported_bytes: u64,
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
    let mut bytes_timer = interval(BYTES_REPORT_INTERVAL);
    bytes_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    {
        let gate_snapshot = gate.borrow().clone();
        reconcile(&gate_snapshot, &mut local, &folder, &writer, &task_context).await?;
    }

    loop {
        tokio::select! {
            () = task_context.shutdown.cancelled() => {
                cleanup(&mut local, &writer, &task_context.commands).await;
                break;
            }
            changed = gate.changed() => {
                if changed.is_err() {
                    cleanup(&mut local, &writer, &task_context.commands).await;
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
                    )
                    .await?;
                }
            }
            sample = subscriber.recv() => {
                let Some(sample) = sample else {
                    cleanup(&mut local, &writer, &task_context.commands).await;
                    break;
                };
                if let Err(error) = handle_sample(
                    &sample,
                    &mut gate,
                    &mut local,
                    &folder,
                    &writer,
                    &task_context,
                )
                .await
                {
                    cleanup(&mut local, &writer, &task_context.commands).await;
                    return Err(error);
                }
            }
            _ = bytes_timer.tick() => {
                report_bytes_if_due(&mut local, &writer, &task_context.commands).await;
            }
        }
    }
    Ok(())
}

async fn handle_sample(
    sample: &Sample,
    gate: &mut tokio::sync::watch::Receiver<RecordGate>,
    local: &mut DataPlaneLocal,
    folder: &Arc<RecordingsFolder>,
    writer: &Arc<McapWriterHandle>,
    task_context: &TaskContext<RecorderDomain, RecorderContext>,
) -> Result<(), TaskFailed> {
    let gate_snapshot = gate.borrow().clone();
    if !gate_snapshot.recording_requested {
        return Ok(());
    }
    if !should_record_topic(sample.key()) {
        return Ok(());
    }
    if local
        .open
        .as_ref()
        .is_some_and(|open| open.file_generation != gate_snapshot.desired_file_generation)
    {
        reconcile(&gate_snapshot, local, folder, writer, task_context).await?;
    }
    let Some(descriptor) = descriptor_for_sample(
        sample.key(),
        sample.encoding(),
        sample.payload(),
        &mut local.descriptors,
    ) else {
        return Ok(());
    };
    if local.open.is_none() {
        return Ok(());
    }
    let wall = task_context.clock.now().wall;
    let log_time = sample_log_time_nanos(sample, wall);
    writer.try_write_sample(
        sample.key().to_owned(),
        log_time,
        log_time,
        sample.payload().clone(),
        descriptor,
    );
    Ok(())
}

async fn reconcile(
    gate: &RecordGate,
    local: &mut DataPlaneLocal,
    folder: &Arc<RecordingsFolder>,
    writer: &Arc<McapWriterHandle>,
    task_context: &TaskContext<RecorderDomain, RecorderContext>,
) -> Result<(), TaskFailed> {
    if !gate.recording_requested {
        finish_open_file(local, writer, &task_context.commands).await;
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

    finish_open_file(local, writer, &task_context.commands).await;

    let wall = task_context.clock.now().wall;
    let (path, file_name) = folder.allocate_new_recording(wall).map_err(|error| {
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
) {
    finish_open_file(local, writer, commands).await;
}

async fn finish_open_file(
    local: &mut DataPlaneLocal,
    writer: &Arc<McapWriterHandle>,
    commands: &CommandSender<RecorderDomain>,
) {
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
