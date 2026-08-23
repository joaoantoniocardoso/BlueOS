use std::collections::HashMap;
use std::fs;
use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;

use crate::drift::{DriftFinding, DriftReport};
use crate::id::ServiceId;
use crate::journey::{HttpMethod, RouteRef, UserJourney};
use crate::provenance::{Evidence, Grounded, GroundedItem, GroundedSet, Provenance};

pub const FASTAPI_ROUTE_COUNT: usize = 99;
pub const FASTAPI_METHOD_HISTOGRAM: [usize; 5] = [56, 31, 1, 11, 0];
pub const FASTAPI_SERVICE_WITH_ROUTES_COUNT: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ExtractedFastApiRoute {
    pub service_dir: String,
    pub file: String,
    pub line: usize,
    pub method: HttpMethod,
    pub path: String,
    pub version: Option<String>,
}

#[derive(Debug)]
pub enum ExtractFastApiError {
    Io(String),
    Parse(String),
}

struct RouterInfo {
    prefix: String,
    version: Option<String>,
}

pub fn extract_fastapi_from_repo(
    repo_root: &Path,
) -> Result<Vec<ExtractedFastApiRoute>, ExtractFastApiError> {
    let services_root = repo_root.join("core/services");
    let mut routes = Vec::new();
    let entries = fs::read_dir(&services_root).map_err(|err| {
        ExtractFastApiError::Io(format!("read {}: {err}", services_root.display()))
    })?;
    for entry in entries {
        let entry = entry.map_err(|err| ExtractFastApiError::Io(err.to_string()))?;
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }
        let main_py = entry.path().join("main.py");
        if !main_py.is_file() {
            continue;
        }
        let service_dir = entry.file_name().to_string_lossy().into_owned();
        let rel = format!("core/services/{service_dir}/main.py");
        let contents = fs::read_to_string(&main_py)
            .map_err(|err| ExtractFastApiError::Io(format!("read {}: {err}", main_py.display())))?;
        routes.extend(parse_fastapi_main_py(&service_dir, &rel, &contents)?);
    }
    routes.sort_by(|left, right| left.file.cmp(&right.file).then(left.line.cmp(&right.line)));
    Ok(routes)
}

fn parse_fastapi_main_py(
    service_dir: &str,
    file: &str,
    contents: &str,
) -> Result<Vec<ExtractedFastApiRoute>, ExtractFastApiError> {
    let lines: Vec<&str> = contents.lines().collect();
    let routers = collect_routers(&lines);
    let mut routes = Vec::new();
    let mut idx = 0usize;
    while idx < lines.len() {
        let line_number = idx + 1;
        let line = lines[idx];
        if let Some((router_var, method)) = parse_route_decorator_start(line) {
            let (path, end_idx) = parse_route_path(&lines, idx).ok_or_else(|| {
                ExtractFastApiError::Parse(format!(
                    "{file}:{line_number}: unterminated route decorator path"
                ))
            })?;
            let router = routers.get(&router_var);
            let version = router
                .and_then(|router| router.version.clone())
                .or_else(|| scan_version_directive(&lines, end_idx + 1));
            let full_path = join_router_path(router.map(|router| router.prefix.as_str()), &path);
            routes.push(ExtractedFastApiRoute {
                service_dir: service_dir.to_string(),
                file: file.to_string(),
                line: line_number,
                method,
                path: full_path,
                version,
            });
            idx = end_idx + 1;
            continue;
        }
        idx += 1;
    }
    Ok(routes)
}

fn collect_routers(lines: &[&str]) -> HashMap<String, RouterInfo> {
    let mut routers = HashMap::new();
    let mut idx = 0usize;
    while idx < lines.len() {
        let line = lines[idx];
        let Some(name) = router_assignment_name(line) else {
            idx += 1;
            continue;
        };
        let block_end = find_paren_block_end(lines, idx);
        let block = lines[idx..=block_end.min(lines.len() - 1)].join(" ");
        if !block.contains("APIRouter(") {
            idx += 1;
            continue;
        }
        let prefix = extract_quoted_after(&block, "prefix=").unwrap_or_else(|| "/".to_string());
        let version = block
            .find("versioned_api_route(")
            .and_then(|start| parse_version_pair(&block[start + "versioned_api_route(".len()..]));
        routers.insert(name.to_string(), RouterInfo { prefix, version });
        idx = block_end + 1;
    }
    routers
}

fn router_assignment_name(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let eq = trimmed.find('=')?;
    let name = trimmed[..eq].trim();
    if name.ends_with("_router") {
        Some(name)
    } else {
        None
    }
}

fn find_paren_block_end(lines: &[&str], start: usize) -> usize {
    let mut depth = 0i32;
    let mut saw_open = false;
    for (offset, line) in lines.iter().enumerate().skip(start) {
        for ch in line.chars() {
            if ch == '(' {
                depth += 1;
                saw_open = true;
            } else if ch == ')' {
                depth -= 1;
                if saw_open && depth == 0 {
                    return offset;
                }
            }
        }
    }
    lines.len().saturating_sub(1)
}

fn parse_route_decorator_start(line: &str) -> Option<(String, HttpMethod)> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix('@')?;
    let dot = rest.find('.')?;
    let router_var = rest[..dot].to_string();
    let method_part = &rest[dot + 1..];
    let paren = method_part.find('(')?;
    let method = match method_part[..paren].trim() {
        "get" => HttpMethod::Get,
        "post" => HttpMethod::Post,
        "put" => HttpMethod::Put,
        "delete" => HttpMethod::Delete,
        "patch" => HttpMethod::Patch,
        _ => return None,
    };
    if router_var != "app" && router_var != "fast_api_app" && !router_var.ends_with("_router") {
        return None;
    }
    Some((router_var, method))
}

fn parse_route_path(lines: &[&str], start: usize) -> Option<(String, usize)> {
    let mut buffer = String::new();
    for (idx, line) in lines.iter().enumerate().skip(start).take(12) {
        buffer.push(' ');
        buffer.push_str(line.trim());
        if let Some(path) = first_quoted_string(&buffer) {
            return Some((path, idx));
        }
    }
    None
}

fn scan_version_directive(lines: &[&str], start: usize) -> Option<String> {
    for line in lines.iter().skip(start).take(6) {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("@version(") {
            return parse_version_pair(rest);
        }
        if trimmed.starts_with('@')
            && !trimmed.starts_with("@version(")
            && !trimmed.starts_with("@to_http_exception")
            && !trimmed.starts_with("@temporary_cache")
            && !trimmed.starts_with("@cache")
        {
            break;
        }
    }
    None
}

fn parse_version_pair(input: &str) -> Option<String> {
    let numbers: Vec<u32> = input
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect();
    if numbers.len() >= 2 {
        Some(format!("v{}.{}", numbers[0], numbers[1]))
    } else {
        None
    }
}

fn join_router_path(prefix: Option<&str>, path: &str) -> String {
    let Some(prefix) = prefix.filter(|prefix| !prefix.is_empty()) else {
        return normalize_path(path);
    };
    if path.is_empty() {
        return normalize_path(prefix);
    }
    format!(
        "{}{}",
        normalize_path(prefix).trim_end_matches('/'),
        normalize_path(path)
    )
}

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn first_quoted_string(input: &str) -> Option<String> {
    let quote = input.find(['"', '\''])?;
    let rest = &input[quote + 1..];
    let quote_ch = input.as_bytes()[quote] as char;
    let end = rest.find(quote_ch)?;
    Some(rest[..end].to_string())
}

fn extract_quoted_after(input: &str, key: &str) -> Option<String> {
    let start = input.find(key)? + key.len();
    let rest = input[start..].trim_start();
    first_quoted_string(rest)
}

pub fn fastapi_method_histogram(routes: &[ExtractedFastApiRoute]) -> [usize; 5] {
    let mut counts = [0usize; 5];
    for route in routes {
        let idx = match route.method {
            HttpMethod::Get => 0,
            HttpMethod::Post => 1,
            HttpMethod::Put => 2,
            HttpMethod::Delete => 3,
            HttpMethod::Patch => 4,
        };
        counts[idx] += 1;
    }
    counts
}

pub fn fastapi_service_histogram(routes: &[ExtractedFastApiRoute]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for route in routes {
        *counts.entry(route.service_dir.clone()).or_insert(0) += 1;
    }
    counts
}

pub fn find_fastapi_route_at_line<'a>(
    routes: &'a [ExtractedFastApiRoute],
    file: &str,
    line: u32,
) -> Option<&'a ExtractedFastApiRoute> {
    routes
        .iter()
        .find(|route| route.file == file && route.line == line as usize)
}

pub fn find_fastapi_route_for_observed<'a>(
    routes: &'a [ExtractedFastApiRoute],
    evidence: &Evidence,
    observed: &RouteRef,
) -> Option<&'a ExtractedFastApiRoute> {
    if let Some(route) = find_fastapi_route_at_line(routes, evidence.file, evidence.line) {
        return Some(route);
    }
    let observed_path = strip_query(observed.path);
    routes.iter().find(|route| {
        route.file == evidence.file
            && route.method == observed.method
            && strip_fastapi_path_converter(&route.path) == observed_path
            && route.line.abs_diff(evidence.line as usize) <= 12
    })
}

pub(crate) fn extracted_fastapi_matches_observed(
    extracted: &ExtractedFastApiRoute,
    observed: &RouteRef,
) -> bool {
    extracted.method == observed.method
        && strip_query(observed.path) == strip_fastapi_path_converter(&extracted.path)
        && extracted.version.as_deref() == observed.version
        && extracted.service_dir == service_dir_for_id(observed.service)
}

pub fn check_fastapi_against_observed(
    extracted: &[ExtractedFastApiRoute],
    journeys: &[UserJourney],
) -> DriftReport {
    let mut findings = Vec::new();
    check_journey_fastapi_routes(extracted, journeys, &mut findings);
    DriftReport { findings }
}

fn check_journey_fastapi_routes(
    extracted: &[ExtractedFastApiRoute],
    journeys: &[UserJourney],
    findings: &mut Vec<DriftFinding>,
) {
    for journey in journeys {
        let Some(steps) = journey_steps(journey) else {
            continue;
        };
        for step in steps {
            let Some(Grounded::Known {
                value: route,
                provenance: Provenance::Source(evidence),
            }) = &step.value.route
            else {
                continue;
            };
            if !evidence_from_main_py(evidence) {
                continue;
            }
            let Some(extracted_route) = find_fastapi_route_for_observed(extracted, evidence, route)
            else {
                findings.push(DriftFinding {
                    field: format!("{}.step.route", journey.id.as_str()),
                    message: format!(
                        "observed {} {} {:?} ({}:{}), no extracted FastAPI route at cited line",
                        http_method_label(&route.method),
                        route.path,
                        route.version,
                        evidence.file,
                        evidence.line
                    ),
                });
                continue;
            };
            compare_fastapi_route(
                journey.id.as_str(),
                route,
                evidence,
                extracted_route,
                findings,
            );
        }
    }
}

fn compare_fastapi_route(
    journey_id: &str,
    observed: &RouteRef,
    evidence: &Evidence,
    extracted: &ExtractedFastApiRoute,
    findings: &mut Vec<DriftFinding>,
) {
    if observed.method != extracted.method {
        findings.push(DriftFinding {
            field: format!("{journey_id}.step.route"),
            message: format!(
                "observed method {:?} ({}:{}), extracted {:?}",
                observed.method, evidence.file, evidence.line, extracted.method
            ),
        });
    }
    let observed_path = strip_query(observed.path);
    if observed_path != strip_fastapi_path_converter(&extracted.path) {
        findings.push(DriftFinding {
            field: format!("{journey_id}.step.route"),
            message: format!(
                "observed path {observed_path:?} ({}:{}), extracted path {:?}",
                evidence.file, evidence.line, extracted.path
            ),
        });
    }
    if observed.version != extracted.version.as_deref() {
        findings.push(DriftFinding {
            field: format!("{journey_id}.step.route"),
            message: format!(
                "observed version {:?} ({}:{}), extracted version {:?}",
                observed.version, evidence.file, evidence.line, extracted.version
            ),
        });
    }
    let expected_dir = service_dir_for_id(observed.service);
    if extracted.service_dir != expected_dir {
        findings.push(DriftFinding {
            field: format!("{journey_id}.step.route"),
            message: format!(
                "observed service {:?} ({}:{}), extracted service_dir {:?}",
                observed.service, evidence.file, evidence.line, extracted.service_dir
            ),
        });
    }
}

fn strip_query(path: &str) -> &str {
    path.split('?').next().unwrap_or(path)
}

fn is_path_ident(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub(crate) fn strip_fastapi_path_converter(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    let mut chars = path.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' {
            out.push(ch);
            continue;
        }
        let mut inner = String::new();
        let mut closed = false;
        for next in chars.by_ref() {
            if next == '}' {
                closed = true;
                break;
            }
            inner.push(next);
        }
        if !closed {
            out.push('{');
            out.push_str(&inner);
            continue;
        }
        match inner.split_once(':') {
            Some((name, converter))
                if is_path_ident(name) && is_path_ident(converter) && !converter.is_empty() =>
            {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
            _ => {
                out.push('{');
                out.push_str(&inner);
                out.push('}');
            }
        }
    }
    out
}

fn service_dir_for_id(service: ServiceId) -> &'static str {
    match service {
        ServiceId::Wifi => "wifi",
        ServiceId::Ping => "ping",
        ServiceId::RecorderExtractor => "recorder_extractor",
        ServiceId::NmeaInjector => "nmea_injector",
        ServiceId::Helper => "helper",
        ServiceId::DiskUsage => "disk_usage",
        ServiceId::Customization => "customization",
        ServiceId::CableGuy => "cable_guy",
        ServiceId::Commander => "commander",
        ServiceId::Bridget => "bridget",
        ServiceId::Beacon => "beacon",
        ServiceId::BagOfHolding => "bag_of_holding",
        ServiceId::Pardal => "pardal",
        other => other.as_str(),
    }
}

fn http_method_label(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
    }
}

fn journey_steps(journey: &UserJourney) -> Option<&[GroundedItem<crate::journey::JourneyStep>]> {
    match &journey.steps {
        GroundedSet::Known { items } => Some(items),
        GroundedSet::Unknown { .. } => None,
    }
}

pub(crate) fn evidence_from_main_py(evidence: &Evidence) -> bool {
    evidence.file.starts_with("core/services/")
        && evidence.file.ends_with("/main.py")
        && (evidence.anchor.contains("@app.")
            || evidence.anchor.contains("_router.")
            || evidence.anchor.contains("@fast_api_app.")
            || evidence.anchor.contains("async def")
            || evidence.anchor.contains("def "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Catalog;

    #[test]
    fn extract_pins_fastapi_route_count() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_fastapi_from_repo(repo_root).expect("extract fastapi");
        assert_eq!(extracted.len(), FASTAPI_ROUTE_COUNT);
        assert_eq!(
            fastapi_method_histogram(&extracted),
            FASTAPI_METHOD_HISTOGRAM
        );
        assert_eq!(
            fastapi_service_histogram(&extracted).len(),
            FASTAPI_SERVICE_WITH_ROUTES_COUNT
        );
    }

    #[test]
    fn customization_branding_router_applies_prefix() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let contents =
            std::fs::read_to_string(repo_root.join("core/services/customization/main.py")).unwrap();
        let routes = parse_fastapi_main_py(
            "customization",
            "core/services/customization/main.py",
            &contents,
        )
        .unwrap();
        let post_logo = routes
            .iter()
            .find(|route| route.line == 273)
            .expect("line 273 route");
        assert_eq!(post_logo.path, "/branding/logo");
        assert_eq!(post_logo.version.as_deref(), Some("v1.0"));
    }

    #[test]
    fn extract_wifi_scan_maps_to_get_v1() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_fastapi_from_repo(repo_root).expect("extract fastapi");
        let scan = extracted
            .iter()
            .find(|route| route.service_dir == "wifi" && route.path == "/scan")
            .expect("wifi /scan route");
        assert_eq!(scan.line, 63);
        assert_eq!(scan.method, HttpMethod::Get);
        assert_eq!(scan.version.as_deref(), Some("v1.0"));
    }

    #[test]
    fn scan_version_reads_following_line() {
        let lines = vec!["@app.get(\"/scan\")", "@version(1, 0)"];
        assert_eq!(scan_version_directive(&lines, 1), Some("v1.0".to_string()));
    }

    #[test]
    fn bootstrap_fastapi_cross_check_runs() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_fastapi_from_repo(repo_root).expect("extract fastapi");
        assert!(!extracted.is_empty());
        let catalog = Catalog::bootstrap();
        let report = check_fastapi_against_observed(&extracted, catalog.journeys());
        assert!(
            !report.has_drift(),
            "expected no FastAPI drift, got {:?}",
            report.findings
        );
    }

    #[test]
    fn path_converter_strips_ident_suffix_only() {
        assert_eq!(
            strip_fastapi_path_converter("/models/{name:path}"),
            "/models/{name}"
        );
        assert_eq!(
            strip_fastapi_path_converter("/disk/paths/{target_path:path}"),
            "/disk/paths/{target_path}"
        );
        assert_eq!(
            strip_fastapi_path_converter("/recorder/files/{filename:path}"),
            "/recorder/files/{filename}"
        );
        assert_eq!(
            strip_fastapi_path_converter("/models/{name}"),
            "/models/{name}"
        );
        assert_ne!(
            strip_fastapi_path_converter("/models/{name:path}"),
            "/models/{filename}"
        );
        assert_ne!(
            strip_fastapi_path_converter("/models/{name:path}"),
            "/models/{name}/extra"
        );
        assert_eq!(
            strip_fastapi_path_converter("/models/{name:path:extra}"),
            "/models/{name:path:extra}"
        );
        assert_eq!(strip_fastapi_path_converter("foo:{bar}"), "foo:{bar}");
    }

    #[test]
    fn extract_requires_fastapi_decorator_prefix() {
        let contents = "@bogus.get(\"/scan\")\n@version(1, 0)\n";
        let extracted = parse_fastapi_main_py("wifi", "core/services/wifi/main.py", contents)
            .expect("parse synthetic main.py");
        assert_eq!(extracted.len(), 0);
    }

    #[test]
    fn perturbed_fastapi_path_reports_drift() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut extracted = extract_fastapi_from_repo(repo_root).expect("extract fastapi");
        let scan = extracted
            .iter_mut()
            .find(|route| route.service_dir == "wifi" && route.line == 63)
            .expect("wifi scan line");
        scan.path = "/scanned".to_string();
        let catalog = Catalog::bootstrap();
        let report = check_fastapi_against_observed(&extracted, catalog.journeys());
        assert!(
            report.has_drift(),
            "expected drift, got {:?}",
            report.findings
        );
        assert!(report.findings.iter().any(|finding| {
            finding.message.contains("/scanned") || finding.message.contains("/scan")
        }));
    }

    #[test]
    fn fastapi_window_is_two_sided() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_fastapi_from_repo(repo_root).expect("extract fastapi");
        let observed = RouteRef {
            service: crate::id::ServiceId::Ping,
            method: HttpMethod::Get,
            path: "/sensors",
            version: Some("v1.0"),
        };
        let file = "core/services/ping/main.py";
        let on_decorator = Evidence {
            file,
            line: 38,
            anchor: "@app.get(\"/sensors\", response_m",
        };
        assert!(find_fastapi_route_for_observed(&extracted, &on_decorator, &observed).is_some());
        let shebang = Evidence {
            file,
            line: 1,
            anchor: "#! /usr/bin/env python3",
        };
        assert!(
            find_fastapi_route_for_observed(&extracted, &shebang, &observed).is_none(),
            "citation before the route must be outside the window"
        );
        let far_after = Evidence {
            file,
            line: 999,
            anchor: "not-a-route",
        };
        assert!(find_fastapi_route_for_observed(&extracted, &far_after, &observed).is_none());
        let nearby_fn = Evidence {
            file,
            line: 40,
            anchor: "def get_sensors() -> Any:",
        };
        assert!(find_fastapi_route_for_observed(&extracted, &nearby_fn, &observed).is_some());
    }
}
