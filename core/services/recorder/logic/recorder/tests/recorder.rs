//! L1 tests for Recorder capture lifecycle and known draft 1 bugs (reconcile model).

use core::time::Duration;

use blueos_domain::{Command, Domain, DomainQueries, Now, Outcome};
use blueos_recorder_capture::{
    ActiveRecording, CaptureObservedFact, CaptureSettings, RecordingState,
};
use blueos_recorder_domain::{
    RecorderDomain, RecorderObservedFact, RecorderQuery, RecorderRequest, RecorderSnapshot,
};

const fn now_at(monotonic_seconds: u64) -> Now {
    Now {
        wall: Duration::from_secs(1_700_000_000),
        monotonic: Duration::from_secs(monotonic_seconds),
    }
}

fn task_reports_mcap_file_opened(
    snapshot: &mut RecorderSnapshot,
    file_generation: u64,
    file_name: &str,
) {
    let decision = RecorderDomain::handle(
        snapshot,
        Command::ObservedFact(RecorderObservedFact::Capture(
            CaptureObservedFact::McapFileOpened {
                file_generation,
                file_name: file_name.into(),
            },
        )),
        now_at(0),
    );
    assert!(matches!(decision, Outcome::Applied { .. }));
}

#[test]
fn start_stop_and_rotation_use_record_gate_without_file_io() {
    let mut snapshot = RecorderSnapshot::default();
    let start = RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::StartRecording {
            rotate_if_active: false,
        }),
        now_at(0),
    );
    let Outcome::Applied {
        effects: start_effects,
        ..
    } = start
    else {
        panic!("start must apply");
    };
    assert!(start_effects.is_empty());
    assert!(matches!(
        snapshot.capture.recording,
        RecordingState::AwaitingMcapFile { file_generation: 1 }
    ));
    let gate_after_start = RecorderDomain::query(&snapshot, RecorderQuery::RecordGate, now_at(0));
    assert!(gate_after_start.recording_requested);
    assert_eq!(gate_after_start.desired_file_generation, 1);

    task_reports_mcap_file_opened(&mut snapshot, 1, "a.mcap");
    assert!(matches!(
        snapshot.capture.recording,
        RecordingState::Active(ActiveRecording {
            file_generation: 1,
            ..
        })
    ));

    let rotate = RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::StartRecording {
            rotate_if_active: true,
        }),
        now_at(1),
    );
    let Outcome::Applied {
        effects: rotate_effects,
        ..
    } = rotate
    else {
        panic!("rotate must apply");
    };
    assert!(rotate_effects.is_empty());
    let gate_after_rotate = RecorderDomain::query(&snapshot, RecorderQuery::RecordGate, now_at(1));
    assert!(gate_after_rotate.recording_requested);
    assert_eq!(gate_after_rotate.desired_file_generation, 2);
    task_reports_mcap_file_opened(&mut snapshot, 2, "b.mcap");

    let stop = RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::StopRecording),
        now_at(2),
    );
    let Outcome::Applied {
        effects: stop_effects,
        ..
    } = stop
    else {
        panic!("stop must apply");
    };
    assert!(stop_effects.is_empty());
    assert!(matches!(snapshot.capture.recording, RecordingState::Idle));
    let gate_after_stop = RecorderDomain::query(&snapshot, RecorderQuery::RecordGate, now_at(2));
    assert!(!gate_after_stop.recording_requested);
}

#[test]
fn late_mcap_file_finished_for_previous_file_does_not_clear_active_recording() {
    let mut snapshot = RecorderSnapshot::default();
    RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::StartRecording {
            rotate_if_active: false,
        }),
        now_at(0),
    );
    task_reports_mcap_file_opened(&mut snapshot, 1, "first.mcap");
    RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::StartRecording {
            rotate_if_active: true,
        }),
        now_at(1),
    );
    task_reports_mcap_file_opened(&mut snapshot, 2, "second.mcap");

    RecorderDomain::handle(
        &mut snapshot,
        Command::ObservedFact(RecorderObservedFact::Capture(
            CaptureObservedFact::McapFileFinished { file_generation: 1 },
        )),
        now_at(2),
    );

    let RecordingState::Active(ActiveRecording {
        file_generation,
        file_name,
        ..
    }) = &snapshot.capture.recording
    else {
        panic!("active recording must remain after a stale file finished fact");
    };
    assert_eq!(*file_generation, 2);
    assert_eq!(file_name, "second.mcap");

    RecorderDomain::handle(
        &mut snapshot,
        Command::ObservedFact(RecorderObservedFact::Capture(
            CaptureObservedFact::RecordingBytesWritten {
                file_generation: 2,
                bytes: 4096,
            },
        )),
        now_at(3),
    );
    let RecordingState::Active(ActiveRecording { bytes_written, .. }) = snapshot.capture.recording
    else {
        panic!("bytes must still apply to the active file generation");
    };
    assert_eq!(bytes_written, 4096);
}

#[test]
fn rotation_keeps_recording_requested_until_new_file_is_active() {
    let mut snapshot = RecorderSnapshot::default();
    RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::StartRecording {
            rotate_if_active: false,
        }),
        now_at(0),
    );
    task_reports_mcap_file_opened(&mut snapshot, 1, "first.mcap");
    RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::StartRecording {
            rotate_if_active: true,
        }),
        now_at(1),
    );

    let gate_mid_rotation = RecorderDomain::query(&snapshot, RecorderQuery::RecordGate, now_at(1));
    assert!(gate_mid_rotation.recording_requested);
    assert_eq!(gate_mid_rotation.desired_file_generation, 2);
    assert!(matches!(
        snapshot.capture.recording,
        RecordingState::Active(ActiveRecording {
            file_generation: 1,
            ..
        })
    ));

    task_reports_mcap_file_opened(&mut snapshot, 2, "second.mcap");
    assert!(matches!(
        snapshot.capture.recording,
        RecordingState::Active(ActiveRecording {
            file_generation: 2,
            ..
        })
    ));
    let gate_after_new_file =
        RecorderDomain::query(&snapshot, RecorderQuery::RecordGate, now_at(2));
    assert!(gate_after_new_file.recording_requested);
}

#[test]
fn recording_time_ms_uses_fresh_monotonic_clock() {
    let started_at = Duration::from_secs(10);
    assert_eq!(
        blueos_recorder_capture::Capture::recording_time_ms(started_at, now_at(12)),
        2_000
    );
    assert_eq!(
        blueos_recorder_capture::Capture::recording_time_ms(started_at, now_at(15)),
        5_000
    );
}

#[test]
fn auto_start_recording_applies_live_on_settings_update() {
    let mut snapshot = RecorderSnapshot::default();
    snapshot.capture.settings.auto_start_recording = false;
    let decision = RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::UpdateSettings(CaptureSettings {
            record_mavlink_only_when_armed: true,
            auto_start_recording: true,
        })),
        now_at(0),
    );
    let Outcome::Applied { effects, .. } = decision else {
        panic!("update must apply");
    };
    assert!(effects.is_empty());
    assert!(matches!(
        snapshot.capture.recording,
        RecordingState::AwaitingMcapFile { file_generation: 1 }
    ));
    let gate_after_auto_start =
        RecorderDomain::query(&snapshot, RecorderQuery::RecordGate, now_at(0));
    assert!(gate_after_auto_start.recording_requested);
}

#[test]
fn recording_lifecycle_is_one_enum_without_disagreeing_flags() {
    let mut snapshot = RecorderSnapshot::default();
    assert!(matches!(snapshot.capture.recording, RecordingState::Idle));
    RecorderDomain::handle(
        &mut snapshot,
        Command::Request(RecorderRequest::StartRecording {
            rotate_if_active: false,
        }),
        now_at(0),
    );
    assert!(matches!(
        snapshot.capture.recording,
        RecordingState::AwaitingMcapFile { .. }
    ));
    task_reports_mcap_file_opened(&mut snapshot, 1, "file.mcap");
    assert!(matches!(
        snapshot.capture.recording,
        RecordingState::Active(_)
    ));
}
