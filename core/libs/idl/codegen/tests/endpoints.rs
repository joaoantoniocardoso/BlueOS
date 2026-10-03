//! The endpoint generator: what it rejects, what it generates for each mix of endpoints, and where it reads.

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process,
};

use blueos_idl_codegen::{
    endpoints::{
        EndpointsError, ManifestError, collect_endpoint_lock_lines, generate, generate_all,
        stray_files,
    },
    message_schema_names,
};

const EMPTY: &str = "blueos_example_msgs/msg/EmptyRequest";
const LEVEL: &str = "blueos_example_msgs/msg/LevelQueryResponse";

#[test]
fn rejects_two_endpoints_that_generate_the_same_function() {
    let manifest = format!(
        "service = \"test\"\n[command]\nLevel = {{ request = \"{EMPTY}\" }}\n[state]\nlevel = {{ message = \
         \"{LEVEL}\" }}\n"
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
        "service = \"test\"\n[command]\nLevelResponse = {{ request = \"{EMPTY}\" }}\n[query]\nLevel = {{ request = \
         \"{EMPTY}\", response = \"{LEVEL}\" }}\n"
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
    ] {
        let manifest =
            format!("service = \"test\"\n[state]\n{name} = {{ message = \"{LEVEL}\" }}\n");

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
            format!("service = \"test\"\n[state]\n\"{name}\" = {{ message = \"{LEVEL}\" }}\n");

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
fn rejects_a_message_that_is_not_in_blueos_idl() {
    for message in ["blueos_example_msgs/msg/Missing", "LevelQueryResponse"] {
        let manifest =
            format!("service = \"test\"\n[event]\nChanged = {{ message = \"{message}\" }}\n");

        assert_eq!(
            rejection(&manifest),
            ManifestError::UnknownMessage {
                endpoint: "Changed".to_owned(),
                message: message.to_owned(),
            }
        );
    }
}

#[test]
fn rejects_what_the_format_does_not_have() {
    let custom_state = format!(
        "service = \"test\"\n[state]\nlevel = {{ message = \"{LEVEL}\", custom = true }}\n"
    );

    for manifest in [
        custom_state.as_str(),
        "[state]\n",
        "service = \"test\"\n[timer]\n",
    ] {
        assert!(
            matches!(rejection(manifest), ManifestError::Parse(_)),
            "{manifest}"
        );
    }
}

#[test]
fn a_service_whose_endpoints_are_all_handled_has_no_conversions() {
    let manifest = format!(
        "service = \"test\"\n[command]\nSet = {{ request = \"{EMPTY}\", custom = true }}\n[io_query]\nProbe = {{ \
         request = \"{EMPTY}\", response = \"{LEVEL}\" }}\n"
    );

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
        "service = \"test\"\n[command]\nSet = {{ request = \"{EMPTY}\" }}\n[state]\nlevel = {{ message = \
         \"{LEVEL}\" }}\n"
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
fn a_command_with_a_nature_is_a_job_type_whose_conversion_gets_the_job_id() {
    let manifest = format!(
        "service = \"test\"\n[command]\nRepair = {{ request = \"{EMPTY}\", custom = true, nature = {{ lasting = \
         true, cancellable = true }} }}\nSet = {{ request = \"{EMPTY}\", nature = {{ needs_permission = true }} \
         }}\nStop = {{ request = \"{EMPTY}\" }}\n"
    );

    let generated = generate(&manifest, "blueos_test_api", &messages()).unwrap();

    assert!(generated.api.contains(
        "fn set(job_id: JobId, request: blueos_example_msgs::EmptyRequest) -> Self::Request;"
    ));
    assert!(
        generated
            .api
            .contains("fn stop(request: blueos_example_msgs::EmptyRequest) -> Self::Request;")
    );
    assert!(generated.app.contains("job_id: JobId,"));
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
    assert!(
        generated
            .app
            .contains("H::repair(&handlers, job_id, request)")
    );
    assert!(
        generated
            .app
            .contains("Ok(<D as Conversions>::set(job_id, request))")
    );
    assert!(generated.app.contains(".command(\"Stop\""));
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
fn typescript_client_uses_key_helpers_and_idl_types() {
    let manifest = format!(
        "service = \"tank\"\n[command]\nDrain = {{ request = \"{EMPTY}\" }}\n[state]\ntank = {{ message = \
         \"{LEVEL}\" }}\n"
    );

    let generated = generate(&manifest, "blueos_tank_api", &messages()).unwrap();

    assert!(
        generated
            .typescript
            .contains("import type * as Idl from '@blueos-idl/messages'")
    );
    assert!(generated.typescript.contains("from '../keys'"));
    assert!(generated.typescript.contains("commandKey(NAME, 'Drain')"));
    assert!(generated.typescript.contains("stateKey(NAME, 'tank')"));
    assert!(
        generated
            .typescript
            .contains("export type DrainRequest = Idl.EmptyRequest")
    );
    assert!(
        generated
            .typescript
            .contains("export type Tank = Idl.LevelQueryResponse")
    );
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
            "blueos/v1/test/command/AnswerPermission 1 \
             request=blueos_msgs/msg/PermissionAnswer;response=blueos_msgs/msg/CommandAck",
            "blueos/v1/test/command/CancelJob 1 request=;response=blueos_msgs/msg/CommandAck",
            "blueos/v1/test/command/PauseJob 1 request=;response=blueos_msgs/msg/CommandAck",
            "blueos/v1/test/command/ResumeJob 1 request=;response=blueos_msgs/msg/CommandAck",
            "blueos/v1/test/command/UpdateSettings 1 \
             request=blueos_msgs/msg/SettingsEnvelope;response=blueos_msgs/msg/CommandAck",
            "blueos/v1/test/jobs 1 request=;response=blueos_msgs/msg/JobList",
            "blueos/v1/test/log 1 request=;response=foxglove_msgs/msg/Log",
            "blueos/v1/test/query/info 1 request=;response=blueos_msgs/msg/ServiceInfo",
            "blueos/v1/test/settings 1 request=;response=blueos_msgs/msg/SettingsEnvelope",
            "blueos/v1/test/state/status 1 request=;response=blueos_msgs/msg/ServiceStatus",
        ]
    );
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
        "service = \"Info\"\n[state]\nInfo = { message = \"x\" }\n",
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

fn messages() -> BTreeSet<String> {
    [EMPTY, LEVEL].into_iter().map(str::to_owned).collect()
}
