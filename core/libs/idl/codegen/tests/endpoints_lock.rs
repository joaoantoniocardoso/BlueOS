//! Endpoint lock line listing tests.

mod common;

use blueos_idl_codegen::endpoints::collect_endpoint_lock_lines;
use common::endpoints::{LEVEL, PUMP, SET_LEVEL, Workspace, messages};

#[test]
fn the_lock_lists_the_standard_endpoints_of_every_service() {
    let workspace = Workspace::new("standard_endpoints");
    workspace.write("Cargo.toml", "[workspace]\nmembers = [\"test/app\"]\n");
    workspace.write("test/app/endpoints.toml", "service = \"test\"\n");

    assert_eq!(
        collect_endpoint_lock_lines(&workspace.root, &messages()).unwrap(),
        [
            "blueos/v1/test/command/AnswerPermission 1 type=blueos_msgs/msg/PermissionAnswer",
            "blueos/v1/test/command/CancelJob 1 type=std_msgs/msg/Empty",
            "blueos/v1/test/command/PauseJob 1 type=std_msgs/msg/Empty",
            "blueos/v1/test/command/ResumeJob 1 type=std_msgs/msg/Empty",
            "blueos/v1/test/command/UpdateSettings 1 type=blueos_msgs/action/UpdateSettings",
            "blueos/v1/test/jobs 1 type=blueos_msgs/msg/JobList",
            "blueos/v1/test/jobs/UpdateSettings/feedback 1 type=blueos_msgs/msg/JobFeedbackList",
            "blueos/v1/test/jobs/UpdateSettings/history 1 type=blueos_msgs/msg/JobList",
            "blueos/v1/test/jobs/UpdateSettings/result 1 type=blueos_msgs/msg/JobResult",
            "blueos/v1/test/log 1 type=foxglove_msgs/msg/Log",
            "blueos/v1/test/query/info 1 type=blueos_msgs/msg/ServiceInfo",
            "blueos/v1/test/settings 1 type=blueos_msgs/msg/SettingsEnvelope",
            "blueos/v1/test/state/metrics 1 type=blueos_msgs/msg/ServiceMetrics",
            "blueos/v1/test/state/status 1 type=blueos_msgs/msg/ServiceStatus",
        ]
    );
}

#[test]
fn the_lock_lists_the_feedback_result_and_history_of_each_job_type_with_their_wire_types() {
    let workspace = Workspace::new("job_outputs");
    workspace.write("Cargo.toml", "[workspace]\nmembers = [\"test/app\"]\n");
    workspace.write(
        "test/app/endpoints.toml",
        &format!("service = \"test\"\n[job]\nSetLevel = {{ type = \"{SET_LEVEL}\" }}\n"),
    );

    let lines = collect_endpoint_lock_lines(&workspace.root, &messages()).unwrap();

    for line in [
        "blueos/v1/test/jobs/SetLevel/feedback 1 type=blueos_msgs/msg/JobFeedbackList",
        "blueos/v1/test/jobs/SetLevel/history 1 type=blueos_msgs/msg/JobList",
        "blueos/v1/test/jobs/SetLevel/result 1 type=blueos_msgs/msg/JobResult",
    ] {
        assert!(
            lines.iter().any(|locked| locked == line),
            "{line} in {lines:?}"
        );
    }
}

#[test]
fn the_lock_lists_every_manifest_endpoint_key_with_its_interface_type() {
    let workspace = Workspace::new("manifest_endpoints");
    workspace.write("Cargo.toml", "[workspace]\nmembers = [\"test/app\"]\n");
    workspace.write(
        "test/app/endpoints.toml",
        &format!(
            "service = \"test\"\n[job]\nSetLevel = {{ type = \"{SET_LEVEL}\" }}\n[query]\nProbe = {{ type = \
             \"{LEVEL}\", io = true }}\n[state]\npump = {{ type = \"{PUMP}\" }}\n"
        ),
    );

    let lines = collect_endpoint_lock_lines(&workspace.root, &messages()).unwrap();

    for expected in [
        format!("blueos/v1/test/command/SetLevel 1 type={SET_LEVEL}"),
        format!("blueos/v1/test/query/Probe 1 type={LEVEL}"),
        format!("blueos/v1/test/state/pump 1 type={PUMP}"),
    ] {
        assert!(lines.contains(&expected), "{expected} in {lines:?}");
    }
}
