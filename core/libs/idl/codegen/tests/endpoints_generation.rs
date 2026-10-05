//! Generated endpoint source shape tests.

mod common;

use blueos_idl_codegen::endpoints::generate;
use common::endpoints::{LEVEL, PUMP, SET_LEVEL, messages};

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
