//! Recording policy tests for MAVLink and video gating.

use blueos_recorder_capture::{Capture, RecordGate};

#[test]
fn mavlink_not_recorded_while_disarmed_when_policy_enabled() {
    let gate = RecordGate {
        recording_requested: true,
        armed: false,
        record_mavlink_only_when_armed: true,
        desired_file_generation: 1,
        recording_video_topics: Default::default(),
    };
    assert!(!Capture::should_record_sample(
        "mavlink/1/1/HEARTBEAT",
        &gate
    ));
    assert!(!Capture::should_record_sample("mavlink_raw/in", &gate));
    assert!(Capture::should_record_sample(
        "blueos/v1/example/state",
        &gate
    ));
}

#[test]
fn mavlink_recorded_when_armed() {
    let gate = RecordGate {
        recording_requested: true,
        armed: true,
        record_mavlink_only_when_armed: true,
        desired_file_generation: 1,
        recording_video_topics: Default::default(),
    };
    assert!(Capture::should_record_sample(
        "mavlink/1/1/HEARTBEAT",
        &gate
    ));
}

#[test]
fn mavlink_recorded_when_armed_gating_disabled() {
    let gate = RecordGate {
        recording_requested: true,
        armed: false,
        record_mavlink_only_when_armed: false,
        desired_file_generation: 1,
        recording_video_topics: Default::default(),
    };
    assert!(Capture::should_record_sample(
        "mavlink/1/1/HEARTBEAT",
        &gate
    ));
}
