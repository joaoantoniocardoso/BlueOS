//! The endpoint generator: what it rejects, what it generates for each mix of endpoints, and where it reads.

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process,
};

use blueos_idl_codegen::{
    endpoints::{
        EndpointsError, ManifestError, collect_endpoint_lock_lines, format, generate, generate_all,
        stray_files,
    },
    message_schema_names,
};

const SET_LEVEL: &str = "blueos_example_msgs/action/SetLevel";
const LEVEL: &str = "blueos_example_msgs/srv/Level";
const PUMP: &str = "blueos_example_msgs/msg/PumpState";

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

#[test]
fn a_service_whose_endpoints_are_all_io_queries_has_no_conversions() {
    let manifest =
        format!("service = \"test\"\n[query]\nProbe = {{ type = \"{LEVEL}\", io = true }}\n");

    let generated = generate(&manifest, "blueos_test_api", &messages()).unwrap();

    assert!(!generated.api.contains("trait Conversions"));
    assert!(generated.app.contains("pub trait Handlers<D: Domain>"));
    assert!(
        generated
            .app
            .contains("pub fn register<D: Domain, H: Handlers<D>, Context>")
    );
    assert!(!generated.app.contains("Conversions"));
}

#[test]
fn a_service_without_custom_endpoints_has_no_handlers() {
    let manifest = format!(
        "service = \"test\"\n[job]\nSet = {{ type = \"{SET_LEVEL}\" }}\n[state]\nlevel = {{ type = \"{PUMP}\" \
         }}\n"
    );

    let generated = generate(&manifest, "blueos_test_api", &messages()).unwrap();

    assert!(generated.api.contains("pub trait Conversions: Domain"));
    assert!(
        generated
            .app
            .contains("pub fn register<D: Conversions, Context>(builder")
    );
    assert!(
        generated
            .app
            .contains("use blueos_test_api::endpoints::Conversions;")
    );
    assert!(!generated.app.contains("Handlers"));
    assert!(!generated.app.contains("Arc"));
}

#[test]
fn a_goal_becomes_a_request_through_a_fallible_conversion_whose_error_is_the_rejection() {
    let manifest = format!("service = \"test\"\n[job]\nSetLevel = {{ type = \"{SET_LEVEL}\" }}\n");

    let generated = generate(&manifest, "blueos_test_api", &messages()).unwrap();

    assert!(
        generated
            .api
            .contains("type SetLevelError: core::error::Error + Send + Sync + 'static;")
    );
    assert!(generated.api.contains(
        "fn set_level(goal: blueos_example_msgs::SetLevelGoal) -> Result<Self::Request, \
         Self::SetLevelError>;"
    ));
    assert!(generated.app.contains(
        "|goal: blueos_example_msgs::SetLevelGoal| <D as Conversions>::set_level(goal).map_err(Refusal::from)"
    ));
}

#[test]
fn each_job_type_publishes_its_feedback_and_job_result_through_conversions() {
    let manifest = format!(
        "service = \"test\"\n[job]\nRepair = {{ type = \"{SET_LEVEL}\", custom = true }}\nSetLevel = {{ type = \
         \"{SET_LEVEL}\" }}\n"
    );

    let generated = generate(&manifest, "blueos_test_api", &messages()).unwrap();

    for function in ["repair", "set_level"] {
        assert!(generated.api.contains(&format!(
            "fn {function}_feedback(snapshot: &Self::Snapshot, job_id: JobId) -> \
             Option<blueos_example_msgs::SetLevelFeedback>;"
        )));
        assert!(generated.api.contains(&format!(
            "fn {function}_result(snapshot: &Self::Snapshot, job_id: JobId) -> \
             blueos_example_msgs::SetLevelResult;"
        )));
    }
    assert!(!generated.api.contains("fn repair(goal"));
    let app = generated.app.split_whitespace().collect::<String>();
    assert!(app.contains(".job_feedback(\"Repair\",<DasConversions>::repair_feedback)"));
    assert!(app.contains(".job_result(\"Repair\",<DasConversions>::repair_result)"));
    assert!(app.contains(".job_feedback(\"SetLevel\",<DasConversions>::set_level_feedback)"));
    assert!(app.contains(".job_result(\"SetLevel\",<DasConversions>::set_level_result)"));
}

#[test]
fn a_job_type_with_a_nature_gets_the_job_id_in_its_goal_mapping() {
    let manifest = format!(
        "service = \"test\"\n[job]\nRepair = {{ type = \"{SET_LEVEL}\", custom = true, nature = {{ lasting = \
         true, cancellable = true }} }}\nSet = {{ type = \"{SET_LEVEL}\", nature = {{ needs_permission = true \
         }} }}\nStop = {{ type = \"{SET_LEVEL}\" }}\n"
    );

    let generated = generate(&manifest, "blueos_test_api", &messages()).unwrap();

    assert!(generated.api.contains(
        "fn set(job_id: JobId, goal: blueos_example_msgs::SetLevelGoal) -> Result<Self::Request, \
         Self::SetError>;"
    ));
    assert!(generated.api.contains(
        "fn stop(goal: blueos_example_msgs::SetLevelGoal) -> Result<Self::Request, Self::StopError>;"
    ));
    assert!(generated.app.contains(
        "fn repair(&self, job_id: JobId, goal: blueos_example_msgs::SetLevelGoal) -> Result<D::Request, \
         Refusal>;"
    ));
    assert!(
        generated
            .app
            .contains("pub fn register<D: Conversions + DomainJobs, H: Handlers<D>, Context>(")
    );
    let repair = generated.app.split(".job(").nth(1).unwrap();
    assert!(repair.trim_start().starts_with("\"Repair\""));
    assert!(repair.contains("lasting: true,"));
    assert!(repair.contains("cancellable: true,"));
    assert!(repair.contains("pausable: false,"));
    assert!(generated.app.contains("H::repair(&handlers, job_id, goal)"));
    assert!(
        generated
            .app
            .contains("<D as Conversions>::set(job_id, goal).map_err(Refusal::from)")
    );
    assert!(generated.app.contains(".command(\"Stop\""));
}

#[test]
fn an_io_query_is_a_handler_method_answered_outside_the_inbox() {
    let manifest =
        format!("service = \"test\"\n[query]\nProbe = {{ type = \"{LEVEL}\", io = true }}\n");

    let generated = generate(&manifest, "blueos_test_api", &messages()).unwrap();

    assert!(generated.app.contains(
        "fn probe(&self, request: blueos_example_msgs::LevelRequest) -> impl Future<Output = \
         Result<blueos_example_msgs::LevelResponse, Refusal>> + Send;"
    ));
    assert!(generated.app.contains(".io_query("));
}

#[test]
fn info_lists_each_endpoint_with_its_kind_interface_type_and_schema_text() {
    let manifest = format!(
        "service = \"test\"\n[job]\nSetLevel = {{ type = \"{SET_LEVEL}\" }}\n[query]\nLevel = {{ type = \
         \"{LEVEL}\" }}\nProbe = {{ type = \"{LEVEL}\", io = true }}\n[event]\nchanged = {{ type = \"{PUMP}\" \
         }}\n"
    );

    let generated = generate(&manifest, "blueos_test_api", &messages()).unwrap();

    let app = generated.app.split_whitespace().collect::<String>();
    for (kind, name, segment, interface_type) in [
        ("job", "SetLevel", "command", SET_LEVEL),
        ("query", "Level", "query", LEVEL),
        ("query", "Probe", "query", LEVEL),
        ("event", "changed", "event", PUMP),
    ] {
        assert!(
            app.contains(&format!(
                "EndpointInfo{{kind:\"{kind}\".into(),name:\"{name}\".into(),key:\
                 \"blueos/v1/test/{segment}/{name}\".into(),interface_type:\"{interface_type}\".into(),\
                 schema:blueos_idl::schema(\"{interface_type}\").unwrap_or_default().into(),}}"
            )),
            "{name}"
        );
    }
}

#[test]
fn a_service_without_endpoints_registers_nothing() {
    let generated = generate("service = \"test\"\n", "blueos_test_api", &messages()).unwrap();

    assert!(!generated.api.contains("trait"));
    assert!(
        generated
            .app
            .contains("pub fn register<D: Domain, Context>(builder")
    );
    assert!(!generated.app.contains("blueos_idl"));
    assert!(generated.typescript.contains("export const NAME = 'test'"));
    assert!(!generated.typescript.contains("commandKey"));
}

#[test]
fn typescript_client_exposes_the_goal_feedback_job_result_request_and_response_types() {
    let manifest = format!(
        "service = \"tank\"\n[job]\nDrain = {{ type = \"{SET_LEVEL}\" }}\n[query]\nLevel = {{ type = \
         \"{LEVEL}\", io = true }}\n[state]\ntank = {{ type = \"{PUMP}\" }}\n"
    );

    let generated = generate(&manifest, "blueos_tank_api", &messages()).unwrap();

    for expected in [
        "import type * as Idl from '@blueos-idl/messages'",
        "from '../keys'",
        "kind: 'job' as const,",
        "key: commandKey(NAME, 'Drain'),",
        "type: 'blueos_example_msgs/action/SetLevel' as const,",
        "goalSchema: 'blueos_example_msgs/action/SetLevel_Goal' as const,",
        "feedbackSchema: 'blueos_example_msgs/action/SetLevel_Feedback' as const,",
        "resultSchema: 'blueos_example_msgs/action/SetLevel_Result' as const,",
        "export type DrainGoal = Idl.SetLevelGoal",
        "export type DrainFeedback = Idl.SetLevelFeedback",
        "export type DrainResult = Idl.SetLevelResult",
        "kind: 'query' as const,",
        "key: queryKey(NAME, 'Level'),",
        "requestSchema: 'blueos_example_msgs/srv/Level_Request' as const,",
        "responseSchema: 'blueos_example_msgs/srv/Level_Response' as const,",
        "export type LevelRequest = Idl.LevelRequest",
        "export type LevelResponse = Idl.LevelResponse",
        "key: stateKey(NAME, 'tank'),",
        "messageSchema: 'blueos_example_msgs/msg/PumpState' as const,",
        "export type Tank = Idl.PumpState",
    ] {
        assert!(generated.typescript.contains(expected), "{expected}");
    }
    assert!(!generated.typescript.contains("io_query"));
    assert!(!generated.typescript.contains("vue"));
}

#[test]
fn the_committed_endpoint_files_match_their_manifests() {
    let core_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let messages = message_schema_names(&core_dir.join("libs/idl/interfaces"));

    let files = generate_all(&core_dir, &messages).unwrap();

    assert!(!files.is_empty());
    for file in files {
        assert_eq!(
            fs::read_to_string(&file.path).unwrap(),
            file.contents,
            "{} is stale: run `cargo run -p blueos-idl-codegen -- --write`",
            file.path.display()
        );
    }
}

#[test]
fn the_custom_handler_fixture_of_the_compile_fail_tests_matches_its_manifest() {
    let core_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let fixture = core_dir.join("services/example/app/tests/compile_fail/custom");
    let manifest = fs::read_to_string(fixture.join("endpoints.toml")).unwrap();
    let messages = message_schema_names(&core_dir.join("libs/idl/interfaces"));

    let generated = generate(&manifest, "blueos_example_api", &messages).unwrap();

    assert_eq!(
        fs::read_to_string(fixture.join("endpoints.rs")).unwrap(),
        format(&generated.app),
        "{} is stale: write the generated app source there",
        fixture.join("endpoints.rs").display()
    );
}

#[test]
fn a_generated_file_without_its_manifest_is_stray() {
    let workspace = Workspace::new("stray_files");
    workspace.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"test/app\", \"test/logic/api\", \"gone/app\"]\n",
    );
    workspace.write("test/app/endpoints.toml", "service = \"test\"\n");
    workspace.write(
        "test/logic/api/Cargo.toml",
        "[package]\nname = \"blueos-test-api\"\n",
    );
    let files = generate_all(&workspace.root, &messages()).unwrap();
    for file in &files {
        fs::create_dir_all(file.path.parent().unwrap()).unwrap();
        fs::write(&file.path, &file.contents).unwrap();
    }
    let gone = generate("service = \"gone\"\n", "blueos_gone_api", &messages()).unwrap();
    workspace.write(
        "frontend/src/libs/blueos-api/services/gone.ts",
        &gone.typescript,
    );
    workspace.write("gone/app/src/endpoints.rs", &gone.app);
    workspace.write("gone/app/src/lib.rs", "//! Written by hand.\n");

    assert_eq!(
        stray_files(&workspace.root, &files).unwrap(),
        [
            workspace
                .root
                .join("frontend/src/libs/blueos-api/services/gone.ts"),
            workspace.root.join("gone/app/src/endpoints.rs"),
        ]
    );
}

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

#[test]
fn generate_all_names_the_manifest_that_is_wrong() {
    let workspace = Workspace::new("wrong_manifest");
    workspace.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"other\", \"test/app\"]\n",
    );
    workspace.write(
        "test/app/endpoints.toml",
        "service = \"Info\"\n[state]\nInfo = { type = \"x\" }\n",
    );
    workspace.write(
        "test/logic/api/Cargo.toml",
        "[package]\nname = \"blueos-test-api\"\n",
    );

    let error = generate_all(&workspace.root, &messages()).unwrap_err();

    assert!(matches!(
        error,
        EndpointsError::Manifest { path, error: ManifestError::Reserved(_) }
            if path == workspace.root.join("test/app/endpoints.toml")
    ));
}

#[test]
fn generate_all_names_the_cargo_toml_it_cannot_use() {
    let cases = [
        ("no_members", "[workspace]\n", None),
        ("bad_toml", "[workspace\n", None),
        (
            "no_package_name",
            "[workspace]\nmembers = [\"test/app\"]\n",
            Some("[package]\n"),
        ),
    ];
    for (name, workspace_toml, api_toml) in cases {
        let workspace = Workspace::new(name);
        workspace.write("Cargo.toml", workspace_toml);
        workspace.write("test/app/endpoints.toml", "service = \"test\"\n");
        if let Some(api_toml) = api_toml {
            workspace.write("test/logic/api/Cargo.toml", api_toml);
        }

        let error = generate_all(&workspace.root, &messages()).unwrap_err();

        assert!(
            matches!(error, EndpointsError::Cargo { .. }),
            "{name}: {error}"
        );
    }
}

#[test]
fn generate_all_names_the_file_it_cannot_read() {
    let workspace = Workspace::new("no_api_crate");
    workspace.write("Cargo.toml", "[workspace]\nmembers = [\"test/app\"]\n");
    workspace.write("test/app/endpoints.toml", "service = \"test\"\n");

    let error = generate_all(&workspace.root, &messages()).unwrap_err();

    assert!(
        error.to_string().starts_with(
            &workspace
                .root
                .join("test/logic/api/Cargo.toml")
                .display()
                .to_string()
        ),
        "{error}"
    );
}

/// A throwaway workspace directory, removed when dropped.
struct Workspace {
    root: PathBuf,
}

impl Drop for Workspace {
    fn drop(&mut self) {
        _ = fs::remove_dir_all(&self.root);
    }
}

impl Workspace {
    fn new(name: &str) -> Self {
        let root = env::temp_dir().join(format!("blueos-endpoints-{}-{name}", process::id()));
        _ = fs::remove_dir_all(&root);
        Self { root }
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}

fn rejection(manifest: &str) -> ManifestError {
    generate(manifest, "blueos_test_api", &messages()).unwrap_err()
}

/// The schema names `message_schema_names` lists for the three interface types above: one per part.
fn messages() -> BTreeSet<String> {
    [
        "blueos_example_msgs/action/SetLevel_Goal",
        "blueos_example_msgs/action/SetLevel_Result",
        "blueos_example_msgs/action/SetLevel_Feedback",
        "blueos_example_msgs/srv/Level_Request",
        "blueos_example_msgs/srv/Level_Response",
        PUMP,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
