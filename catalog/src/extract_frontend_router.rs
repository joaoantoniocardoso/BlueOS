use std::fs;
use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;

use crate::drift::{DriftFinding, DriftReport};
use crate::page::Page;
use crate::provenance::{Evidence, Observed};

pub const FRONTEND_ROUTER_ROUTE_COUNT: usize = 29;
pub const FRONTEND_MENU_ENTRY_COUNT: usize = 21;
pub const FRONTEND_ROUTE_KIND_HISTOGRAM: [usize; 3] = [5, 23, 1];

pub const ROUTER_INDEX: &str = "core/frontend/src/router/index.ts";
pub const MENUS_TS: &str = "core/frontend/src/menus.ts";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub enum FrontendRouteKind {
    StaticComponent,
    AsyncComponent,
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ExtractedFrontendRoute {
    pub line: usize,
    pub block_end: usize,
    pub path: String,
    pub name: String,
    pub component: String,
    pub kind: FrontendRouteKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ExtractedFrontendMenu {
    pub line: usize,
    pub block_end: usize,
    pub title: String,
    pub route: String,
    pub advanced_only: Option<bool>,
}

#[derive(Debug)]
pub enum ExtractFrontendRouterError {
    Io(String),
    Parse(String),
}

pub fn extract_frontend_router_from_repo(
    repo_root: &Path,
) -> Result<(Vec<ExtractedFrontendRoute>, Vec<ExtractedFrontendMenu>), ExtractFrontendRouterError> {
    let router_path = repo_root.join(ROUTER_INDEX);
    let menus_path = repo_root.join(MENUS_TS);
    let router_contents = fs::read_to_string(&router_path).map_err(|err| {
        ExtractFrontendRouterError::Io(format!("read {}: {err}", router_path.display()))
    })?;
    let menus_contents = fs::read_to_string(&menus_path).map_err(|err| {
        ExtractFrontendRouterError::Io(format!("read {}: {err}", menus_path.display()))
    })?;
    Ok((
        extract_router_routes(&router_contents)?,
        extract_menu_entries(&menus_contents)?,
    ))
}

pub fn extract_router_routes(
    contents: &str,
) -> Result<Vec<ExtractedFrontendRoute>, ExtractFrontendRouterError> {
    let mut routes = Vec::new();
    let lines: Vec<&str> = contents.lines().collect();
    let (array_start, array_end) = routes_array_bounds(&lines);
    let mut idx = array_start;
    while idx <= array_end {
        if !lines[idx].trim().starts_with('{') {
            idx += 1;
            continue;
        }
        let close = object_block_end(&lines, idx, array_end);
        let block = &lines[idx..=close];
        if let Some(path) = block.iter().find_map(|line| parse_ts_field(line, "path")) {
            let line_number = block
                .iter()
                .enumerate()
                .find_map(|(offset, line)| parse_ts_field(line, "path").map(|_| idx + offset + 1))
                .unwrap_or(idx + 1);
            let name = block
                .iter()
                .find_map(|line| parse_ts_field(line, "name"))
                .ok_or_else(|| {
                    ExtractFrontendRouterError::Parse(format!(
                        "{ROUTER_INDEX}:{line_number}: route block missing name"
                    ))
                })?;
            let component_line = block
                .iter()
                .find(|line| line.trim().starts_with("component:"))
                .ok_or_else(|| {
                    ExtractFrontendRouterError::Parse(format!(
                        "{ROUTER_INDEX}:{line_number}: route block missing component"
                    ))
                })?;
            let component = parse_component_value(component_line).ok_or_else(|| {
                ExtractFrontendRouterError::Parse(format!(
                    "{ROUTER_INDEX}:{line_number}: route block missing component"
                ))
            })?;
            let kind = classify_route_kind(&path, component_line);
            routes.push(ExtractedFrontendRoute {
                line: line_number,
                block_end: close + 1,
                path,
                name,
                component,
                kind,
            });
        }
        idx = close + 1;
    }
    Ok(routes)
}

pub fn extract_menu_entries(
    contents: &str,
) -> Result<Vec<ExtractedFrontendMenu>, ExtractFrontendRouterError> {
    let mut menus = Vec::new();
    let lines: Vec<&str> = contents.lines().collect();
    let (array_start, array_end) = menus_array_bounds(&lines);
    let mut idx = array_start;
    while idx <= array_end {
        if lines[idx].trim() != "{" {
            idx += 1;
            continue;
        }
        let block_end = object_block_end(&lines, idx, array_end);
        let block = &lines[idx..=block_end];
        if let Some(route) = block.iter().find_map(|line| parse_ts_field(line, "route")) {
            let title = block
                .iter()
                .find_map(|line| parse_ts_field(line, "title"))
                .ok_or_else(|| {
                    ExtractFrontendRouterError::Parse(format!(
                        "{MENUS_TS}:{}: menu block missing title",
                        idx + 1
                    ))
                })?;
            let title_line = block
                .iter()
                .enumerate()
                .find_map(|(offset, line)| parse_ts_field(line, "title").map(|_| idx + offset + 1))
                .unwrap_or(idx + 1);
            let advanced_only = block.iter().find_map(|line| {
                parse_bool_field(line, "advanced").or_else(|| parse_bool_field(line, "show"))
            });
            menus.push(ExtractedFrontendMenu {
                line: title_line,
                block_end: block_end + 1,
                title,
                route,
                advanced_only,
            });
        }
        idx = block_end + 1;
    }
    Ok(menus)
}

fn object_block_end(lines: &[&str], start: usize, array_end: usize) -> usize {
    let mut depth = 0i32;
    for (idx, line) in lines.iter().enumerate().take(array_end + 1).skip(start) {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    return idx;
                }
            }
        }
    }
    array_end
}

fn menus_array_bounds(lines: &[&str]) -> (usize, usize) {
    array_bracket_bounds(lines, "const menus")
}

fn routes_array_bounds(lines: &[&str]) -> (usize, usize) {
    array_bracket_bounds(lines, "const routes")
}

fn array_bracket_bounds(lines: &[&str], marker: &str) -> (usize, usize) {
    let marker_line = lines
        .iter()
        .position(|line| line.contains(marker))
        .unwrap_or(0);
    let open = lines
        .iter()
        .enumerate()
        .skip(marker_line)
        .find(|(_, line)| line.contains('['))
        .map(|(idx, _)| idx)
        .unwrap_or(marker_line);
    let mut depth = 0i32;
    for (idx, line) in lines.iter().enumerate().skip(open) {
        for ch in line.chars() {
            if ch == '[' {
                depth += 1;
            } else if ch == ']' {
                depth -= 1;
                if depth == 0 {
                    return (open, idx);
                }
            }
        }
    }
    (open, lines.len().saturating_sub(1))
}

fn classify_route_kind(path: &str, component_line: &str) -> FrontendRouteKind {
    if path == "*" {
        FrontendRouteKind::Wildcard
    } else if component_line.contains("defineAsyncComponent") {
        FrontendRouteKind::AsyncComponent
    } else {
        FrontendRouteKind::StaticComponent
    }
}

fn parse_component_value(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("component:") {
        return None;
    }
    let rest = trimmed
        .strip_prefix("component:")?
        .trim()
        .trim_end_matches(',');
    if let Some(path) = extract_import_path(rest) {
        return Some(path);
    }
    match rest {
        "Main" => Some("core/frontend/src/views/MainView.vue".to_string()),
        "ExtensionView" => Some("core/frontend/src/views/ExtensionView.vue".to_string()),
        "PageNotFound" => Some("core/frontend/src/views/PageNotFound.vue".to_string()),
        other => Some(other.to_string()),
    }
}

fn extract_import_path(input: &str) -> Option<String> {
    let needle = "import('../";
    let start = input.find(needle)? + needle.len();
    let rest = &input[start..];
    let end = rest.find('\'')?;
    Some(format!("core/frontend/src/{}", &rest[..end]))
}

fn parse_ts_field(line: &str, field: &str) -> Option<String> {
    let trimmed = line.trim();
    let prefix = format!("{field}:");
    if !trimmed.starts_with(&prefix) {
        return None;
    }
    let rest = trimmed[prefix.len()..].trim().trim_end_matches(',');
    first_quoted_string(rest)
}

fn parse_bool_field(line: &str, field: &str) -> Option<bool> {
    let trimmed = line.trim();
    let prefix = format!("{field}:");
    if !trimmed.starts_with(&prefix) {
        return None;
    }
    let rest = trimmed[prefix.len()..].trim().trim_end_matches(',');
    match rest {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn first_quoted_string(input: &str) -> Option<String> {
    let quote = input.find(['"', '\''])?;
    let rest = &input[quote + 1..];
    let quote_ch = input.as_bytes()[quote] as char;
    let end = rest.find(quote_ch)?;
    Some(rest[..end].to_string())
}

pub fn frontend_route_kind_histogram(routes: &[ExtractedFrontendRoute]) -> [usize; 3] {
    let mut counts = [0usize; 3];
    for route in routes {
        let idx = match route.kind {
            FrontendRouteKind::StaticComponent => 0,
            FrontendRouteKind::AsyncComponent => 1,
            FrontendRouteKind::Wildcard => 2,
        };
        counts[idx] += 1;
    }
    counts
}

pub fn find_router_route_at_line(
    routes: &[ExtractedFrontendRoute],
    line: u32,
) -> Option<&ExtractedFrontendRoute> {
    routes
        .iter()
        .filter(|route| route.line <= line as usize && line as usize <= route.block_end)
        .max_by_key(|route| route.line)
}

pub fn find_menu_entry_at_line(
    menus: &[ExtractedFrontendMenu],
    line: u32,
) -> Option<&ExtractedFrontendMenu> {
    menus
        .iter()
        .filter(|menu| menu.line <= line as usize && line as usize <= menu.block_end)
        .max_by_key(|menu| menu.line)
}

pub fn check_frontend_router_against_observed(
    routes: &[ExtractedFrontendRoute],
    menus: &[ExtractedFrontendMenu],
    pages: &[Page],
) -> DriftReport {
    let mut findings = Vec::new();
    for page in pages {
        check_page_route_fields(page, routes, &mut findings);
        check_page_menu_fields(page, menus, &mut findings);
    }
    DriftReport { findings }
}

fn check_page_route_fields(
    page: &Page,
    routes: &[ExtractedFrontendRoute],
    findings: &mut Vec<DriftFinding>,
) {
    if let Observed::Known { value, evidence } = &page.route {
        if evidence_from_router(evidence) {
            compare_router_path(page, "route", value, evidence, routes, findings);
        }
    }
    if let Observed::Known { value, evidence } = &page.name {
        if evidence_from_router(evidence) {
            compare_router_name(page, value, evidence, routes, findings);
        }
    }
    if let Observed::Known { value, evidence } = &page.component {
        if evidence_from_router(evidence) {
            compare_router_component(page, value, evidence, routes, findings);
        }
    }
}

fn check_page_menu_fields(
    page: &Page,
    menus: &[ExtractedFrontendMenu],
    findings: &mut Vec<DriftFinding>,
) {
    if let Observed::Known { value, evidence } = &page.menu_title {
        if evidence_from_menus(evidence) {
            compare_menu_title(page, value, evidence, menus, findings);
        }
    }
    if let Observed::Known { value, evidence } = &page.advanced_only {
        if evidence_from_menus(evidence) {
            compare_menu_advanced(page, *value, evidence, menus, findings);
        }
    }
}

fn compare_router_path(
    page: &Page,
    field: &str,
    observed: &str,
    evidence: &Evidence,
    routes: &[ExtractedFrontendRoute],
    findings: &mut Vec<DriftFinding>,
) {
    let Some(extracted) = find_router_route_at_line(routes, evidence.line) else {
        findings.push(DriftFinding {
            field: format!("{}.{}", page.id.as_str(), field),
            message: format!(
                "observed route {observed:?} ({}:{}), no extracted router block at cited line",
                evidence.file, evidence.line
            ),
        });
        return;
    };
    if observed != extracted.path {
        findings.push(DriftFinding {
            field: format!("{}.{}", page.id.as_str(), field),
            message: format!(
                "observed route {observed:?} ({}:{}), extracted route {:?}",
                evidence.file, evidence.line, extracted.path
            ),
        });
    }
}

fn compare_router_name(
    page: &Page,
    observed: &str,
    evidence: &Evidence,
    routes: &[ExtractedFrontendRoute],
    findings: &mut Vec<DriftFinding>,
) {
    let Some(extracted) = find_router_route_at_line(routes, evidence.line) else {
        findings.push(DriftFinding {
            field: format!("{}.name", page.id.as_str()),
            message: format!(
                "observed name {observed:?} ({}:{}), no extracted router block at cited line",
                evidence.file, evidence.line
            ),
        });
        return;
    };
    if observed != extracted.name {
        findings.push(DriftFinding {
            field: format!("{}.name", page.id.as_str()),
            message: format!(
                "observed name {observed:?} ({}:{}), extracted name {:?}",
                evidence.file, evidence.line, extracted.name
            ),
        });
    }
}

fn compare_router_component(
    page: &Page,
    observed: &str,
    evidence: &Evidence,
    routes: &[ExtractedFrontendRoute],
    findings: &mut Vec<DriftFinding>,
) {
    let Some(extracted) = find_router_route_at_line(routes, evidence.line) else {
        findings.push(DriftFinding {
            field: format!("{}.component", page.id.as_str()),
            message: format!(
                "observed component {observed:?} ({}:{}), no extracted router block at cited line",
                evidence.file, evidence.line
            ),
        });
        return;
    };
    if observed != extracted.component {
        findings.push(DriftFinding {
            field: format!("{}.component", page.id.as_str()),
            message: format!(
                "observed component {observed:?} ({}:{}), extracted component {:?}",
                evidence.file, evidence.line, extracted.component
            ),
        });
    }
}

fn compare_menu_title(
    page: &Page,
    observed: &str,
    evidence: &Evidence,
    menus: &[ExtractedFrontendMenu],
    findings: &mut Vec<DriftFinding>,
) {
    let Some(extracted) = find_menu_entry_at_line(menus, evidence.line) else {
        findings.push(DriftFinding {
            field: format!("{}.menu_title", page.id.as_str()),
            message: format!(
                "observed menu_title {observed:?} ({}:{}), no extracted menu block at cited line",
                evidence.file, evidence.line
            ),
        });
        return;
    };
    if observed != extracted.title {
        findings.push(DriftFinding {
            field: format!("{}.menu_title", page.id.as_str()),
            message: format!(
                "observed menu_title {observed:?} ({}:{}), extracted title {:?}",
                evidence.file, evidence.line, extracted.title
            ),
        });
    }
}

fn compare_menu_advanced(
    page: &Page,
    observed: bool,
    evidence: &Evidence,
    menus: &[ExtractedFrontendMenu],
    findings: &mut Vec<DriftFinding>,
) {
    let Some(extracted) = find_menu_entry_at_line(menus, evidence.line) else {
        findings.push(DriftFinding {
            field: format!("{}.advanced_only", page.id.as_str()),
            message: format!(
                "observed advanced_only {observed} ({}:{}), no extracted menu block at cited line",
                evidence.file, evidence.line
            ),
        });
        return;
    };
    if !menu_matches_page_title(page, extracted) {
        let expected = match &page.menu_title {
            Observed::Known { value, .. } => *value,
            Observed::Unknown { .. } => "<unknown menu_title>",
        };
        findings.push(DriftFinding {
            field: format!("{}.advanced_only", page.id.as_str()),
            message: format!(
                "observed advanced_only {observed} ({}:{}), cited line is in menu {:?} not {expected:?}",
                evidence.file, evidence.line, extracted.title
            ),
        });
        return;
    }
    let Some(extracted_advanced) = extracted.advanced_only else {
        findings.push(DriftFinding {
            field: format!("{}.advanced_only", page.id.as_str()),
            message: format!(
                "observed advanced_only {observed} ({}:{}), extracted menu has no advanced/show field",
                evidence.file,
                evidence.line
            ),
        });
        return;
    };
    if observed != extracted_advanced {
        findings.push(DriftFinding {
            field: format!("{}.advanced_only", page.id.as_str()),
            message: format!(
                "observed advanced_only {observed} ({}:{}), extracted {:?}",
                evidence.file, evidence.line, extracted.advanced_only
            ),
        });
    }
}

pub(crate) fn menu_matches_page_title(page: &Page, menu: &ExtractedFrontendMenu) -> bool {
    match &page.menu_title {
        Observed::Known { value, .. } => menu.title == *value,
        Observed::Unknown { .. } => false,
    }
}

pub(crate) fn evidence_from_router(evidence: &Evidence) -> bool {
    evidence.file == ROUTER_INDEX
}

pub(crate) fn evidence_from_menus(evidence: &Evidence) -> bool {
    evidence.file == MENUS_TS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Catalog;

    #[test]
    fn extract_pins_frontend_route_counts() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let (routes, menus) =
            extract_frontend_router_from_repo(repo_root).expect("extract frontend router");
        assert_eq!(routes.len(), FRONTEND_ROUTER_ROUTE_COUNT);
        assert_eq!(menus.len(), FRONTEND_MENU_ENTRY_COUNT);
        assert_eq!(
            frontend_route_kind_histogram(&routes),
            FRONTEND_ROUTE_KIND_HISTOGRAM
        );
    }

    #[test]
    fn extract_pings_route_maps_by_value() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let (routes, _) =
            extract_frontend_router_from_repo(repo_root).expect("extract frontend router");
        let pings = routes
            .iter()
            .find(|route| route.path == "/vehicle/pings")
            .expect("/vehicle/pings route");
        assert_eq!(pings.line, 27);
        assert_eq!(pings.name, "Pings");
        assert_eq!(pings.component, "core/frontend/src/views/Pings.vue");
    }

    #[test]
    fn bootstrap_frontend_router_cross_check_runs() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let (routes, menus) =
            extract_frontend_router_from_repo(repo_root).expect("extract frontend router");
        assert!(!routes.is_empty());
        let catalog = Catalog::bootstrap();
        let report = check_frontend_router_against_observed(&routes, &menus, catalog.pages());
        assert!(
            !report.has_drift(),
            "expected no frontend-router drift, got {:?}",
            report.findings
        );
    }

    #[test]
    fn extract_requires_router_path_field() {
        let contents = "const routes = [\n  { bogus: '/vehicle/pings' },\n]\n";
        let routes = extract_router_routes(contents).expect("parse synthetic router");
        assert_eq!(routes.len(), 0);
    }

    #[test]
    fn menu_advanced_is_read_from_same_object_not_previous() {
        let contents = concat!(
            "const menus = [\n",
            "  {\n",
            "    title: 'Previous',\n",
            "    route: '/prev',\n",
            "    advanced: true,\n",
            "    text: 'prev text here',\n",
            "  },\n",
            "  {\n",
            "    title: 'Current',\n",
            "    route: '/cur',\n",
            "    advanced: false,\n",
            "    text: 'cur text here',\n",
            "  },\n",
            "]\n",
        );
        let menus = extract_menu_entries(contents).expect("parse synthetic menus");
        assert_eq!(menus.len(), 2);
        assert_eq!(menus[0].advanced_only, Some(true));
        assert_eq!(menus[1].title, "Current");
        assert_eq!(menus[1].advanced_only, Some(false));
        assert!(menus[1].line <= 13 && 13 <= menus[1].block_end);
    }

    #[test]
    fn perturbed_router_path_reports_drift() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let (mut routes, menus) =
            extract_frontend_router_from_repo(repo_root).expect("extract frontend router");
        let pings = routes
            .iter_mut()
            .find(|route| route.path == "/vehicle/pings")
            .expect("/vehicle/pings route");
        pings.path = "/vehicle/ping".to_string();
        let catalog = Catalog::bootstrap();
        let report = check_frontend_router_against_observed(&routes, &menus, catalog.pages());
        assert!(
            report.has_drift(),
            "expected drift, got {:?}",
            report.findings
        );
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.field.contains("pings")));
    }

    #[test]
    fn router_blocks_do_not_overlap() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let (routes, _) =
            extract_frontend_router_from_repo(repo_root).expect("extract frontend router");
        for pair in routes.windows(2) {
            assert!(
                pair[0].block_end < pair[1].line,
                "route {} block_end {} overlaps next path {}",
                pair[0].path,
                pair[0].block_end,
                pair[1].line
            );
        }
    }

    #[test]
    fn missing_router_block_reports_name_and_component() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let (routes, menus) =
            extract_frontend_router_from_repo(repo_root).expect("extract frontend router");
        let page = synthetic_bridges_page(160, 160, 160, 106, 109);
        let report = check_frontend_router_against_observed(&routes, &menus, &[page]);
        assert!(
            report.findings.iter().any(|finding| {
                finding.field == "bridges.name" && finding.message.contains("no extracted router")
            }),
            "{:?}",
            report.findings
        );
        assert!(
            report.findings.iter().any(|finding| {
                finding.field == "bridges.component"
                    && finding.message.contains("no extracted router")
            }),
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn matching_advanced_bool_in_wrong_menu_block_is_drift() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let (routes, menus) =
            extract_frontend_router_from_repo(repo_root).expect("extract frontend router");
        let page = synthetic_bridges_page(77, 78, 79, 106, 80);
        let report = check_frontend_router_against_observed(&routes, &menus, &[page]);
        assert!(
            report.findings.iter().any(|finding| {
                finding.field == "bridges.advanced_only"
                    && finding.message.contains("cited line is in menu")
            }),
            "{:?}",
            report.findings
        );
    }

    fn synthetic_bridges_page(
        route_line: u32,
        name_line: u32,
        component_line: u32,
        title_line: u32,
        advanced_line: u32,
    ) -> Page {
        Page {
            id: crate::page::PageId::Bridges,
            route: Observed::known(
                "/tools/bridges",
                Evidence {
                    file: ROUTER_INDEX,
                    line: route_line,
                    anchor: "path: '/tools/bridges',",
                },
            ),
            name: Observed::known(
                "Bridges",
                Evidence {
                    file: ROUTER_INDEX,
                    line: name_line,
                    anchor: "name: 'Bridges',",
                },
            ),
            component: Observed::known(
                "core/frontend/src/views/BridgesView.vue",
                Evidence {
                    file: ROUTER_INDEX,
                    line: component_line,
                    anchor: "component: defineAsyncComponent(() => import('../views/Bridg",
                },
            ),
            menu_title: Observed::known(
                "Serial Bridges",
                Evidence {
                    file: MENUS_TS,
                    line: title_line,
                    anchor: "title: 'Serial Bridges',",
                },
            ),
            advanced_only: Observed::known(
                true,
                Evidence {
                    file: MENUS_TS,
                    line: advanced_line,
                    anchor: "advanced: true,",
                },
            ),
            stores: crate::provenance::ObservedSet::known(&[]),
            consumes: crate::provenance::ObservedSet::known(&[]),
            frontend_features: crate::provenance::AssertedSet::established(&[]),
            client_state: crate::provenance::AssertedSet::established(&[]),
        }
    }
}
