use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::extract_fastapi::{
    extract_fastapi_from_repo, normalized_route_path, service_dir_for_id, strip_query,
    ExtractedFastApiRoute,
};
use crate::id::ServiceId;
use crate::journey::{HttpMethod, RouteRef, UseCase};
use crate::page::{ConsumeTarget, Page};
use crate::provenance::{Grounded, GroundedSet, ObservedSet};

pub const DEFAULT_API_CONTRACT_BASELINE: &str = "api-contracts/baseline.json";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ApiContractKey {
    pub service: String,
    pub method: String,
    pub path: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiRouteContract {
    pub service: String,
    pub method: String,
    pub path: String,
    pub version: Option<String>,
    pub status_code: Option<u16>,
    pub response_model: Option<String>,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiContractSnapshot {
    pub routes: Vec<ApiRouteContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiContractFieldChange {
    pub key: ApiContractKey,
    pub field: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiContractDiff {
    pub added: Vec<ApiContractKey>,
    pub removed: Vec<ApiContractKey>,
    pub changed: Vec<ApiContractFieldChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiCoverageSource {
    Journey,
    Page,
    Slo,
    NegativeProbe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiCoverageReport {
    pub total: usize,
    pub mapped: usize,
    pub unmapped: usize,
    pub by_source: BTreeMap<String, usize>,
    pub unmapped_routes: Vec<ApiContractKey>,
    /// Catalog references to a route the inventory does not have, restricted to services that do
    /// appear in the inventory. This is how a renamed or deleted endpoint gets caught: the
    /// frontend page or journey keeps citing the old path and shows up here.
    pub orphan_hits: Vec<ApiContractKey>,
}

impl ApiRouteContract {
    pub fn key(&self) -> ApiContractKey {
        ApiContractKey {
            service: self.service.clone(),
            method: self.method.clone(),
            path: self.path.clone(),
            version: self.version.clone(),
        }
    }

    pub fn from_extracted(route: &ExtractedFastApiRoute) -> Self {
        Self {
            service: route.service_dir.clone(),
            method: method_label(&route.method).to_string(),
            path: normalized_route_path(&route.path),
            version: route.version.clone(),
            status_code: route.status_code,
            response_model: route.response_model.clone(),
            file: route.file.clone(),
            line: route.line,
        }
    }
}

impl ApiContractSnapshot {
    pub fn from_extracted(routes: &[ExtractedFastApiRoute]) -> Self {
        let mut snapshot = Self {
            routes: routes
                .iter()
                .map(ApiRouteContract::from_extracted)
                .collect(),
        };
        snapshot.routes.sort_by(|left, right| {
            left.key()
                .cmp(&right.key())
                .then(left.line.cmp(&right.line))
        });
        snapshot
    }

    pub fn duplicate_keys(&self) -> Vec<ApiContractKey> {
        let mut seen = BTreeSet::new();
        let mut duplicates = Vec::new();
        for route in &self.routes {
            if !seen.insert(route.key()) {
                duplicates.push(route.key());
            }
        }
        duplicates
    }

    pub fn from_repo(repo_root: &Path) -> Result<Self, String> {
        let routes = extract_fastapi_from_repo(repo_root).map_err(|error| format!("{error:?}"))?;
        Ok(Self::from_extracted(&routes))
    }
}

impl ApiContractDiff {
    pub fn is_breaking(&self) -> bool {
        !self.removed.is_empty() || !self.changed.is_empty()
    }

    pub fn is_clean(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.changed.is_empty()
    }
}

pub fn diff_snapshots(
    baseline: &ApiContractSnapshot,
    current: &ApiContractSnapshot,
) -> ApiContractDiff {
    let baseline_map = contract_map(baseline);
    let current_map = contract_map(current);
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();

    for (key, current_route) in &current_map {
        match baseline_map.get(key) {
            None => added.push(key.clone()),
            Some(baseline_route) => {
                push_field_change(
                    &mut changed,
                    key,
                    "status_code",
                    format_opt_u16(baseline_route.status_code),
                    format_opt_u16(current_route.status_code),
                );
                push_field_change(
                    &mut changed,
                    key,
                    "response_model",
                    format_opt_str(baseline_route.response_model.as_deref()),
                    format_opt_str(current_route.response_model.as_deref()),
                );
            }
        }
    }
    for key in baseline_map.keys() {
        if !current_map.contains_key(key) {
            removed.push(key.clone());
        }
    }
    added.sort();
    removed.sort();
    changed.sort_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.field.cmp(&right.field))
    });
    ApiContractDiff {
        added,
        removed,
        changed,
    }
}

pub fn api_coverage_report(
    catalog: &Catalog,
    routes: &[ExtractedFastApiRoute],
) -> ApiCoverageReport {
    let hits = coverage_hits(catalog);
    let keys: Vec<(ApiContractKey, bool)> = routes
        .iter()
        .map(|route| {
            (
                ApiRouteContract::from_extracted(route).key(),
                route.path.contains(":path}"),
            )
        })
        .collect();
    let mut by_source: BTreeMap<String, usize> = BTreeMap::new();
    let mut unmapped_routes = Vec::new();
    let mut mapped = 0usize;
    for (key, greedy_tail) in &keys {
        let mut sources = BTreeSet::new();
        for (hit, hit_sources) in &hits {
            if route_matches_hit(key, *greedy_tail, hit) {
                sources.extend(hit_sources.iter().copied());
            }
        }
        if sources.is_empty() {
            unmapped_routes.push(key.clone());
            continue;
        }
        mapped += 1;
        for source in sources {
            *by_source
                .entry(coverage_source_label(source).to_string())
                .or_default() += 1;
        }
    }
    let inventory_services: BTreeSet<&str> = routes
        .iter()
        .map(|route| route.service_dir.as_str())
        .collect();
    let mut orphan_hits: Vec<ApiContractKey> = hits
        .keys()
        .filter(|hit| inventory_services.contains(hit.service.as_str()))
        .filter(|hit| {
            !keys
                .iter()
                .any(|(key, greedy_tail)| route_matches_hit(key, *greedy_tail, hit))
        })
        .cloned()
        .collect();
    unmapped_routes.sort();
    orphan_hits.sort();
    ApiCoverageReport {
        total: routes.len(),
        mapped,
        unmapped: unmapped_routes.len(),
        by_source,
        unmapped_routes,
        orphan_hits,
    }
}

/// `None` when extraction fails. Callers substitute `usize::MAX` so the ratchet trips instead of
/// silently recording a zero, and snapshotting that sentinel is refused.
pub fn api_coverage_counts(repo_root: &Path, catalog: &Catalog) -> Option<ApiCoverageReport> {
    let routes = extract_fastapi_from_repo(repo_root).ok()?;
    Some(api_coverage_report(catalog, &routes))
}

pub fn load_api_contract_baseline(path: &Path) -> Result<ApiContractSnapshot, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&content).map_err(|error| format!("parse {}: {error}", path.display()))
}

pub fn write_api_contract_baseline(
    path: &Path,
    snapshot: &ApiContractSnapshot,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(snapshot)
        .map_err(|error| format!("serialize api contracts: {error}"))?;
    fs::write(path, format!("{content}\n"))
        .map_err(|error| format!("write {}: {error}", path.display()))
}

fn contract_map(snapshot: &ApiContractSnapshot) -> BTreeMap<ApiContractKey, &ApiRouteContract> {
    let mut map = BTreeMap::new();
    for route in &snapshot.routes {
        map.insert(route.key(), route);
    }
    map
}

fn push_field_change(
    changed: &mut Vec<ApiContractFieldChange>,
    key: &ApiContractKey,
    field: &str,
    before: String,
    after: String,
) {
    if before != after {
        changed.push(ApiContractFieldChange {
            key: key.clone(),
            field: field.to_string(),
            before,
            after,
        });
    }
}

fn coverage_hits(catalog: &Catalog) -> BTreeMap<ApiContractKey, BTreeSet<ApiCoverageSource>> {
    let mut hits: BTreeMap<ApiContractKey, BTreeSet<ApiCoverageSource>> = BTreeMap::new();
    for journey in catalog.journeys() {
        insert_journey_hits(catalog, journey, &mut hits);
    }
    for page in catalog.pages() {
        insert_page_hits(catalog, page, &mut hits);
    }
    for service in catalog.services() {
        insert_slo_hits(catalog, &service.runtime.slo_baselines, &mut hits);
    }
    for probe in crate::negative_probes::NEGATIVE_PROBES {
        insert_probe_hit(catalog, probe, &mut hits);
    }
    hits
}

fn insert_journey_hits(
    catalog: &Catalog,
    journey: &UseCase,
    hits: &mut BTreeMap<ApiContractKey, BTreeSet<ApiCoverageSource>>,
) {
    let GroundedSet::Known { items } = &journey.steps else {
        return;
    };
    for step in items.iter() {
        let Some(Grounded::Known { value: route, .. }) = &step.value.route else {
            continue;
        };
        insert_route_hit(catalog, route, ApiCoverageSource::Journey, hits);
    }
}

fn insert_page_hits(
    catalog: &Catalog,
    page: &Page,
    hits: &mut BTreeMap<ApiContractKey, BTreeSet<ApiCoverageSource>>,
) {
    let ObservedSet::Known { items } = &page.consumes else {
        return;
    };
    for item in items.iter() {
        let ConsumeTarget::Service(service) = item.value.service else {
            continue;
        };
        let Some((method, nginx_path)) = parse_consume_endpoint(item.value.endpoint) else {
            continue;
        };
        let Some((version, path)) = service_path_from_nginx(catalog, service, nginx_path) else {
            continue;
        };
        insert_lookup(
            ApiContractKey {
                service: service_dir_for_id(service).to_string(),
                method: method_label(&method).to_string(),
                path,
                version,
            },
            ApiCoverageSource::Page,
            hits,
        );
    }
}

fn insert_slo_hits(
    catalog: &Catalog,
    baselines: &GroundedSet<crate::runtime::SloBaseline>,
    hits: &mut BTreeMap<ApiContractKey, BTreeSet<ApiCoverageSource>>,
) {
    let GroundedSet::Known { items } = baselines else {
        return;
    };
    for item in items.iter() {
        insert_route_hit(catalog, &item.value.route, ApiCoverageSource::Slo, hits);
    }
}

fn insert_probe_hit(
    catalog: &Catalog,
    probe: &crate::negative_probes::NegativeProbe,
    hits: &mut BTreeMap<ApiContractKey, BTreeSet<ApiCoverageSource>>,
) {
    let Some(journey) = catalog.journey_by_id(&probe.journey_id) else {
        return;
    };
    let Some(service) = primary_service(journey) else {
        return;
    };
    let Some((version, path)) = service_path_from_nginx(catalog, service, probe.path) else {
        return;
    };
    insert_lookup(
        ApiContractKey {
            service: service_dir_for_id(service).to_string(),
            method: method_label(&probe.method).to_string(),
            path,
            version,
        },
        ApiCoverageSource::NegativeProbe,
        hits,
    );
}

fn insert_route_hit(
    catalog: &Catalog,
    route: &RouteRef,
    source: ApiCoverageSource,
    hits: &mut BTreeMap<ApiContractKey, BTreeSet<ApiCoverageSource>>,
) {
    let (version, path) = if let Some(resolved) = crate::runner::resolve_http_path(catalog, route) {
        match service_path_from_nginx(catalog, route.service, &resolved) {
            Some((version, path)) => (version.or_else(|| route.version.map(str::to_string)), path),
            None => (
                route.version.map(str::to_string),
                normalized_route_path(route.path),
            ),
        }
    } else {
        (
            route.version.map(str::to_string),
            normalized_route_path(route.path),
        )
    };
    insert_lookup(
        ApiContractKey {
            service: service_dir_for_id(route.service).to_string(),
            method: method_label(&route.method).to_string(),
            path,
            version,
        },
        source,
        hits,
    );
}

fn insert_lookup(
    lookup: ApiContractKey,
    source: ApiCoverageSource,
    hits: &mut BTreeMap<ApiContractKey, BTreeSet<ApiCoverageSource>>,
) {
    hits.entry(lookup).or_default().insert(source);
}

/// Whether a catalog reference exercises an inventory route. Used in both directions: to credit a
/// route with coverage, and to flag a reference that matches nothing as an orphan.
///
/// A versionless hit credits any version of the route, because most `RouteRef`s and page
/// `consumes` entries record the nginx path without the `/v1.0` segment.
/// ponytail: versionless `GET /` therefore credits every versioned `GET /`. Ceiling is
/// over-crediting; upgrade path is requiring a version on every RouteRef / consume / SLO.
fn route_matches_hit(key: &ApiContractKey, greedy_tail: bool, hit: &ApiContractKey) -> bool {
    if key.service != hit.service || key.method != hit.method {
        return false;
    }
    if !hit_version_reaches_route(hit.version.as_deref(), key.version.as_deref()) {
        return false;
    }
    key.path == hit.path
        || (key.path.contains('{') && path_matches_template(&key.path, &hit.path, greedy_tail))
}

/// `VersionedFastAPI` mounts a route under its declared version prefix *and every later one*, plus
/// `/latest`, so a reference to `/v2.0/extension/install` legitimately exercises a route declared at
/// `v1.0`. Confirmed on a live device: for both multi-version services the `v1.0` route set was an
/// exact subset of `v2.0`, and `latest` was byte-identical to the highest version.
///
/// Unversioned routes live on the root app, outside every version prefix, so a versioned hit never
/// reaches them.
fn hit_version_reaches_route(hit: Option<&str>, route: Option<&str>) -> bool {
    let Some(hit) = hit else {
        return true;
    };
    if hit == "latest" {
        return route.is_some();
    }
    let Some(route) = route else {
        return false;
    };
    if hit == route {
        return true;
    }
    match (parse_api_version(hit), parse_api_version(route)) {
        (Some(hit), Some(route)) => hit >= route,
        _ => false,
    }
}

/// `v1.0` -> `(1, 0)`. Parsed rather than string-compared so `v10.0` outranks `v2.0`.
fn parse_api_version(version: &str) -> Option<(u32, u32)> {
    let (major, minor) = version.strip_prefix('v')?.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Matches a concrete path (`/dhcp/details/eth0`) against a route template
/// (`/dhcp/details/{interface_name}`). `greedy_tail` is set for FastAPI's `:path` converter, whose
/// placeholder is allowed to swallow the remaining segments.
fn path_matches_template(template: &str, concrete: &str, greedy_tail: bool) -> bool {
    let template_segments: Vec<&str> = template.trim_matches('/').split('/').collect();
    let concrete_segments: Vec<&str> = concrete.trim_matches('/').split('/').collect();
    let greedy = greedy_tail
        && template_segments
            .last()
            .is_some_and(|segment| is_placeholder(segment));
    if greedy {
        if concrete_segments.len() < template_segments.len() {
            return false;
        }
    } else if concrete_segments.len() != template_segments.len() {
        return false;
    }
    for (index, template_segment) in template_segments.iter().enumerate() {
        if is_placeholder(template_segment) {
            if concrete_segments[index].is_empty() {
                return false;
            }
            if greedy && index + 1 == template_segments.len() {
                return true;
            }
            continue;
        }
        if *template_segment != concrete_segments[index] {
            return false;
        }
    }
    true
}

fn is_placeholder(segment: &str) -> bool {
    segment.starts_with('{') && segment.ends_with('}')
}

fn parse_consume_endpoint(endpoint: &str) -> Option<(HttpMethod, &str)> {
    let (method, path) = endpoint.split_once(' ')?;
    let method = match method {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        "PATCH" => HttpMethod::Patch,
        _ => return None,
    };
    Some((method, path))
}

fn service_path_from_nginx(
    catalog: &Catalog,
    service: ServiceId,
    nginx_path: &str,
) -> Option<(Option<String>, String)> {
    let path = strip_query(nginx_path);
    let prefixes = nginx_prefixes(catalog, service)?;
    for prefix in prefixes {
        if let Some(rest) = strip_nginx_prefix(path, prefix) {
            return Some(split_versioned_path(rest));
        }
    }
    None
}

fn nginx_prefixes(catalog: &Catalog, service: ServiceId) -> Option<Vec<&'static str>> {
    let observed = catalog.observed_by_id(&service)?;
    match &observed.nginx_prefixes {
        ObservedSet::Known { items } => Some(items.iter().map(|item| item.value.0).collect()),
        ObservedSet::Unknown { .. } => None,
    }
}

fn strip_nginx_prefix<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let prefix = prefix.trim_end_matches('/');
    let rest = path.strip_prefix(prefix)?;
    if rest.is_empty() || rest.starts_with('/') {
        Some(rest)
    } else {
        None
    }
}

fn split_versioned_path(rest: &str) -> (Option<String>, String) {
    let rest = rest.trim_start_matches('/');
    if let Some((head, tail)) = rest.split_once('/') {
        if is_version_token(head) {
            return (Some(head.to_string()), normalized_route_path(tail));
        }
    } else if is_version_token(rest) {
        return (Some(rest.to_string()), "/".to_string());
    }
    (None, normalized_route_path(rest))
}

fn is_version_token(token: &str) -> bool {
    let Some(rest) = token.strip_prefix('v') else {
        return false;
    };
    let mut parts = rest.split('.');
    let major = parts.next().unwrap_or("");
    let minor = parts.next().unwrap_or("");
    !major.is_empty()
        && major.chars().all(|ch| ch.is_ascii_digit())
        && !minor.is_empty()
        && minor.chars().all(|ch| ch.is_ascii_digit())
        && parts.next().is_none()
}

fn primary_service(journey: &UseCase) -> Option<ServiceId> {
    match &journey.services {
        GroundedSet::Known { items } => items.first().map(|item| item.value),
        GroundedSet::Unknown { .. } => None,
    }
}

fn method_label(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
    }
}

fn coverage_source_label(source: ApiCoverageSource) -> &'static str {
    match source {
        ApiCoverageSource::Journey => "journey",
        ApiCoverageSource::Page => "page",
        ApiCoverageSource::Slo => "slo",
        ApiCoverageSource::NegativeProbe => "negative_probe",
    }
}

fn format_opt_u16(value: Option<u16>) -> String {
    value
        .map(|code| code.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn format_opt_str(value: Option<&str>) -> String {
    value.unwrap_or("none").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract_fastapi::extract_fastapi_from_repo;
    use std::path::Path;

    #[test]
    fn snapshot_keys_are_unique() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let snapshot = ApiContractSnapshot::from_repo(repo_root).expect("snapshot");
        let mut seen = BTreeSet::new();
        for route in &snapshot.routes {
            assert!(
                seen.insert(route.key()),
                "duplicate contract key {:?}",
                route.key()
            );
        }
    }

    #[test]
    fn split_router_route_is_in_snapshot() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let snapshot = ApiContractSnapshot::from_repo(repo_root).expect("snapshot");
        assert!(snapshot.routes.iter().any(|route| {
            route.service == "ardupilot_manager"
                && route.method == "POST"
                && route.path == "/endpoints"
                && route.version.as_deref() == Some("v1.0")
        }));
    }

    #[test]
    fn journey_route_is_mapped_and_endpoint_crud_is_page_mapped() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let catalog = Catalog::bootstrap();
        let routes = extract_fastapi_from_repo(repo_root).expect("extract");
        let report = api_coverage_report(&catalog, &routes);
        assert!(report.total >= 194);
        assert!(report.mapped > 0);
        assert!(report.unmapped > 0);
        let start = report.unmapped_routes.iter().find(|key| {
            key.service == "ardupilot_manager" && key.method == "POST" && key.path == "/start"
        });
        assert!(start.is_none(), "POST /start should be journey-mapped");
        let overwrite = report.unmapped_routes.iter().find(|key| {
            key.service == "bag_of_holding" && key.method == "POST" && key.path == "/overwrite"
        });
        assert!(
            overwrite.is_none(),
            "POST /overwrite is journey-mapped at /bag/v1.0/overwrite"
        );
        let endpoints = ApiContractKey {
            service: "ardupilot_manager".to_string(),
            method: "POST".to_string(),
            path: "/endpoints".to_string(),
            version: Some("v1.0".to_string()),
        };
        assert!(
            !report.unmapped_routes.contains(&endpoints),
            "POST /endpoints is consumed by the Endpoints page"
        );
    }

    #[test]
    fn templated_route_is_credited_by_a_concrete_hit() {
        let template = "/dhcp/details/{interface_name}";
        assert!(path_matches_template(template, "/dhcp/details/eth0", false));
        assert!(!path_matches_template(template, "/dhcp/details", false));
        assert!(!path_matches_template(
            template,
            "/dhcp/details/eth0/extra",
            false
        ));
        assert!(!path_matches_template(template, "/dhcp/leases/eth0", false));
        // A `:path` converter swallows the remaining segments, a plain placeholder does not.
        let greedy = "/recorder/files/{filename}";
        assert!(path_matches_template(
            greedy,
            "/recorder/files/a/b.mp4",
            true
        ));
        assert!(!path_matches_template(
            greedy,
            "/recorder/files/a/b.mp4",
            false
        ));
    }

    #[test]
    fn concrete_probe_path_credits_the_templated_route() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let catalog = Catalog::bootstrap();
        let routes = extract_fastapi_from_repo(repo_root).expect("extract");
        let report = api_coverage_report(&catalog, &routes);
        let details = report
            .unmapped_routes
            .iter()
            .find(|key| key.service == "cable_guy" && key.path == "/dhcp/details/{interface_name}");
        assert!(
            details.is_none(),
            "GET /dhcp/details/{{interface_name}} is probed as /dhcp/details/eth0"
        );
    }

    #[test]
    fn orphan_hits_are_reported_only_for_services_in_the_inventory() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let catalog = Catalog::bootstrap();
        let routes = extract_fastapi_from_repo(repo_root).expect("extract");
        let report = api_coverage_report(&catalog, &routes);
        let inventory_services: BTreeSet<&str> = routes
            .iter()
            .map(|route| route.service_dir.as_str())
            .collect();
        for orphan in &report.orphan_hits {
            assert!(
                inventory_services.contains(orphan.service.as_str()),
                "{orphan:?} belongs to a service with no extracted routes"
            );
        }
        // The frontend posts to `${KRAKEN_API_V2_URL}/extension/install`, which kraken declares on
        // its v1 router. That is not a 404: version prefixes inherit, and a live device answered
        // `POST /kraken/v2.0/extension/install` with 200.
        assert!(!report.orphan_hits.iter().any(|key| {
            key.service == "kraken"
                && key.method == "POST"
                && key.path == "/extension/install"
                && key.version.as_deref() == Some("v2.0")
        }));
    }

    #[test]
    fn later_version_prefixes_inherit_earlier_routes() {
        // A v2.0 reference reaches a v1.0 route, but not the reverse.
        assert!(hit_version_reaches_route(Some("v2.0"), Some("v1.0")));
        assert!(!hit_version_reaches_route(Some("v1.0"), Some("v2.0")));
        assert!(hit_version_reaches_route(Some("v1.0"), Some("v1.0")));
        // Parsed, not string-compared.
        assert!(hit_version_reaches_route(Some("v10.0"), Some("v2.0")));
        assert!(!hit_version_reaches_route(Some("v2.0"), Some("v10.0")));
        // `latest` aliases the highest version prefix; unversioned root-app routes sit outside it.
        assert!(hit_version_reaches_route(Some("latest"), Some("v2.0")));
        assert!(!hit_version_reaches_route(Some("latest"), None));
        assert!(!hit_version_reaches_route(Some("v1.0"), None));
        // A versionless reference still credits any version.
        assert!(hit_version_reaches_route(None, Some("v1.0")));
        assert!(hit_version_reaches_route(None, None));
    }

    #[test]
    fn adding_a_route_is_drift_but_removing_one_is_breaking() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let current = ApiContractSnapshot::from_repo(repo_root).expect("snapshot");
        let mut baseline = current.clone();
        let removed = baseline.routes.pop().expect("non-empty");
        let diff = diff_snapshots(&baseline, &current);
        assert!(diff.added.iter().any(|key| key == &removed.key()));
        assert!(!diff.is_breaking());
        assert!(!diff.is_clean());

        let mut shrunk = current.clone();
        let dropped = shrunk.routes.pop().expect("non-empty");
        let breaking = diff_snapshots(&current, &shrunk);
        assert!(breaking.removed.iter().any(|key| key == &dropped.key()));
        assert!(breaking.is_breaking());
    }

    #[test]
    fn status_code_change_is_breaking() {
        let mut baseline = ApiContractSnapshot {
            routes: vec![sample_contract(Some(200))],
        };
        let mut current = ApiContractSnapshot {
            routes: vec![sample_contract(Some(201))],
        };
        baseline.routes[0].line = 10;
        current.routes[0].line = 99;
        let diff = diff_snapshots(&baseline, &current);
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].field, "status_code");
        assert!(diff.is_breaking());
    }

    fn sample_contract(status_code: Option<u16>) -> ApiRouteContract {
        ApiRouteContract {
            service: "ardupilot_manager".to_string(),
            method: "POST".to_string(),
            path: "/start".to_string(),
            version: Some("v1.0".to_string()),
            status_code,
            response_model: None,
            file: "core/services/ardupilot_manager/api/v1/routers/index.py".to_string(),
            line: 241,
        }
    }
}
