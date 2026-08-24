use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::Serialize;

use crate::drift::{DriftFinding, DriftReport};
use crate::id::ServiceId;
use crate::journey::{HttpMethod, RouteRef, UseCase};
use crate::provenance::{Evidence, Grounded, GroundedItem, GroundedSet, Provenance};

// Counts cover every Python HTTP route in core/services, which includes pardal's aiohttp handlers
// alongside the FastAPI ones.
pub const PYTHON_ROUTE_COUNT: usize = 201;
pub const PYTHON_ROUTE_METHOD_HISTOGRAM: [usize; 5] = [107, 69, 8, 17, 0];
pub const PYTHON_ROUTE_SERVICE_COUNT: usize = 16;

// Route owners are discovered by constructor, not by variable name: `router = APIRouter()` is the
// documented FastAPI idiom and would be invisible to a name-based heuristic. `VersionedFastAPI(`
// matches via `FastAPI(`. `RouteTableDef(` covers the aiohttp services (pardal).
const ROUTE_OWNER_CONSTRUCTORS: [&str; 3] = ["APIRouter(", "FastAPI(", "RouteTableDef("];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ExtractedFastApiRoute {
    pub service_dir: String,
    pub file: String,
    pub line: usize,
    pub method: HttpMethod,
    pub path: String,
    pub version: Option<String>,
    pub status_code: Option<u16>,
    pub response_model: Option<String>,
}

#[derive(Debug)]
pub enum ExtractFastApiError {
    Io(String),
    Parse(String),
}

struct RouterInfo {
    prefix: String,
    version: Option<String>,
    is_app: bool,
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
        let service_dir = entry.file_name().to_string_lossy().into_owned();
        let mut sources = Vec::new();
        collect_python_sources(&entry.path(), &mut sources)?;
        for source in sources {
            let rel = source
                .strip_prefix(repo_root)
                .unwrap_or(&source)
                .to_string_lossy()
                .into_owned();
            let contents = fs::read_to_string(&source)
                .map_err(|err| ExtractFastApiError::Io(format!("read {rel}: {err}")))?;
            routes.extend(parse_fastapi_source(&service_dir, &rel, &contents)?);
        }
    }
    routes.sort_by(|left, right| left.file.cmp(&right.file).then(left.line.cmp(&right.line)));
    Ok(routes)
}

fn collect_python_sources(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), ExtractFastApiError> {
    let entries = fs::read_dir(dir)
        .map_err(|err| ExtractFastApiError::Io(format!("read {}: {err}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|err| ExtractFastApiError::Io(err.to_string()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| ExtractFastApiError::Io(format!("{}: {err}", path.display())))?;
        if file_type.is_dir() {
            if is_skip_dir(&path) {
                continue;
            }
            collect_python_sources(&path, out)?;
        } else if file_type.is_file() && is_python_source(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(
            "__pycache__"
                | "__tests__"
                | ".venv"
                | "node_modules"
                | "site-packages"
                | "test"
                | "tests"
                | "venv"
        )
    )
}

fn is_python_source(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.ends_with(".py") && !name.starts_with("test_") && !name.ends_with("_test.py")
}

fn parse_fastapi_source(
    service_dir: &str,
    file: &str,
    contents: &str,
) -> Result<Vec<ExtractedFastApiRoute>, ExtractFastApiError> {
    let lines: Vec<&str> = contents.lines().collect();
    let routers = collect_routers(&lines);
    let wrap_line = versioned_fastapi_wrap_line(&lines);
    let default_version = default_versioned_fastapi_version(&lines);
    let mut routes = Vec::new();
    let mut idx = 0usize;
    while idx < lines.len() {
        let line_number = idx + 1;
        let line = lines[idx];
        if let Some((owner, method)) = parse_route_decorator_start(line) {
            let Some(router) = routers.get(&owner) else {
                return Err(ExtractFastApiError::Parse(format!(
                    "{file}:{line_number}: route decorator on unknown owner {owner:?}"
                )));
            };
            let (path, status_code, response_model, end_idx) = parse_decorator_block(&lines, idx)
                .ok_or_else(|| {
                ExtractFastApiError::Parse(format!(
                    "{file}:{line_number}: unterminated route decorator"
                ))
            })?;
            let mut version = router
                .version
                .clone()
                .or_else(|| scan_version_directive(&lines, end_idx + 1));
            if version.is_none() && router.is_app && wrap_line.is_some_and(|wrap| idx < wrap) {
                version = default_version.clone();
            }
            routes.push(ExtractedFastApiRoute {
                service_dir: service_dir.to_string(),
                file: file.to_string(),
                line: line_number,
                method,
                path: join_router_path(Some(router.prefix.as_str()), &path),
                version,
                status_code,
                response_model,
            });
            idx = end_idx + 1;
            continue;
        }
        if let Some((method, path)) = parse_router_add_call(line) {
            routes.push(ExtractedFastApiRoute {
                service_dir: service_dir.to_string(),
                file: file.to_string(),
                line: line_number,
                method,
                path,
                version: None,
                status_code: None,
                response_model: None,
            });
        }
        idx += 1;
    }
    Ok(routes)
}

fn collect_routers(lines: &[&str]) -> HashMap<String, RouterInfo> {
    let mut routers = HashMap::new();
    let mut idx = 0usize;
    while idx < lines.len() {
        let Some(name) = assignment_target_name(lines[idx]) else {
            idx += 1;
            continue;
        };
        let Some(block_end) = find_paren_block_end(lines, idx) else {
            idx += 1;
            continue;
        };
        let block = lines[idx..=block_end].join(" ");
        if !ROUTE_OWNER_CONSTRUCTORS
            .iter()
            .any(|constructor| block.contains(constructor))
        {
            idx += 1;
            continue;
        }
        let prefix = extract_quoted_after(&block, "prefix=").unwrap_or_else(|| "/".to_string());
        let version = block
            .find("versioned_api_route(")
            .and_then(|start| parse_version_pair(&block[start + "versioned_api_route(".len()..]));
        routers.insert(
            name.to_string(),
            RouterInfo {
                prefix,
                version,
                is_app: !block.contains("APIRouter("),
            },
        );
        idx = block_end + 1;
    }
    routers
}

fn assignment_target_name(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let eq = trimmed.find('=')?;
    if trimmed[eq + 1..].starts_with('=') {
        return None;
    }
    let name = trimmed[..eq].trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }
    Some(name)
}

/// Index of the line closing the parenthesis opened on `start`, or `None` when `start` opens no
/// parenthesis at all. Quoted text and `#` comments are skipped so a `(` in a string or a comment
/// does not shift the depth.
fn find_paren_block_end(lines: &[&str], start: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (offset, line) in lines.iter().enumerate().skip(start) {
        let mut quote: Option<char> = None;
        let mut escaped = false;
        for ch in line.chars() {
            if let Some(open_quote) = quote {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == open_quote {
                    quote = None;
                }
                continue;
            }
            match ch {
                '#' => break,
                '"' | '\'' => quote = Some(ch),
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(offset);
                    }
                }
                _ => {}
            }
        }
        if depth == 0 {
            return None;
        }
    }
    None
}

fn parse_route_decorator_start(line: &str) -> Option<(String, HttpMethod)> {
    let rest = line.trim().strip_prefix('@')?;
    let dot = rest.find('.')?;
    let method_part = &rest[dot + 1..];
    let paren = method_part.find('(')?;
    let method = http_method_from_lowercase(method_part[..paren].trim())?;
    Some((rest[..dot].to_string(), method))
}

/// aiohttp services register handlers imperatively (`app.router.add_get("/get_file", handler)`)
/// instead of by decorator, so those call sites need their own shape.
fn parse_router_add_call(line: &str) -> Option<(HttpMethod, String)> {
    let trimmed = line.trim();
    let add = trimmed.find(".add_")?;
    let rest = &trimmed[add + ".add_".len()..];
    let paren = rest.find('(')?;
    let method = http_method_from_lowercase(&rest[..paren])?;
    Some((method, normalize_path(&first_quoted_string(rest)?)))
}

fn http_method_from_lowercase(name: &str) -> Option<HttpMethod> {
    match name {
        "get" => Some(HttpMethod::Get),
        "post" => Some(HttpMethod::Post),
        "put" => Some(HttpMethod::Put),
        "delete" => Some(HttpMethod::Delete),
        "patch" => Some(HttpMethod::Patch),
        _ => None,
    }
}

fn parse_decorator_block(
    lines: &[&str],
    start: usize,
) -> Option<(String, Option<u16>, Option<String>, usize)> {
    let end = find_paren_block_end(lines, start)?;
    let block = lines[start..=end].join(" ");
    let path = first_quoted_string(&block)?;
    Some((
        path,
        parse_status_code(&block),
        parse_response_model(&block),
        end,
    ))
}

fn parse_status_code(block: &str) -> Option<u16> {
    let start = block.find("status_code=")?;
    let value = top_level_value(&block[start + "status_code=".len()..]).trim();
    // `status.HTTP_204_NO_CONTENT` and a bare `204` are both idiomatic.
    let digits = match value.find("HTTP_") {
        Some(offset) => &value[offset + "HTTP_".len()..],
        None => value,
    };
    digits
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
}

fn parse_response_model(block: &str) -> Option<String> {
    let start = block.find("response_model=")?;
    let model = top_level_value(&block[start + "response_model=".len()..]).trim();
    if model.is_empty() {
        None
    } else {
        Some(model.to_string())
    }
}

/// Slices a keyword argument's value: everything up to the comma or closing paren that sits at
/// bracket depth zero, so `Dict[str, Any]` and `Response(status_code=1)` stay intact.
fn top_level_value(rest: &str) -> &str {
    let mut depth = 0i32;
    for (idx, ch) in rest.char_indices() {
        match ch {
            '[' | '(' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            ')' if depth == 0 => return &rest[..idx],
            ')' => depth -= 1,
            ',' if depth == 0 => return &rest[..idx],
            _ => {}
        }
    }
    rest
}

fn versioned_fastapi_wrap_line(lines: &[&str]) -> Option<usize> {
    lines
        .iter()
        .position(|line| line.contains("VersionedFastAPI("))
}

fn default_versioned_fastapi_version(lines: &[&str]) -> Option<String> {
    let start = versioned_fastapi_wrap_line(lines)?;
    let end = find_paren_block_end(lines, start)?;
    let block = lines[start..=end].join(" ");
    if let Some(offset) = block.find("default_version=") {
        return parse_version_pair(&block[offset + "default_version=".len()..]);
    }
    Some("v1.0".to_string())
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
    let observed_path = normalized_route_path(observed.path);
    routes.iter().find(|route| {
        route.file == evidence.file
            && route.method == observed.method
            && normalized_route_path(&route.path) == observed_path
            && route.line.abs_diff(evidence.line as usize) <= 12
    })
}

pub(crate) fn extracted_fastapi_matches_observed(
    extracted: &ExtractedFastApiRoute,
    observed: &RouteRef,
) -> bool {
    extracted.method == observed.method
        && normalized_route_path(observed.path) == normalized_route_path(&extracted.path)
        && extracted.version.as_deref() == observed.version
        && extracted.service_dir == service_dir_for_id(observed.service)
}

pub fn check_fastapi_against_observed(
    extracted: &[ExtractedFastApiRoute],
    journeys: &[UseCase],
) -> DriftReport {
    let mut findings = Vec::new();
    check_journey_fastapi_routes(extracted, journeys, &mut findings);
    DriftReport { findings }
}

fn check_journey_fastapi_routes(
    extracted: &[ExtractedFastApiRoute],
    journeys: &[UseCase],
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
            if !evidence_from_fastapi_source(evidence) {
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
    let observed_path = normalized_route_path(observed.path);
    if observed_path != normalized_route_path(&extracted.path) {
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

pub(crate) fn strip_query(path: &str) -> &str {
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

/// Comparison form for a route path: query stripped, `{name:path}` reduced to `{name}`, leading
/// slash forced, trailing slashes dropped. `/endpoints/` and `/endpoints` are the same endpoint.
pub(crate) fn normalized_route_path(path: &str) -> String {
    let stripped = strip_fastapi_path_converter(strip_query(path));
    let mut out = if stripped.starts_with('/') {
        stripped
    } else {
        format!("/{stripped}")
    };
    while out.len() > 1 && out.ends_with('/') {
        out.pop();
    }
    out
}

pub(crate) fn service_dir_for_id(service: ServiceId) -> &'static str {
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

fn journey_steps(journey: &UseCase) -> Option<&[GroundedItem<crate::journey::JourneyStep>]> {
    match &journey.steps {
        GroundedSet::Known { items } => Some(items),
        GroundedSet::Unknown { .. } => None,
    }
}

pub(crate) fn evidence_from_fastapi_source(evidence: &Evidence) -> bool {
    evidence.file.starts_with("core/services/")
        && evidence.file.ends_with(".py")
        && !is_test_python_file(evidence.file)
        && (evidence.anchor.contains("@app.")
            || evidence.anchor.contains("_router")
            || evidence.anchor.contains("@fast_api_app.")
            || evidence.anchor.contains("@application.")
            || evidence.anchor.contains("async def")
            || evidence.anchor.contains("def "))
}

fn is_test_python_file(file: &str) -> bool {
    file.rsplit('/')
        .next()
        .is_some_and(|name| name.starts_with("test_") || name.ends_with("_test.py"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Catalog;

    #[test]
    fn extract_pins_fastapi_route_count() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_fastapi_from_repo(repo_root).expect("extract fastapi");
        assert_eq!(extracted.len(), PYTHON_ROUTE_COUNT);
        assert_eq!(
            fastapi_method_histogram(&extracted),
            PYTHON_ROUTE_METHOD_HISTOGRAM
        );
        assert_eq!(
            fastapi_service_histogram(&extracted).len(),
            PYTHON_ROUTE_SERVICE_COUNT
        );
    }

    #[test]
    fn extract_includes_split_router_services() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_fastapi_from_repo(repo_root).expect("extract fastapi");
        let endpoints_post = extracted
            .iter()
            .find(|route| {
                route.service_dir == "ardupilot_manager"
                    && route.method == HttpMethod::Post
                    && route.path == "/endpoints/"
            })
            .expect("ardupilot_manager POST /endpoints/");
        assert_eq!(endpoints_post.version.as_deref(), Some("v1.0"));
        assert_eq!(endpoints_post.status_code, Some(201));
        assert!(extracted
            .iter()
            .any(|route| route.service_dir == "kraken" && route.file.contains("/api/")));
        assert!(extracted
            .iter()
            .any(|route| route.service_dir == "versionchooser" && route.file.contains("/api/")));
    }

    #[test]
    fn pre_wrap_unversioned_app_route_inherits_default_version() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_fastapi_from_repo(repo_root).expect("extract fastapi");
        let overwrite = extracted
            .iter()
            .find(|route| {
                route.service_dir == "bag_of_holding"
                    && route.method == HttpMethod::Post
                    && route.path == "/overwrite"
            })
            .expect("bag_of_holding POST /overwrite");
        assert_eq!(overwrite.version.as_deref(), Some("v1.0"));
        let root = extracted
            .iter()
            .find(|route| {
                route.service_dir == "bag_of_holding"
                    && route.method == HttpMethod::Get
                    && route.path == "/"
            })
            .expect("bag_of_holding GET /");
        assert_eq!(root.version, None);
    }

    #[test]
    fn customization_branding_router_applies_prefix() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let contents =
            std::fs::read_to_string(repo_root.join("core/services/customization/main.py")).unwrap();
        let routes = parse_fastapi_source(
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
    fn route_decorator_on_undiscovered_owner_is_an_error() {
        let contents = "@bogus.get(\"/scan\")\n@version(1, 0)\n";
        let error = parse_fastapi_source("wifi", "core/services/wifi/main.py", contents)
            .expect_err("undiscovered owner must not be skipped silently");
        let ExtractFastApiError::Parse(message) = error else {
            panic!("expected a parse error");
        };
        assert!(message.contains("unknown owner"), "{message}");
    }

    #[test]
    fn owners_are_discovered_by_constructor_not_by_variable_name() {
        // `router = APIRouter()` is the name FastAPI's own docs use, and it matches none of the
        // `_router` suffix conventions the rest of core/services happens to follow.
        let contents = concat!(
            "router = APIRouter(prefix=\"/tools\", route_class=versioned_api_route(1, 0))\n",
            "\n",
            "@router.get(\"/scan\")\n",
            "async def scan() -> None:\n",
        );
        let extracted = parse_fastapi_source("wifi", "core/services/wifi/main.py", contents)
            .expect("parse synthetic main.py");
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].path, "/tools/scan");
        assert_eq!(extracted[0].version.as_deref(), Some("v1.0"));
    }

    #[test]
    fn aiohttp_route_table_and_imperative_registration_are_extracted() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_fastapi_from_repo(repo_root).expect("extract routes");
        let pardal: Vec<&ExtractedFastApiRoute> = extracted
            .iter()
            .filter(|route| route.service_dir == "pardal")
            .collect();
        // Four `@routes.get` decorators on a RouteTableDef plus three `app.router.add_*` calls.
        assert_eq!(pardal.len(), 7, "{pardal:?}");
        assert!(pardal.iter().any(
            |route| route.path == "/internet_download_speed" && route.method == HttpMethod::Get
        ));
        assert!(pardal
            .iter()
            .any(|route| route.path == "/post_file" && route.method == HttpMethod::Post));
    }

    #[test]
    fn paren_counting_ignores_parens_in_strings_and_comments() {
        let lines = [
            "foo = APIRouter(  # opening ( in a comment",
            "    \")(\",",
            ")",
        ];
        assert_eq!(find_paren_block_end(&lines, 0), Some(2));
        assert_eq!(find_paren_block_end(&["name = 1"], 0), None);
    }

    #[test]
    fn keyword_values_stop_at_the_top_level_separator() {
        assert_eq!(
            top_level_value("Dict[str, Any], summary=\"x\""),
            "Dict[str, Any]"
        );
        assert_eq!(top_level_value("None)"), "None");
        assert_eq!(
            parse_status_code(
                "@r.post(\"/x\", response_model=Foo, status_code=status.HTTP_201_CREATED)"
            ),
            Some(201)
        );
        // A later `HTTP_` must not be mistaken for this argument's value.
        assert_eq!(
            parse_status_code(
                "@r.get(\"/x\", status_code=204, responses={status.HTTP_404_NOT_FOUND: {}})"
            ),
            Some(204)
        );
        assert_eq!(
            parse_response_model("@r.get(\"/x\", response_model=Dict[str, Any], summary=\"s\")"),
            Some("Dict[str, Any]".to_string())
        );
    }

    #[test]
    fn trailing_slash_does_not_split_a_route_identity() {
        assert_eq!(normalized_route_path("/endpoints/"), "/endpoints");
        assert_eq!(normalized_route_path("/endpoints"), "/endpoints");
        assert_eq!(normalized_route_path("/"), "/");
        assert_eq!(normalized_route_path("files?x=1"), "/files");
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
