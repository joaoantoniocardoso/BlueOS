//! Endpoint manifest rejection tests.

mod common;

use blueos_idl_codegen::endpoints::ManifestError;
use common::endpoints::{LEVEL, PUMP, SET_LEVEL, rejection};

#[test]
fn rejects_two_endpoints_that_generate_the_same_function() {
    let manifest = format!(
        "service = \"test\"\n[job]\nLevel = {{ type = \"{SET_LEVEL}\" }}\n[state]\nlevel = {{ type = \
         \"{PUMP}\" }}\n"
    );

    assert_eq!(
        rejection(&manifest),
        ManifestError::Duplicate {
            first: "Level".to_owned(),
            second: "level".to_owned(),
            function: "level".to_owned(),
        }
    );
}

#[test]
fn rejects_an_endpoint_named_like_the_reply_function_of_a_query() {
    let manifest = format!(
        "service = \"test\"\n[job]\nLevelResponse = {{ type = \"{SET_LEVEL}\" }}\n[query]\nLevel = {{ type = \
         \"{LEVEL}\" }}\n"
    );

    assert_eq!(
        rejection(&manifest),
        ManifestError::Duplicate {
            first: "LevelResponse".to_owned(),
            second: "Level".to_owned(),
            function: "level_response".to_owned(),
        }
    );
}

#[test]
fn rejects_an_endpoint_named_like_the_feedback_or_job_result_of_a_job_type() {
    for function in ["set_level_feedback", "set_level_result"] {
        let manifest = format!(
            "service = \"test\"\n[job]\nSetLevel = {{ type = \"{SET_LEVEL}\" }}\n[state]\n{function} = {{ type \
             = \"{PUMP}\" }}\n"
        );

        assert_eq!(
            rejection(&manifest),
            ManifestError::Duplicate {
                first: "SetLevel".to_owned(),
                second: function.to_owned(),
                function: function.to_owned(),
            }
        );
    }
}

#[test]
fn rejects_the_names_every_service_has() {
    for name in [
        "Info",
        "Status",
        "settings",
        "UpdateSettings",
        "jobs",
        "CancelJob",
        "PauseJob",
        "ResumeJob",
        "AnswerPermission",
        "Log",
        "metrics",
        "Metrics",
    ] {
        let manifest = format!("service = \"test\"\n[state]\n{name} = {{ type = \"{PUMP}\" }}\n");

        assert_eq!(
            rejection(&manifest),
            ManifestError::Reserved(name.to_owned())
        );
    }
}

#[test]
fn rejects_names_that_are_not_identifiers() {
    for name in ["1st", "Set-Level", "fn", "_hidden"] {
        let manifest =
            format!("service = \"test\"\n[state]\n\"{name}\" = {{ type = \"{PUMP}\" }}\n");

        assert_eq!(
            rejection(&manifest),
            ManifestError::InvalidName(name.to_owned())
        );
    }
    assert_eq!(
        rejection("service = \"my service\"\n"),
        ManifestError::InvalidName("my service".to_owned())
    );
}

#[test]
fn rejects_an_interface_type_of_another_kind_or_not_in_blueos_idl() {
    for (kind, interface_type, expected) in [
        ("job", PUMP, ".action"),
        ("job", LEVEL, ".action"),
        ("query", SET_LEVEL, ".srv"),
        ("state", LEVEL, ".msg"),
        ("event", "blueos_example_msgs/msg/Missing", ".msg"),
        ("event", "PumpState", ".msg"),
        ("job", "blueos_example_msgs/action/Missing", ".action"),
    ] {
        let manifest =
            format!("service = \"test\"\n[{kind}]\nChanged = {{ type = \"{interface_type}\" }}\n");

        assert_eq!(
            rejection(&manifest),
            ManifestError::UnknownType {
                endpoint: "Changed".to_owned(),
                interface_type: interface_type.to_owned(),
                expected,
            },
            "{manifest}"
        );
    }
}

#[test]
fn rejects_what_the_format_does_not_have() {
    let cases = [
        format!("service = \"test\"\n[state]\nlevel = {{ type = \"{PUMP}\", custom = true }}\n"),
        format!("service = \"test\"\n[query]\nLevel = {{ type = \"{LEVEL}\", custom = true }}\n"),
        format!("service = \"test\"\n[job]\nSetLevel = {{ type = \"{SET_LEVEL}\", io = true }}\n"),
        format!("service = \"test\"\n[state]\nlevel = {{ message = \"{PUMP}\" }}\n"),
        format!("service = \"test\"\n[command]\nSetLevel = {{ type = \"{SET_LEVEL}\" }}\n"),
        format!("service = \"test\"\n[io_query]\nLevel = {{ type = \"{LEVEL}\" }}\n"),
        "[state]\n".to_owned(),
        "service = \"test\"\n[timer]\n".to_owned(),
    ];

    for manifest in &cases {
        assert!(
            matches!(rejection(manifest), ManifestError::Parse(_)),
            "{manifest}"
        );
    }
}
