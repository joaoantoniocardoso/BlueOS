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
        });
    }
    runnable
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
    let (status_code, body) = match execute_curl(&step.route.method, &url, allow_mutating) {
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

fn path_starts_with_service_prefix(catalog: &Catalog, service: ServiceId, path: &str) -> bool {
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
