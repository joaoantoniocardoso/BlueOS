use std::process::Command;

use crate::catalog::Catalog;
use crate::id::{JourneyId, ServiceId};
use crate::journey::{derive_automatable, Actor, Automatable, HttpMethod, RouteRef, UserJourney};
use crate::provenance::{Grounded, GroundedSet, ObservedSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Pass,
    Fail(String),
    Skip(String),
    Unasserted,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JourneyResult {
    Pass,
    Fail,
    Skip,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnableStep {
    pub journey_id: JourneyId,
    pub step_index: usize,
    pub route: RouteRef,
    pub expected_status: Option<u16>,
    pub body_predicate: Option<&'static str>,
    pub body: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RunCounts {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub unasserted: usize,
}

impl RunCounts {
    pub fn record(&mut self, result: &StepResult) {
        match result {
            StepResult::Pass => self.passed += 1,
            StepResult::Fail(_) => self.failed += 1,
            StepResult::Skip(_) => self.skipped += 1,
            StepResult::Unasserted => self.unasserted += 1,
            StepResult::Ignored => self.skipped += 1,
        }
    }
}

pub fn http_journeys(catalog: &Catalog) -> Vec<&UserJourney> {
    catalog
        .journeys()
        .iter()
        .filter(|journey| derive_automatable(journey) == Automatable::Http)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier1GetCoverage {
    pub http_journeys: usize,
    pub get_concrete: usize,
    pub get_asserted: usize,
    pub get_unasserted: usize,
    pub get_templated: usize,
    pub unasserted: Vec<(JourneyId, usize, &'static str)>,
}

fn is_templated_http_path(path: &str) -> bool {
    path.contains('{') || path.contains('*')
}

pub fn is_smoke_excluded_get(path: &str) -> bool {
    matches!(
        path,
        "/disk/speed/stream" | "/internet_download_speed" | "/internet_upload_speed"
    )
}

pub fn tier1_get_coverage(catalog: &Catalog) -> Tier1GetCoverage {
    let journeys = http_journeys(catalog);
    let mut coverage = Tier1GetCoverage {
        http_journeys: journeys.len(),
        get_concrete: 0,
        get_asserted: 0,
        get_unasserted: 0,
        get_templated: 0,
        unasserted: Vec::new(),
    };

    for journey in journeys {
        for step in http_steps(journey) {
            if !matches!(step.route.method, HttpMethod::Get) {
                continue;
            }
            if is_templated_http_path(step.route.path) {
                coverage.get_templated += 1;
                continue;
            }
            coverage.get_concrete += 1;
            if step.expected_status.is_some() {
                coverage.get_asserted += 1;
            } else {
                coverage.get_unasserted += 1;
                coverage
                    .unasserted
                    .push((step.journey_id, step.step_index, step.route.path));
            }
        }
    }

    coverage
}

pub const SMOKE_DEFAULT_FIXTURES: &str = "internet,pirate,advanced";

/// Journeys safe for automated mutating smoke on a live BlueOS (reversible / non-destructive).
pub const MUTATING_SMOKE_JOURNEY_IDS: &[JourneyId] = &[
    JourneyId::ChangeUiThemeColor,
    JourneyId::ResetUiThemeColor,
    JourneyId::RunLanSpeedTest,
    JourneyId::RemoveCustomLogo,
    JourneyId::RemoveCustomVehicleImage,
];

pub fn http_method_label(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
    }
}

pub fn format_http_fail(
    journey_id: JourneyId,
    step: &RunnableStep,
    resolved_url: &str,
    reason: &str,
) -> String {
    format!(
        "FAIL {journey_id} step {} {} {} → {resolved_url} — {reason}",
        step.step_index,
        http_method_label(&step.route.method),
        step.route.path
    )
}

pub fn format_dry_run(journey_id: JourneyId, step: &RunnableStep, resolved_path: &str) -> String {
    format!(
        "DRY-RUN {journey_id} step {} {} {} → {resolved_path}",
        step.step_index,
        http_method_label(&step.route.method),
        step.route.path
    )
}

pub fn journey_http_mode_conflict(smoke: bool, mutating_smoke: bool) -> Result<(), &'static str> {
    if smoke && mutating_smoke {
        return Err("--smoke and --mutating-smoke are mutually exclusive");
    }
    Ok(())
}

pub fn journey_http_requires_base(
    dry_run: bool,
    smoke: bool,
    mutating_smoke: bool,
    base: Option<&str>,
) -> Result<(), &'static str> {
    if (smoke || mutating_smoke) && base.is_none() {
        return Err("--base is required for --smoke and --mutating-smoke");
    }
    if !dry_run && base.is_none() {
        return Err("--base is required unless --dry-run is set");
    }
    Ok(())
}

pub fn http_steps(journey: &UserJourney) -> Vec<RunnableStep> {
    let GroundedSet::Known { items: steps } = &journey.steps else {
        return Vec::new();
    };

    let mut runnable = Vec::new();
    for (step_index, step) in steps.iter().enumerate() {
        if matches!(step.value.actor, Actor::Frontend(_)) {
            continue;
        }
        let Some(route) = &step.value.route else {
            continue;
        };
        let Grounded::Known { value: route, .. } = route else {
            continue;
        };
        let (expected_status, body_predicate) = match &step.value.outcome {
            None => (None, None),
            Some(Grounded::Unknown { .. }) => (None, None),
            Some(Grounded::Known { value: outcome, .. }) => {
                (outcome.expected_status, outcome.body_predicate)
            }
        };
        runnable.push(RunnableStep {
            journey_id: journey.id,
            step_index,
            route: route.clone(),
            expected_status,
            body_predicate,
            body: None,
        });
    }
    runnable
}

pub fn http_smoke_steps(journey: &UserJourney) -> Vec<RunnableStep> {
    http_steps(journey)
        .into_iter()
        .filter(|step| {
            matches!(step.route.method, HttpMethod::Get)
                && step.expected_status.is_some()
                && !is_smoke_excluded_get(step.route.path)
        })
        .collect()
}

pub fn mutating_smoke_body(journey_id: JourneyId, route: &RouteRef) -> Option<&'static str> {
    match journey_id {
        JourneyId::ChangeUiThemeColor
            if matches!(route.method, HttpMethod::Put) && route.path == "/theme" =>
        {
            Some(r##"{"primary":"#1e88e5"}"##)
        }
        _ => None,
    }
}

pub fn http_mutating_smoke_steps(journey: &UserJourney) -> Vec<RunnableStep> {
    http_steps(journey)
        .into_iter()
        .filter(|step| {
            !matches!(step.route.method, HttpMethod::Get)
                && step.expected_status.is_some()
                && !step.route.path.contains('{')
        })
        .map(|mut step| {
            step.body = mutating_smoke_body(step.journey_id, &step.route);
            step
        })
        .collect()
}

pub fn join_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let base = if base.contains("://") {
        base.to_string()
    } else {
        format!("http://{base}")
    };
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    format!("{base}{path}")
}

pub fn resolve_http_path(catalog: &Catalog, route: &RouteRef) -> Option<String> {
    let path = route.path;
    if path.contains('{') {
        return None;
    }
    if path_starts_with_service_prefix(catalog, route.service, path) {
        return Some(ensure_leading_slash(path));
    }

    let observed = catalog.observed_by_id(&route.service)?;
    let prefix = first_nginx_prefix(observed)?;

    let mut full = prefix.trim_end_matches('/').to_string();
    if let Some(version) = route.version {
        full.push('/');
        full.push_str(version);
    }
    if path.starts_with('/') {
        full.push_str(path);
    } else {
        full.push('/');
        full.push_str(path);
    }
    Some(full)
}

pub fn execute_curl(
    method: &HttpMethod,
    url: &str,
    allow_mutating: bool,
    body: Option<&str>,
) -> Result<(u16, String), String> {
    if !matches!(method, HttpMethod::Get) && !allow_mutating {
        return Err("mutating HTTP method blocked (pass --allow-mutating)".into());
    }

    let mut command = Command::new("curl");
    command.args(["-s", "-m", "30", "-w", "\n%{http_code}"]);
    match method {
        HttpMethod::Get => {}
        HttpMethod::Post => {
            command.args(["-X", "POST"]);
        }
        HttpMethod::Put => {
            command.args(["-X", "PUT"]);
        }
        HttpMethod::Delete => {
            command.args(["-X", "DELETE"]);
        }
        HttpMethod::Patch => {
            command.args(["-X", "PATCH"]);
        }
    }
    if let Some(body) = body {
        command.args(["-H", "Content-Type: application/json", "-d", body]);
    }
    command.arg(url);

    let output = command
        .output()
        .map_err(|err| format!("curl failed to start: {err}"))?;
    if !output.status.success() && output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl exited with {}: {stderr}", output.status));
    }

    let response = String::from_utf8_lossy(&output.stdout);
    let (body, status) = response
        .rsplit_once('\n')
        .ok_or_else(|| "curl response missing status line".to_string())?;
    let status_code = status
        .trim()
        .parse::<u16>()
        .map_err(|_| format!("invalid HTTP status from curl: {status}"))?;
    Ok((status_code, body.to_string()))
}

pub fn evaluate_http_response(
    status_code: u16,
    body: &str,
    expected_status: Option<u16>,
    body_predicate: Option<&str>,
) -> StepResult {
    if let Some(expected) = expected_status {
        if status_code != expected {
            return StepResult::Fail(format!("expected HTTP {expected}, got {status_code}"));
        }
        if let Some(predicate) = body_predicate {
            if let Some(needle) = predicate.strip_prefix("contains:") {
                if !body.contains(needle) {
                    return StepResult::Fail(format!("body missing expected substring: {needle}"));
                }
            }
        }
        StepResult::Pass
    } else {
        StepResult::Unasserted
    }
}

pub fn run_http_step(
    catalog: &Catalog,
    base: &str,
    step: &RunnableStep,
    allow_mutating: bool,
) -> StepResult {
    if !matches!(step.route.method, HttpMethod::Get) && !allow_mutating {
        return StepResult::Skip(format!("{:?} requires --allow-mutating", step.route.method));
    }

    let Some(path) = resolve_http_path(catalog, &step.route) else {
        return StepResult::Skip("unresolved or templated route path".into());
    };

    let url = join_url(base, &path);
    let (status_code, body) =
        match execute_curl(&step.route.method, &url, allow_mutating, step.body) {
            Ok(response) => response,
            Err(err) => return StepResult::Fail(err),
        };

    evaluate_http_response(
        status_code,
        &body,
        step.expected_status,
        step.body_predicate,
    )
}

pub fn summarize_journey(step_results: &[StepResult]) -> JourneyResult {
    if step_results.is_empty() {
        return JourneyResult::Skip;
    }
    let mut has_fail = false;
    let mut has_pass = false;
    let mut has_skip = false;
    let mut has_unasserted = false;

    for result in step_results {
        match result {
            StepResult::Fail(_) => has_fail = true,
            StepResult::Pass => has_pass = true,
            StepResult::Skip(_) | StepResult::Ignored => has_skip = true,
            StepResult::Unasserted => has_unasserted = true,
        }
    }

    if has_fail {
        JourneyResult::Fail
    } else if has_pass && (has_skip || has_unasserted) {
        JourneyResult::Partial
    } else if has_pass || has_unasserted {
        JourneyResult::Pass
    } else {
        JourneyResult::Skip
    }
}

pub fn path_starts_with_service_prefix(catalog: &Catalog, service: ServiceId, path: &str) -> bool {
    let Some(observed) = catalog.observed_by_id(&service) else {
        return false;
    };
    match &observed.nginx_prefixes {
        ObservedSet::Known { items } => items.iter().any(|prefix| {
            let prefix = prefix.value.0;
            if prefix == "/" {
                return false;
            }
            path.starts_with(prefix) || path.starts_with(prefix.trim_end_matches('/'))
        }),
        ObservedSet::Unknown { .. } => false,
    }
}

fn first_nginx_prefix(observed: &crate::observed::ObservedFacts) -> Option<&'static str> {
    match &observed.nginx_prefixes {
        ObservedSet::Known { items } => items.first().map(|prefix| prefix.value.0),
        ObservedSet::Unknown { .. } => None,
    }
}

fn ensure_leading_slash(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ServiceId;
    use crate::journey::{JourneyStep, Visibility};
    use crate::provenance::{GroundedItem, Provenance};

    const DOC: Provenance = Provenance::doc("test.md", 1);

    #[test]
    fn http_journeys_are_http_automatable() {
        let catalog = Catalog::bootstrap();
        for journey in http_journeys(&catalog) {
            assert_eq!(derive_automatable(journey), Automatable::Http);
        }
    }

    #[test]
    fn tier1_get_coverage_reports_http_journey_get_steps() {
        let catalog = Catalog::bootstrap();
        let coverage = tier1_get_coverage(&catalog);

        assert!(coverage.http_journeys > 0);
        assert_eq!(
            coverage.get_concrete,
            coverage.get_asserted + coverage.get_unasserted
        );
        assert_eq!(coverage.get_unasserted, coverage.unasserted.len());
        assert!(
            coverage.get_concrete + coverage.get_templated > 0,
            "expected at least one GET step across Http journeys"
        );
    }

    #[test]
    fn tier1_get_coverage_is_complete() {
        let catalog = Catalog::bootstrap();
        let coverage = tier1_get_coverage(&catalog);
        assert_eq!(
            coverage.get_unasserted, 0,
            "unasserted concrete GET steps: {:?}",
            coverage.unasserted
        );
    }

    #[test]
    fn tier1_get_coverage_treats_wildcard_paths_as_templated() {
        let catalog = Catalog::bootstrap();
        let coverage = tier1_get_coverage(&catalog);
        assert!(
            !coverage
                .unasserted
                .iter()
                .any(|(_, _, path)| path.contains('*')),
            "wildcard paths must not count as unasserted concrete GETs"
        );
    }

    #[test]
    fn http_smoke_steps_excludes_streaming_gets() {
        assert!(is_smoke_excluded_get("/disk/speed/stream"));
        assert!(is_smoke_excluded_get("/internet_download_speed"));
        assert!(is_smoke_excluded_get("/internet_upload_speed"));
        assert!(!is_smoke_excluded_get("/internet_best_server"));
        assert!(!is_smoke_excluded_get("/disk/speed"));
    }

    #[test]
    fn http_steps_extracts_known_routes_only() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "open tray",
                    route: None,
                    outcome: None,
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "scan",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Get,
                            path: "/scan",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "connect",
                    route: Some(Grounded::unknown("not grounded")),
                    outcome: None,
                },
                DOC,
            ),
        ];
        let journey = UserJourney {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            chains_from: None,
        };

        let steps = http_steps(&journey);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_index, 1);
        assert_eq!(steps[0].expected_status, Some(200));
        assert_eq!(steps[0].route.path, "/scan");
    }

    #[test]
    fn dry_run_planning_does_not_require_network() {
        let catalog = Catalog::bootstrap();
        let journeys = http_journeys(&catalog);
        assert!(!journeys.is_empty());

        let mut planned_steps = 0;
        for journey in &journeys {
            planned_steps += http_steps(journey).len();
        }
        assert!(planned_steps > 0);

        for journey in &journeys {
            for step in http_steps(journey) {
                let path = resolve_http_path(&catalog, &step.route);
                if step.route.path.contains('{') {
                    assert!(path.is_none());
                } else {
                    assert!(path.is_some(), "path for {:?}", step.route.path);
                }
            }
        }
    }

    #[test]
    fn join_url_normalizes_base_and_path() {
        assert_eq!(
            join_url("http://example.com/", "/wifi-manager/v1.0/scan"),
            "http://example.com/wifi-manager/v1.0/scan"
        );
        assert_eq!(
            join_url("http://example.com", "wifi-manager/v1.0/scan"),
            "http://example.com/wifi-manager/v1.0/scan"
        );
        assert_eq!(
            join_url("192.168.0.177", "/helper/v1.0/ping?host=1.1.1.1"),
            "http://192.168.0.177/helper/v1.0/ping?host=1.1.1.1"
        );
    }

    #[test]
    fn evaluate_http_response_unasserted_without_expected_status() {
        assert_eq!(
            evaluate_http_response(200, "{}", None, None),
            StepResult::Unasserted
        );
    }

    #[test]
    fn evaluate_http_response_passes_known_status_and_contains_predicate() {
        assert_eq!(
            evaluate_http_response(200, r#"{"online": true}"#, Some(200), Some("contains:true")),
            StepResult::Pass
        );
        assert!(matches!(
            evaluate_http_response(404, "{}", Some(200), None),
            StepResult::Fail(_)
        ));
    }

    #[test]
    fn format_http_fail_includes_resolved_url() {
        let step = RunnableStep {
            journey_id: JourneyId::ConnectToWifiNetwork,
            step_index: 2,
            route: RouteRef {
                service: ServiceId::Helper,
                method: HttpMethod::Get,
                path: "/ping",
                version: Some("v1.0"),
            },
            expected_status: Some(200),
            body_predicate: None,
            body: None,
        };
        let line = format_http_fail(
            JourneyId::ConnectToWifiNetwork,
            &step,
            "http://192.168.0.177/helper/v1.0/ping?host=1.1.1.1",
            "expected HTTP 200, got 404",
        );
        assert!(line.contains("→ http://192.168.0.177/helper/v1.0/ping?host=1.1.1.1"));
        assert!(line.contains("FAIL connect_to_wifi_network step 2 GET /ping"));
        assert!(line.contains("expected HTTP 200, got 404"));
    }

    #[test]
    fn format_dry_run_includes_resolved_path() {
        let step = RunnableStep {
            journey_id: JourneyId::ConnectToWifiNetwork,
            step_index: 1,
            route: RouteRef {
                service: ServiceId::Helper,
                method: HttpMethod::Get,
                path: "/ping",
                version: Some("v1.0"),
            },
            expected_status: Some(200),
            body_predicate: None,
            body: None,
        };
        let line = format_dry_run(JourneyId::ConnectToWifiNetwork, &step, "/helper/v1.0/ping");
        assert_eq!(
            line,
            "DRY-RUN connect_to_wifi_network step 1 GET /ping → /helper/v1.0/ping"
        );
    }

    #[test]
    fn journey_http_base_required_for_smoke() {
        assert!(journey_http_requires_base(false, true, false, None).is_err());
        assert!(journey_http_requires_base(false, false, true, None).is_err());
        assert!(journey_http_requires_base(true, false, false, None).is_ok());
        assert!(journey_http_requires_base(false, false, false, None).is_err());
        assert!(journey_http_requires_base(false, true, false, Some("http://pi")).is_ok());
        assert!(journey_http_requires_base(false, false, true, Some("http://pi")).is_ok());
    }

    #[test]
    fn journey_http_smoke_modes_are_mutually_exclusive() {
        assert!(journey_http_mode_conflict(true, true).is_err());
        assert!(journey_http_mode_conflict(true, false).is_ok());
        assert!(journey_http_mode_conflict(false, true).is_ok());
        assert!(journey_http_mode_conflict(false, false).is_ok());
    }

    #[test]
    fn mutating_smoke_journey_ids_are_http_automatable() {
        assert_eq!(MUTATING_SMOKE_JOURNEY_IDS.len(), 5);
        let catalog = Catalog::bootstrap();
        let journeys: std::collections::HashMap<_, _> = catalog
            .journeys()
            .iter()
            .map(|journey| (journey.id, journey))
            .collect();
        for journey_id in MUTATING_SMOKE_JOURNEY_IDS {
            let journey = journeys
                .get(journey_id)
                .unwrap_or_else(|| panic!("missing journey {journey_id}"));
            assert_eq!(derive_automatable(journey), Automatable::Http);
            assert!(
                !http_mutating_smoke_steps(journey).is_empty(),
                "{journey_id} has no mutating-smoke-eligible steps"
            );
        }
    }

    #[test]
    fn http_mutating_smoke_steps_filters_get_out() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "get scan",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Get,
                            path: "/scan",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "put theme",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Customization,
                            method: HttpMethod::Put,
                            path: "/theme",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "delete templated",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Customization,
                            method: HttpMethod::Delete,
                            path: "/models/{name}",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(204),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
        ];
        let journey = UserJourney {
            id: JourneyId::ChangeUiThemeColor,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            chains_from: None,
        };

        let steps = http_mutating_smoke_steps(&journey);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].route.path, "/theme");
        assert_eq!(steps[0].body, Some(r##"{"primary":"#1e88e5"}"##));
    }

    #[test]
    fn http_smoke_steps_only_get_with_expected_status() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "get scan",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Get,
                            path: "/scan",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "post connect",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Post,
                            path: "/connect",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "get unasserted",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Get,
                            path: "/status",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: None,
                },
                DOC,
            ),
        ];
        let journey = UserJourney {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            chains_from: None,
        };

        let steps = http_smoke_steps(&journey);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].route.path, "/scan");
    }

    #[test]
    fn resolve_http_path_builds_wifi_scan_url() {
        let catalog = Catalog::bootstrap();
        let route = RouteRef {
            service: ServiceId::Wifi,
            method: HttpMethod::Get,
            path: "/scan",
            version: Some("v1.0"),
        };
        assert_eq!(
            resolve_http_path(&catalog, &route).as_deref(),
            Some("/wifi-manager/v1.0/scan")
        );
    }
}
