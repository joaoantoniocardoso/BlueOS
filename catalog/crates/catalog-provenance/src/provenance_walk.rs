use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use catalog_paths::{catalog_dir, docs_root, repo_root};
use serde::Serialize;
use serde_json::Value;

const DEFAULT_API_CONTRACT_BASELINE: &str = "api-contracts/baseline.json";
const TRACKER_CSV_PATH: &str =
    "extras/BlueOS Release Testing Tracker - BlueOS 1.x.x [TEMPLATE].csv";
const DEFAULT_BASELINE_PATH: &str = "extras/qa-harness-improve/ratchet_baseline.json";
const FAILURE_MODE_LEDGER_PATH: &str = "extras/qa-1.4-full/FAILURE_MODE_LEDGER.md";

use crate::provenance_anchor::{
    verify_anchor, window_match_count, AnchorMatchStatus, LineRelocation,
};
use catalog_core::catalog::Catalog;
use catalog_core::frontend_routes::FRONTEND_API_ENDPOINTS;
use catalog_model::journey::BLAST_RADIUS_UNKNOWN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationKind {
    Observed,
    Source,
    Doc,
    Runtime,
    Asserted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Citation {
    pub kind: CitationKind,
    pub file: String,
    pub line: Option<u32>,
    pub anchor: Option<String>,
    pub capture: Option<String>,
    pub capture_key: Option<String>,
    pub environment: Option<String>,
    pub model_path: String,
    pub root: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolveStatus {
    Resolved,
    MissingFile,
    LineOutOfRange,
    MissingCaptureKey,
    Skipped,
    Misclassified,
    NoFileReference,
    Relocated,
    AmbiguousAnchor,
    AnchorMovedFar,
    AnchorLost,
    AnchorExempt,
    Unanchored,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedCitation {
    pub citation: Citation,
    pub status: ResolveStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RegistryCoverage {
    pub name: &'static str,
    pub walked: bool,
    pub citation_count: usize,
    pub note: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvenanceWalkReport {
    pub citations: Vec<ResolvedCitation>,
    pub registries: Vec<RegistryCoverage>,
    pub docs_root_present: bool,
}

pub fn collect_citations() -> (Vec<Citation>, Vec<RegistryCoverage>) {
    let mut citations = Vec::new();
    let mut registries = Vec::new();

    walk_root(
        "catalog/bootstrap",
        &serde_json::to_value(Catalog::bootstrap()).expect("serialize bootstrap catalog"),
        &mut citations,
    );
    registries.push(RegistryCoverage {
        name: "catalog/bootstrap",
        walked: true,
        citation_count: citations.len(),
        note: None,
    });

    let before = citations.len();
    walk_root(
        "frontend_routes/FRONTEND_API_ENDPOINTS",
        &serde_json::to_value(FRONTEND_API_ENDPOINTS).expect("serialize FRONTEND_API_ENDPOINTS"),
        &mut citations,
    );
    dedupe_shared_base_sites(&mut citations, before);
    registries.push(RegistryCoverage {
        name: "frontend_routes/FRONTEND_API_ENDPOINTS",
        walked: true,
        citation_count: citations.len() - before,
        note: None,
    });

    let before = citations.len();
    walk_root(
        "journey/BLAST_RADIUS_UNKNOWN",
        &serde_json::to_value(BLAST_RADIUS_UNKNOWN).expect("serialize BLAST_RADIUS_UNKNOWN"),
        &mut citations,
    );
    registries.push(RegistryCoverage {
        name: "journey/BLAST_RADIUS_UNKNOWN",
        walked: true,
        citation_count: citations.len() - before,
        note: None,
    });

    push_catalog_path(
        "coverage/TRACKER_CSV_PATH",
        TRACKER_CSV_PATH,
        &mut citations,
        &mut registries,
    );
    push_catalog_path(
        "api_contract/DEFAULT_API_CONTRACT_BASELINE",
        DEFAULT_API_CONTRACT_BASELINE,
        &mut citations,
        &mut registries,
    );
    push_catalog_path(
        "harness_ratchet/DEFAULT_BASELINE_PATH",
        DEFAULT_BASELINE_PATH,
        &mut citations,
        &mut registries,
    );
    push_catalog_path(
        "harness_ratchet/FAILURE_MODE_LEDGER_PATH",
        FAILURE_MODE_LEDGER_PATH,
        &mut citations,
        &mut registries,
    );

    for registry in excluded_registries() {
        registries.push(registry);
    }

    (citations, registries)
}

fn excluded_registries() -> Vec<RegistryCoverage> {
    let mut rows = Vec::new();
    rows.push(excluded_registry(
        "ui/ui_suite_plans",
        "UiJourneyPlan has no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "ui/wizard_skip_plan",
        "UiJourneyPlan has no Evidence or Provenance fields",
    ));
    for name in ui_const_registry_names() {
        rows.push(excluded_registry(
            name,
            "JourneyId list only; no Evidence or Provenance fields",
        ));
    }
    rows.push(excluded_registry(
        "negative_probes/NEGATIVE_PROBES",
        "NegativeProbe has no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "mutating_smoke/MUTATING_SMOKE_ENTRIES",
        "MutatingSmokeEntry has no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "capability/CAPABILITIES",
        "CapabilityDef has no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "capability/FRONTEND_CAPABILITIES",
        "FrontendCapabilityDef has no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "journey_presence/ALL_JOURNEY_PRESENCE",
        "Availability uses intro_commit tags, not file:line provenance",
    ));
    rows.push(excluded_registry(
        "domain/DOMAINS",
        "DomainDef carries rationale strings only",
    ));
    rows.push(excluded_registry(
        "domain/ALL_AGGREGATES",
        "Aggregate enum list only; no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "coverage/TRACKER_MAPPINGS",
        "TrackerMapping has no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "tools/feature_trace_report/GOLDEN_JOURNEY_IDS",
        "Journey id strings only; no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "tools/feature_provenance_export/DEFAULT_REFERENCE_TAGS",
        "Git tag names only; no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "tools/sibling_matrix/PAIRS",
        "Journey id pairs only; no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "journey_matrix/HARD_EXCLUDED",
        "JourneyId list only; no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "journey_matrix/PAGE_LOAD_UI",
        "JourneyId list only; no Evidence or Provenance fields",
    ));
    rows.push(excluded_registry(
        "requirement/RequirementCatalog",
        "overlay is Asserted-only; Source/Doc/Runtime on overlay fails overlay_citation_kind_is_asserted",
    ));
    rows.push(excluded_registry(
        "mcm_restore/SMOKE_CATALOG_STREAM_JSON",
        "JSON request body, not a repo path",
    ));
    rows.push(excluded_registry(
        "runner/SMOKE_CORE_SWITCH_JSON",
        "JSON request body, not a repo path",
    ));
    rows.push(excluded_registry(
        "capture_env/*",
        "Docker image identity and environment strings, not repo paths",
    ));
    rows.push(excluded_registry(
        "wifi_rf/*",
        "WiFi smoke credentials and JSON bodies, not repo paths",
    ));
    rows.push(excluded_registry(
        "sitl_cal/*",
        "SITL frame names and JSON payload, not repo paths",
    ));
    rows
}

fn excluded_registry(name: &'static str, reason: &'static str) -> RegistryCoverage {
    RegistryCoverage {
        name,
        walked: false,
        citation_count: 0,
        note: Some(reason),
    }
}

fn push_catalog_path(
    name: &'static str,
    catalog_relative: &str,
    citations: &mut Vec<Citation>,
    registries: &mut Vec<RegistryCoverage>,
) {
    let before = citations.len();
    citations.push(Citation {
        kind: CitationKind::Observed,
        file: format!("catalog/{catalog_relative}"),
        line: None,
        anchor: None,
        capture: None,
        capture_key: None,
        environment: None,
        model_path: String::new(),
        root: name.to_string(),
    });
    registries.push(RegistryCoverage {
        name,
        walked: true,
        citation_count: citations.len() - before,
        note: Some("existence-only; path constant has no line number"),
    });
}

fn dedupe_shared_base_sites(citations: &mut Vec<Citation>, start: usize) {
    let tail = citations.split_off(start);
    let mut seen_bases = HashSet::new();
    for citation in tail {
        if citation.model_path.ends_with("/base")
            && !seen_bases.insert((citation.file.clone(), citation.line))
        {
            continue;
        }
        citations.push(citation);
    }
}

fn ui_const_registry_names() -> &'static [&'static str] {
    &[
        "ui/UI_CALIBRATION_JOURNEYS",
        "ui/UI_NO_HARDWARE_JOURNEYS",
        "ui/UI_CAMERA_JOURNEYS",
        "ui/UI_CONFIGURE_JOURNEYS",
        "ui/UI_EXTENSION_JOURNEYS",
        "ui/UI_BAG_JOURNEYS",
        "ui/UI_VERSION_SETTINGS_JOURNEYS",
        "ui/UI_NMEA_JOURNEYS",
        "ui/UI_BRIDGET_JOURNEYS",
        "ui/UI_CABLE_GUY_JOURNEYS",
        "ui/UI_WIFI_JOURNEYS",
        "ui/UI_TYPED_SKIP",
    ]
}

fn walk_root(root: &str, value: &Value, out: &mut Vec<Citation>) {
    walk_value(root, value, "", None, out);
}

fn walk_value(
    root: &str,
    value: &Value,
    path: &str,
    parent_key: Option<&str>,
    out: &mut Vec<Citation>,
) {
    match value {
        Value::Object(map) => {
            if let Some(citation) = classify_object(root, path, parent_key, map) {
                out.push(citation);
            }
            for (key, child) in map {
                let child_path = join_pointer(path, key);
                walk_value(root, child, &child_path, Some(key.as_str()), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let child_path = join_pointer(path, &index.to_string());
                walk_value(root, child, &child_path, parent_key, out);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn join_pointer(path: &str, segment: &str) -> String {
    if path.is_empty() {
        format!("/{segment}")
    } else {
        format!("{path}/{segment}")
    }
}

fn classify_object(
    root: &str,
    path: &str,
    parent_key: Option<&str>,
    map: &serde_json::Map<String, Value>,
) -> Option<Citation> {
    if is_runtime_object(map) {
        let capture = map.get("capture")?.as_str()?.to_string();
        let environment = map.get("environment")?.as_str()?.to_string();
        let (file, key) = split_capture_key(&capture);
        return Some(Citation {
            kind: CitationKind::Runtime,
            file,
            line: None,
            anchor: None,
            capture: Some(capture),
            capture_key: key,
            environment: Some(environment),
            model_path: path.to_string(),
            root: root.to_string(),
        });
    }

    if let Some(citation) = classify_call_site(root, path, map) {
        return Some(citation);
    }

    if let Some(citation) = classify_store_site(root, path, map) {
        return Some(citation);
    }

    if let Some(citation) = classify_asserted_provenance(root, path, parent_key, map) {
        return Some(citation);
    }

    if !is_evidence_object(map) {
        return None;
    }

    let file = map.get("file")?.as_str()?.to_string();
    let line = map
        .get("line")
        .and_then(|value| value.as_u64())
        .map(|value| value as u32);
    let anchor = map
        .get("anchor")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let kind = citation_kind_from_parent(parent_key);

    Some(Citation {
        kind,
        file,
        line,
        anchor,
        capture: None,
        capture_key: None,
        environment: None,
        model_path: path.to_string(),
        root: root.to_string(),
    })
}

fn classify_call_site(
    root: &str,
    path: &str,
    map: &serde_json::Map<String, Value>,
) -> Option<Citation> {
    let file = map.get("call_file")?.as_str()?.to_string();
    let line = map
        .get("call_line")
        .and_then(|value| value.as_u64())
        .map(|value| value as u32);
    Some(Citation {
        kind: CitationKind::Observed,
        file,
        line,
        anchor: None,
        capture: None,
        capture_key: None,
        environment: None,
        model_path: path.to_string(),
        root: root.to_string(),
    })
}

fn classify_store_site(
    root: &str,
    path: &str,
    map: &serde_json::Map<String, Value>,
) -> Option<Citation> {
    let file = map.get("store_file")?.as_str()?.to_string();
    let line = map
        .get("line")
        .and_then(|value| value.as_u64())
        .map(|value| value as u32);
    Some(Citation {
        kind: CitationKind::Observed,
        file,
        line,
        anchor: None,
        capture: None,
        capture_key: None,
        environment: None,
        model_path: path.to_string(),
        root: root.to_string(),
    })
}

fn classify_asserted_provenance(
    root: &str,
    path: &str,
    parent_key: Option<&str>,
    map: &serde_json::Map<String, Value>,
) -> Option<Citation> {
    if parent_key != Some("asserted") {
        return None;
    }
    debug_assert!(
        !map.contains_key("file"),
        "asserted provenance must not carry a file key"
    );
    map.get("rationale")?.as_str()?;
    Some(Citation {
        kind: CitationKind::Asserted,
        file: String::new(),
        line: None,
        anchor: None,
        capture: None,
        capture_key: None,
        environment: None,
        model_path: path.to_string(),
        root: root.to_string(),
    })
}

fn citation_kind_from_parent(parent_key: Option<&str>) -> CitationKind {
    match parent_key {
        Some("source") => CitationKind::Source,
        Some("doc") => CitationKind::Doc,
        _ => CitationKind::Observed,
    }
}

fn is_evidence_object(map: &serde_json::Map<String, Value>) -> bool {
    map.get("file").and_then(Value::as_str).is_some()
        && map.get("line").and_then(Value::as_u64).is_some()
        && map.get("anchor").and_then(Value::as_str).is_some()
        && !map.contains_key("call_file")
        && !map.contains_key("store_file")
}

fn is_runtime_object(map: &serde_json::Map<String, Value>) -> bool {
    map.get("capture").and_then(Value::as_str).is_some()
        && map.get("environment").and_then(Value::as_str).is_some()
}

fn split_capture_key(capture: &str) -> (String, Option<String>) {
    match capture.split_once('#') {
        Some((file, key)) => (file.to_string(), Some(key.to_string())),
        None => (capture.to_string(), None),
    }
}

pub fn resolve_citations(citations: &[Citation]) -> ProvenanceWalkReport {
    let repo = repo_root();
    let catalog = catalog_dir();
    let docs = docs_root();
    let docs_root_present = docs.is_dir();
    let mut line_cache: HashMap<PathBuf, Option<usize>> = HashMap::new();
    let mut json_cache: HashMap<PathBuf, Option<Value>> = HashMap::new();

    let resolved = citations
        .iter()
        .map(|citation| {
            resolve_one(
                citation,
                &repo,
                &catalog,
                &docs,
                docs_root_present,
                &mut line_cache,
                &mut json_cache,
            )
        })
        .collect();

    ProvenanceWalkReport {
        citations: resolved,
        registries: Vec::new(),
        docs_root_present,
    }
}

pub fn build_report() -> ProvenanceWalkReport {
    let (citations, registries) = collect_citations();
    let mut report = resolve_citations(&citations);
    report.registries = registries;
    report
}

fn resolve_one(
    citation: &Citation,
    repo: &Path,
    catalog: &Path,
    docs: &Path,
    docs_root_present: bool,
    line_cache: &mut HashMap<PathBuf, Option<usize>>,
    json_cache: &mut HashMap<PathBuf, Option<Value>>,
) -> ResolvedCitation {
    if !matches!(citation.kind, CitationKind::Runtime) && citation.line == Some(0) {
        let mut detail = "line 0 is below minimum 1".to_string();
        if citation.kind == CitationKind::Doc && repo.join(&citation.file).exists() {
            detail.push_str("; repo path filed as Doc");
        }
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::LineOutOfRange,
            detail: Some(detail),
        };
    }

    match citation.kind {
        CitationKind::Observed | CitationKind::Source => {
            let repo_path = repo.join(&citation.file);
            if !repo_path.is_file() && docs_root_present && docs.join(&citation.file).is_file() {
                let layer = match citation.kind {
                    CitationKind::Source => "Source",
                    CitationKind::Observed => "Observed",
                    _ => unreachable!(),
                };
                return ResolvedCitation {
                    citation: citation.clone(),
                    status: ResolveStatus::Misclassified,
                    detail: Some(format!("docs path filed as {layer}")),
                };
            }
            resolve_file_line(citation, &repo_path, line_cache)
        }
        CitationKind::Doc => {
            if repo.join(&citation.file).exists() {
                return ResolvedCitation {
                    citation: citation.clone(),
                    status: ResolveStatus::Misclassified,
                    detail: Some("repo path filed as Doc".to_string()),
                };
            }
            if !docs_root_present {
                return ResolvedCitation {
                    citation: citation.clone(),
                    status: ResolveStatus::Skipped,
                    detail: Some("BlueOS-docs checkout missing".to_string()),
                };
            }
            resolve_file_line(citation, &docs.join(&citation.file), line_cache)
        }
        CitationKind::Runtime => resolve_runtime(citation, catalog, json_cache),
        CitationKind::Asserted => ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::NoFileReference,
            detail: None,
        },
    }
}

fn resolve_file_line(
    citation: &Citation,
    path: &Path,
    line_cache: &mut HashMap<PathBuf, Option<usize>>,
) -> ResolvedCitation {
    if citation.line == Some(0) {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::LineOutOfRange,
            detail: Some("line 0 is below minimum 1".to_string()),
        };
    }

    if !path.is_file() {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::MissingFile,
            detail: Some(path.display().to_string()),
        };
    }

    if citation.anchor.is_none() {
        if let Some(line) = citation.line {
            let line_count = file_line_count(path, line_cache);
            let Some(line_count) = line_count else {
                return ResolvedCitation {
                    citation: citation.clone(),
                    status: ResolveStatus::MissingFile,
                    detail: Some(format!("could not read {}", path.display())),
                };
            };
            if line as usize > line_count {
                return ResolvedCitation {
                    citation: citation.clone(),
                    status: ResolveStatus::LineOutOfRange,
                    detail: Some(format!("line {line} exceeds file length {line_count}")),
                };
            }
        }
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::AnchorExempt,
            detail: None,
        };
    }

    let Some(line) = citation.line else {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::Resolved,
            detail: None,
        };
    };

    let line_count = file_line_count(path, line_cache);
    let Some(line_count) = line_count else {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::MissingFile,
            detail: Some(format!("could not read {}", path.display())),
        };
    };

    if line as usize > line_count {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::LineOutOfRange,
            detail: Some(format!("line {line} exceeds file length {line_count}")),
        };
    }

    let anchor = citation.anchor.as_deref().unwrap_or("");
    if anchor.is_empty() {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::Unanchored,
            detail: None,
        };
    }

    match verify_anchor(path, line, anchor) {
        AnchorMatchStatus::MatchesCitedLine => ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::Resolved,
            detail: None,
        },
        AnchorMatchStatus::Relocated { new_line } => ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::Relocated,
            detail: Some(format!("anchor found at line {new_line}")),
        },
        AnchorMatchStatus::AmbiguousAnchor => ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::AmbiguousAnchor,
            detail: Some(format!(
                "anchor matched multiple lines within +/- {} of line {line}",
                crate::provenance_anchor::ANCHOR_WINDOW
            )),
        },
        AnchorMatchStatus::AnchorMovedFar { found_line } => ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::AnchorMovedFar,
            detail: Some(format!(
                "anchor found at line {found_line} outside relocation window"
            )),
        },
        AnchorMatchStatus::AnchorLost => ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::AnchorLost,
            detail: Some("anchor text absent from file".to_string()),
        },
    }
}

fn resolve_runtime(
    citation: &Citation,
    catalog: &Path,
    json_cache: &mut HashMap<PathBuf, Option<Value>>,
) -> ResolvedCitation {
    let capture = citation
        .capture
        .as_deref()
        .or(Some(citation.file.as_str()))
        .unwrap_or_default();
    let (file, key) = split_capture_key(capture);
    let path = catalog.join(&file);

    if !path.is_file() {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::MissingFile,
            detail: Some(path.display().to_string()),
        };
    }

    let Some(key) = key else {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::MissingCaptureKey,
            detail: Some("capture path missing #key suffix".to_string()),
        };
    };

    let value = load_json(&path, json_cache);
    let Some(value) = value else {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::MissingFile,
            detail: Some(format!("could not parse JSON at {}", path.display())),
        };
    };

    if !json_has_key(&value, &key) {
        return ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::MissingCaptureKey,
            detail: Some(format!("key '{key}' not found in {}", path.display())),
        };
    }

    ResolvedCitation {
        citation: citation.clone(),
        status: ResolveStatus::Resolved,
        detail: None,
    }
}

fn file_line_count(path: &Path, cache: &mut HashMap<PathBuf, Option<usize>>) -> Option<usize> {
    if let Some(cached) = cache.get(path) {
        return *cached;
    }
    let count = std::fs::read_to_string(path)
        .ok()
        .map(|content| content.lines().count());
    cache.insert(path.to_path_buf(), count);
    count
}

fn load_json(path: &Path, cache: &mut HashMap<PathBuf, Option<Value>>) -> Option<Value> {
    if let Some(cached) = cache.get(path) {
        return cached.clone();
    }
    let parsed = std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok());
    cache.insert(path.to_path_buf(), parsed.clone());
    parsed
}

fn json_has_key(value: &Value, key: &str) -> bool {
    match value {
        Value::Object(map) => map.contains_key(key),
        _ => false,
    }
}

pub fn kind_status_counts(
    report: &ProvenanceWalkReport,
) -> HashMap<(CitationKind, ResolveStatus), usize> {
    let mut counts = HashMap::new();
    for item in &report.citations {
        *counts.entry((item.citation.kind, item.status)).or_insert(0) += 1;
    }
    counts
}

pub fn gate_failed(report: &ProvenanceWalkReport) -> bool {
    !unresolved_citations(report).is_empty()
}

pub fn source_facing_unresolved(report: &ProvenanceWalkReport) -> Vec<&ResolvedCitation> {
    report
        .citations
        .iter()
        .filter(|item| {
            matches!(
                item.citation.kind,
                CitationKind::Observed | CitationKind::Source | CitationKind::Runtime
            ) && item.status != ResolveStatus::Resolved
                && item.status != ResolveStatus::AnchorExempt
                && item.status != ResolveStatus::Unanchored
        })
        .collect()
}

pub fn unresolved_citations(report: &ProvenanceWalkReport) -> Vec<&ResolvedCitation> {
    report
        .citations
        .iter()
        .filter(|item| {
            item.status != ResolveStatus::Resolved
                && item.status != ResolveStatus::AnchorExempt
                && item.status != ResolveStatus::Skipped
                && item.status != ResolveStatus::NoFileReference
                && item.status != ResolveStatus::Unanchored
        })
        .collect()
}

pub fn file_citation_count(report: &ProvenanceWalkReport) -> usize {
    report
        .citations
        .iter()
        .filter(|item| item.citation.kind != CitationKind::Asserted)
        .count()
}

pub fn relocated_citations(report: &ProvenanceWalkReport) -> Vec<LineRelocation> {
    let mut rows = Vec::new();
    for item in &report.citations {
        if item.status != ResolveStatus::Relocated {
            continue;
        }
        let Some(line) = item.citation.line else {
            continue;
        };
        let Some(anchor) = item.citation.anchor.as_deref() else {
            continue;
        };
        if anchor.is_empty() {
            continue;
        }
        let Some(site) = catalog_site_for_citation(&item.citation) else {
            continue;
        };
        let new_line = item
            .detail
            .as_deref()
            .and_then(|detail| detail.strip_prefix("anchor found at line "))
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(line);
        rows.push(LineRelocation {
            site,
            file: item.citation.file.clone(),
            old_line: line,
            new_line,
            anchor: anchor.to_string(),
        });
    }
    rows
}

pub fn mark_applied_relocations_resolved(
    report: &mut ProvenanceWalkReport,
    applied: &[LineRelocation],
) {
    for item in &mut report.citations {
        if item.status != ResolveStatus::Relocated {
            continue;
        }
        let Some(line) = item.citation.line else {
            continue;
        };
        let Some(anchor) = item.citation.anchor.as_deref() else {
            continue;
        };
        let citation_site = catalog_site_for_citation(&item.citation);
        let Some(rel) = applied.iter().find(|rel| {
            rel.file == item.citation.file
                && rel.old_line == line
                && rel.anchor == anchor
                && citation_site
                    .as_ref()
                    .is_some_and(|site| relocation_site_matches(&rel.site, site))
        }) else {
            continue;
        };
        item.citation.line = Some(rel.new_line);
        item.status = ResolveStatus::Resolved;
        item.detail = None;
    }
}

pub fn unrepaired_relocations<'a>(
    requested: &'a [LineRelocation],
    applied: &[LineRelocation],
) -> Vec<&'a LineRelocation> {
    requested
        .iter()
        .filter(|rel| {
            !applied.iter().any(|got| {
                got.site == rel.site
                    && got.file == rel.file
                    && got.old_line == rel.old_line
                    && got.anchor == rel.anchor
            })
        })
        .collect()
}

pub fn format_unrepaired_fix(rel: &LineRelocation) -> String {
    format!(
        "unrepaired Relocated {}:{} site={} -- no repair pattern matched",
        rel.file,
        rel.old_line,
        rel.site.display()
    )
}

fn relocation_site_matches(rel_site: &Path, citation_site: &Path) -> bool {
    rel_site == citation_site
}

pub fn catalog_site_for_citation(citation: &Citation) -> Option<PathBuf> {
    if citation.root != "catalog/bootstrap" {
        return None;
    }
    let segs: Vec<&str> = citation
        .model_path
        .trim_start_matches('/')
        .split('/')
        .collect();
    match segs.as_slice() {
        ["services", idx, ..] => {
            let index: usize = idx.parse().ok()?;
            let id = catalog_data::SERVICES.get(index)?.id;
            Some(PathBuf::from(format!(
                "services/{}.rs",
                service_mod_name(id)
            )))
        }
        ["pages", idx, ..] => {
            let index: usize = idx.parse().ok()?;
            let name = catalog_data::frontend::PAGES.get(index)?.id.as_str();
            Some(PathBuf::from(format!("pages/{name}.rs")))
        }
        ["journeys", idx, ..] => {
            let index: usize = idx.parse().ok()?;
            let id = catalog_data::journeys::all_journeys().get(index)?.id;
            catalog_data::journeys::source_file_for_journey(id).map(PathBuf::from)
        }
        _ => None,
    }
}

fn service_mod_name(id: catalog_kernel::id::service::ServiceId) -> &'static str {
    match id {
        catalog_kernel::id::service::ServiceId::MavlinkCameraManager => "mavlink_camera_manager",
        other => other.as_str(),
    }
}

pub fn unanchored_citation_count(report: &ProvenanceWalkReport) -> usize {
    report
        .citations
        .iter()
        .filter(|item| item.citation.anchor.as_deref() == Some(""))
        .count()
}

pub fn non_unique_in_window_citation_count(report: &ProvenanceWalkReport) -> usize {
    let repo = repo_root();
    let docs = docs_root();
    report
        .citations
        .iter()
        .filter(|item| {
            if item.status != ResolveStatus::Resolved {
                return false;
            }
            let Some(anchor) = item.citation.anchor.as_deref() else {
                return false;
            };
            if anchor.is_empty() {
                return false;
            }
            let Some(line) = item.citation.line else {
                return false;
            };
            let path = match item.citation.kind {
                CitationKind::Doc => docs.join(&item.citation.file),
                _ => repo.join(&item.citation.file),
            };
            window_match_count(&path, line, anchor) > 1
        })
        .count()
}

pub fn anchor_exempt_citation_count(report: &ProvenanceWalkReport) -> usize {
    report
        .citations
        .iter()
        .filter(|item| item.status == ResolveStatus::AnchorExempt)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog_kernel::provenance::{Evidence, Grounded, Observed, Provenance};
    use catalog_model::journey::BLAST_RADIUS_UNKNOWN;

    #[test]
    fn walk_finds_observed_and_source_provenance() {
        let observed = Observed::known(
            1_u32,
            Evidence {
                file: "core/start-blueos-core",
                line: 1,
                anchor: "#!/usr/bin/env bash",
            },
        );
        let grounded = Grounded::known("x", Provenance::source("core/start-blueos-core", 2, ""));
        let value = serde_json::json!({
            "observed": observed,
            "grounded": grounded,
        });

        let mut citations = Vec::new();
        walk_root("test", &value, &mut citations);
        assert_eq!(citations.len(), 2);
        assert!(citations
            .iter()
            .any(|c| c.kind == CitationKind::Observed && c.line == Some(1)));
        assert!(citations
            .iter()
            .any(|c| c.kind == CitationKind::Source && c.line == Some(2)));
    }

    #[test]
    fn walk_asserted_blast_radius_is_counted() {
        let value = serde_json::to_value(BLAST_RADIUS_UNKNOWN).expect("serialize blast radius");
        let mut citations = Vec::new();
        walk_root("test", &value, &mut citations);
        assert_eq!(citations.len(), 1);
        assert_eq!(citations[0].kind, CitationKind::Asserted);
    }

    #[test]
    fn walk_finds_frontend_call_and_store_sites() {
        let value = serde_json::json!({
            "base": {
                "store_file": "core/frontend/src/store/records.ts",
                "line": 11,
                "api_url": "/recorder-extractor/v1.0/recorder",
                "service": "recorder_extractor",
            },
            "relative_path": "/files",
            "call_file": "core/frontend/src/store/records.ts",
            "call_line": 47,
        });
        let mut citations = Vec::new();
        walk_root("test", &value, &mut citations);
        assert_eq!(citations.len(), 2);
        assert!(citations.iter().any(|c| {
            c.kind == CitationKind::Observed
                && c.file.ends_with("records.ts")
                && c.line == Some(47)
                && c.model_path.is_empty()
        }));
        assert!(citations.iter().any(|c| {
            c.kind == CitationKind::Observed
                && c.file.ends_with("records.ts")
                && c.line == Some(11)
                && c.model_path == "/base"
        }));
    }

    #[test]
    fn bootstrap_walk_covers_citation_surface() {
        const BOOTSTRAP_OBSERVED_FLOOR: usize = 900;
        const BOOTSTRAP_SOURCE_FLOOR: usize = 510;
        const BOOTSTRAP_DOC_FLOOR: usize = 380;
        const BOOTSTRAP_RUNTIME_FLOOR: usize = 200;
        const TOTAL_CITATION_FLOOR: usize = 2040;

        let (citations, registries) = collect_citations();
        let bootstrap: Vec<_> = citations
            .iter()
            .filter(|citation| citation.root == "catalog/bootstrap")
            .collect();

        for kind in [
            CitationKind::Observed,
            CitationKind::Source,
            CitationKind::Doc,
            CitationKind::Runtime,
        ] {
            assert!(
                bootstrap.iter().any(|citation| citation.kind == kind),
                "bootstrap walk missing {kind:?}"
            );
        }

        let mut bootstrap_kind_counts = HashMap::new();
        for citation in &bootstrap {
            *bootstrap_kind_counts.entry(citation.kind).or_insert(0) += 1;
        }
        assert!(bootstrap_kind_counts[&CitationKind::Observed] >= BOOTSTRAP_OBSERVED_FLOOR);
        assert!(bootstrap_kind_counts[&CitationKind::Source] >= BOOTSTRAP_SOURCE_FLOOR);
        assert!(bootstrap_kind_counts[&CitationKind::Doc] >= BOOTSTRAP_DOC_FLOOR);
        assert!(bootstrap_kind_counts[&CitationKind::Runtime] >= BOOTSTRAP_RUNTIME_FLOOR);
        assert!(citations.len() >= TOTAL_CITATION_FLOOR);

        let frontend: Vec<_> = citations
            .iter()
            .filter(|citation| citation.root == "frontend_routes/FRONTEND_API_ENDPOINTS")
            .collect();
        let mut frontend_sites = std::collections::HashSet::new();
        for citation in &frontend {
            assert!(
                frontend_sites.insert((citation.kind, citation.file.as_str(), citation.line)),
                "duplicate frontend site {}:{}",
                citation.file,
                citation.line.unwrap_or(0)
            );
        }
        let unique_bases: std::collections::HashSet<_> = FRONTEND_API_ENDPOINTS
            .iter()
            .map(|endpoint| (endpoint.base.store_file, endpoint.base.line))
            .collect();
        assert_eq!(frontend.len(), frontend_sites.len());
        assert_eq!(
            frontend.len(),
            FRONTEND_API_ENDPOINTS.len() + unique_bases.len()
        );

        for name in [
            "catalog/bootstrap",
            "frontend_routes/FRONTEND_API_ENDPOINTS",
            "coverage/TRACKER_CSV_PATH",
            "api_contract/DEFAULT_API_CONTRACT_BASELINE",
            "harness_ratchet/DEFAULT_BASELINE_PATH",
            "harness_ratchet/FAILURE_MODE_LEDGER_PATH",
            "domain/ALL_AGGREGATES",
            "tools/feature_trace_report/GOLDEN_JOURNEY_IDS",
            "tools/feature_provenance_export/DEFAULT_REFERENCE_TAGS",
            "tools/sibling_matrix/PAIRS",
            "journey_matrix/HARD_EXCLUDED",
            "journey_matrix/PAGE_LOAD_UI",
            "requirement/RequirementCatalog",
            "mcm_restore/SMOKE_CATALOG_STREAM_JSON",
            "runner/SMOKE_CORE_SWITCH_JSON",
            "capture_env/*",
            "wifi_rf/*",
            "sitl_cal/*",
        ] {
            assert!(
                registries.iter().any(|registry| registry.name == name),
                "missing registry {name}"
            );
        }
    }

    #[test]
    fn source_to_asserted_downgrade_trips_citation_pins() {
        const FILE_CITATION_COUNT: usize = 2191;
        const SOURCE_COUNT: usize = 611;
        const ASSERTED_COUNT: usize = 232;

        let (citations, _) = collect_citations();
        let source_count = citations
            .iter()
            .filter(|citation| citation.kind == CitationKind::Source)
            .count();
        let asserted_count = citations
            .iter()
            .filter(|citation| citation.kind == CitationKind::Asserted)
            .count();
        let file_count = citations
            .iter()
            .filter(|citation| citation.kind != CitationKind::Asserted)
            .count();

        assert_eq!(source_count, SOURCE_COUNT);
        assert_eq!(asserted_count, ASSERTED_COUNT);
        assert_eq!(file_count, FILE_CITATION_COUNT);
    }

    #[test]
    fn unanchored_citation_count_is_pinned() {
        const UNANCHORED_COUNT: usize = 1;

        let report = build_report();
        assert_eq!(unanchored_citation_count(&report), UNANCHORED_COUNT);
        assert_eq!(
            report
                .citations
                .iter()
                .filter(|item| item.status == ResolveStatus::Unanchored)
                .count(),
            UNANCHORED_COUNT
        );
    }

    #[test]
    fn non_unique_in_window_citation_count_is_pinned() {
        const NON_UNIQUE_IN_WINDOW_COUNT: usize = 178;

        let report = build_report();
        assert_eq!(
            non_unique_in_window_citation_count(&report),
            NON_UNIQUE_IN_WINDOW_COUNT
        );
    }

    #[test]
    fn anchor_exempt_citation_count_is_pinned() {
        const ANCHOR_EXEMPT_COUNT: usize = 27;

        let report = build_report();
        assert_eq!(anchor_exempt_citation_count(&report), ANCHOR_EXEMPT_COUNT);
    }

    fn test_citation(kind: CitationKind, file: &str, line: Option<u32>) -> Citation {
        Citation {
            kind,
            file: file.to_string(),
            line,
            anchor: None,
            capture: None,
            capture_key: None,
            environment: None,
            model_path: "/test".to_string(),
            root: "test".to_string(),
        }
    }

    fn test_runtime_citation(capture: &str) -> Citation {
        Citation {
            kind: CitationKind::Runtime,
            file: capture.to_string(),
            line: None,
            anchor: None,
            capture: Some(capture.to_string()),
            capture_key: None,
            environment: Some("test".to_string()),
            model_path: "/test".to_string(),
            root: "test".to_string(),
        }
    }

    fn temp_fixture(name: &str) -> PathBuf {
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("TMPDIR").map(PathBuf::from))
            .unwrap_or_else(std::env::temp_dir);
        let dir = base.join(format!(
            "blueos-catalog-provenance-walk-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp fixture");
        dir
    }

    #[test]
    fn resolve_line_zero_is_out_of_range() {
        let dir = temp_fixture("line-zero");
        let file = dir.join("sample.txt");
        std::fs::write(&file, "alpha\n").expect("write sample");
        let mut cache = HashMap::new();
        let citation = test_citation(CitationKind::Observed, "sample.txt", Some(0));
        let resolved = resolve_file_line(&citation, &file, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::LineOutOfRange);
    }

    #[test]
    fn resolve_directory_path_is_missing_file() {
        let dir = temp_fixture("directory");
        let path = dir.join("docs");
        std::fs::create_dir(&path).expect("create dir");
        let mut cache = HashMap::new();
        let citation = test_citation(CitationKind::Doc, "docs", Some(1));
        let resolved = resolve_file_line(&citation, &path, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::MissingFile);
    }

    #[test]
    fn resolve_last_line_with_and_without_trailing_newline() {
        let dir = temp_fixture("last-line");

        let with_newline = dir.join("with-newline.txt");
        std::fs::write(&with_newline, "one\ntwo\n").expect("write with newline");
        let mut cache = HashMap::new();
        let last_line = test_citation(CitationKind::Observed, "with-newline.txt", Some(2));
        assert_eq!(
            resolve_file_line(&last_line, &with_newline, &mut cache).status,
            ResolveStatus::AnchorExempt
        );
        let past_end = test_citation(CitationKind::Observed, "with-newline.txt", Some(3));
        assert_eq!(
            resolve_file_line(&past_end, &with_newline, &mut cache).status,
            ResolveStatus::LineOutOfRange
        );

        let without_newline = dir.join("without-newline.txt");
        std::fs::write(&without_newline, "one\ntwo").expect("write without newline");
        let last_line = test_citation(CitationKind::Observed, "without-newline.txt", Some(2));
        assert_eq!(
            resolve_file_line(&last_line, &without_newline, &mut cache).status,
            ResolveStatus::AnchorExempt
        );
        let past_end = test_citation(CitationKind::Observed, "without-newline.txt", Some(3));
        assert_eq!(
            resolve_file_line(&past_end, &without_newline, &mut cache).status,
            ResolveStatus::LineOutOfRange
        );
    }

    #[test]
    fn resolve_runtime_missing_capture_key() {
        let dir = temp_fixture("runtime");
        let capture_path = dir.join("observed/runtime/foo.json");
        std::fs::create_dir_all(capture_path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&capture_path, r#"{"present":true}"#).expect("write capture");
        let mut cache = HashMap::new();
        let citation = test_runtime_citation("observed/runtime/foo.json#missing");
        let resolved = resolve_runtime(&citation, &dir, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::MissingCaptureKey);
    }

    #[test]
    fn resolve_runtime_capture_without_hash_key() {
        let dir = temp_fixture("runtime-no-hash");
        let capture_path = dir.join("observed/runtime/foo.json");
        std::fs::create_dir_all(capture_path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&capture_path, r#"{"present":true}"#).expect("write capture");
        let mut cache = HashMap::new();
        let citation = test_runtime_citation("observed/runtime/foo.json");
        let resolved = resolve_runtime(&citation, &dir, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::MissingCaptureKey);
        assert_eq!(
            resolved.detail.as_deref(),
            Some("capture path missing #key suffix")
        );
    }

    #[test]
    fn resolve_doc_skipped_when_docs_checkout_missing() {
        let dir = temp_fixture("doc-skipped");
        let mut line_cache = HashMap::new();
        let mut json_cache = HashMap::new();
        let citation = test_citation(CitationKind::Doc, "content/example.md", Some(1));
        let resolved = resolve_one(
            &citation,
            &dir,
            &dir,
            &dir.join("missing-docs"),
            false,
            &mut line_cache,
            &mut json_cache,
        );
        assert_eq!(resolved.status, ResolveStatus::Skipped);
        assert!(!gate_failed(&ProvenanceWalkReport {
            citations: vec![resolved],
            registries: Vec::new(),
            docs_root_present: false,
        }));
    }

    #[test]
    fn resolve_doc_line_zero_is_fatal_without_docs_checkout() {
        let dir = temp_fixture("doc-line-zero-no-docs");
        let mut line_cache = HashMap::new();
        let mut json_cache = HashMap::new();
        let citation = test_citation(
            CitationKind::Doc,
            "catalog/extras/qa-harness-improve",
            Some(0),
        );
        let resolved = resolve_one(
            &citation,
            &dir,
            &dir,
            &dir.join("missing-docs"),
            false,
            &mut line_cache,
            &mut json_cache,
        );
        assert_eq!(resolved.status, ResolveStatus::LineOutOfRange);
        assert!(gate_failed(&ProvenanceWalkReport {
            citations: vec![resolved],
            registries: Vec::new(),
            docs_root_present: false,
        }));
    }

    #[test]
    fn resolve_source_docs_path_is_misclassified() {
        let dir = temp_fixture("source-misclassified");
        let docs = dir.join("BlueOS-docs");
        std::fs::create_dir_all(docs.join("content/development/core")).expect("mkdir docs");
        std::fs::write(
            docs.join("content/development/core/index.md"),
            "commander row\n",
        )
        .expect("write doc");
        let mut line_cache = HashMap::new();
        let mut json_cache = HashMap::new();
        let citation = test_citation(
            CitationKind::Source,
            "content/development/core/index.md",
            Some(1),
        );
        let resolved = resolve_one(
            &citation,
            &dir,
            &dir,
            &docs,
            true,
            &mut line_cache,
            &mut json_cache,
        );
        assert_eq!(resolved.status, ResolveStatus::Misclassified);
        assert_eq!(
            resolved.detail.as_deref(),
            Some("docs path filed as Source")
        );
        assert!(gate_failed(&ProvenanceWalkReport {
            citations: vec![resolved],
            registries: Vec::new(),
            docs_root_present: true,
        }));
    }

    #[test]
    fn resolve_doc_repo_path_is_misclassified() {
        let dir = temp_fixture("doc-misclassified");
        std::fs::create_dir_all(dir.join("catalog/extras/qa-harness-improve")).expect("mkdir");
        let mut line_cache = HashMap::new();
        let mut json_cache = HashMap::new();
        let citation = test_citation(
            CitationKind::Doc,
            "catalog/extras/qa-harness-improve",
            Some(1),
        );
        let resolved = resolve_one(
            &citation,
            &dir,
            &dir,
            &dir.join("missing-docs"),
            false,
            &mut line_cache,
            &mut json_cache,
        );
        assert_eq!(resolved.status, ResolveStatus::Misclassified);
        assert!(gate_failed(&ProvenanceWalkReport {
            citations: vec![resolved],
            registries: Vec::new(),
            docs_root_present: false,
        }));
    }

    #[test]
    fn resolve_asserted_is_no_file_reference_and_non_fatal() {
        let dir = temp_fixture("asserted-no-file");
        let mut line_cache = HashMap::new();
        let mut json_cache = HashMap::new();
        let citation = Citation {
            kind: CitationKind::Asserted,
            file: String::new(),
            line: None,
            anchor: None,
            capture: None,
            capture_key: None,
            environment: None,
            model_path: "/test".to_string(),
            root: "test".to_string(),
        };
        let resolved = resolve_one(
            &citation,
            &dir,
            &dir,
            &dir.join("missing-docs"),
            false,
            &mut line_cache,
            &mut json_cache,
        );
        assert_eq!(resolved.status, ResolveStatus::NoFileReference);
        assert!(!gate_failed(&ProvenanceWalkReport {
            citations: vec![resolved],
            registries: Vec::new(),
            docs_root_present: false,
        }));
    }

    #[test]
    fn resolve_empty_anchor_is_unanchored_and_non_fatal() {
        let dir = temp_fixture("empty-anchor");
        let file = dir.join("sample.txt");
        std::fs::write(&file, "alpha\n").expect("write sample");
        let mut cache = HashMap::new();
        let mut citation = test_citation(CitationKind::Observed, "sample.txt", Some(1));
        citation.anchor = Some(String::new());
        let resolved = resolve_file_line(&citation, &file, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::Unanchored);
        assert!(!gate_failed(&ProvenanceWalkReport {
            citations: vec![resolved],
            registries: Vec::new(),
            docs_root_present: true,
        }));
    }

    fn anchored_citation(file: &str, line: u32, anchor: &str) -> Citation {
        let mut citation = test_citation(CitationKind::Observed, file, Some(line));
        citation.anchor = Some(anchor.to_string());
        citation
    }

    fn write_unique_target(dir: &std::path::Path) -> std::path::PathBuf {
        let path = dir.join("target.py");
        let mut body = String::new();
        for i in 1..=5 {
            body.push_str(&format!("filler_line_{i}_padding_text\n"));
        }
        body.push_str("unique_verify_anchor_token_alpha\n");
        for i in 7..=12 {
            body.push_str(&format!("filler_line_{i}_padding_text\n"));
        }
        std::fs::write(&path, &body).expect("write target");
        path
    }

    fn fatal_report(resolved: ResolvedCitation) -> ProvenanceWalkReport {
        ProvenanceWalkReport {
            citations: vec![resolved],
            registries: Vec::new(),
            docs_root_present: true,
        }
    }

    #[test]
    fn resolve_shifted_line_is_relocated_and_fatal() {
        let dir = temp_fixture("resolve-reloc");
        let path = write_unique_target(&dir);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .expect("read")
            .lines()
            .map(str::to_string)
            .collect();
        for i in 0..3 {
            lines.insert(0, format!("inserted_pad_{i}"));
        }
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");
        let mut cache = HashMap::new();
        let citation = anchored_citation("target.py", 6, "unique_verify_anchor_token_alpha");
        let resolved = resolve_file_line(&citation, &path, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::Relocated);
        assert!(gate_failed(&fatal_report(resolved)));
    }

    #[test]
    fn resolve_deleted_line_is_anchor_lost_and_fatal() {
        let dir = temp_fixture("resolve-lost");
        let path = write_unique_target(&dir);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .expect("read")
            .lines()
            .map(str::to_string)
            .collect();
        lines.remove(5);
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");
        let mut cache = HashMap::new();
        let citation = anchored_citation("target.py", 6, "unique_verify_anchor_token_alpha");
        let resolved = resolve_file_line(&citation, &path, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::AnchorLost);
        assert!(gate_failed(&fatal_report(resolved)));
    }

    #[test]
    fn resolve_duplicated_anchor_is_ambiguous_and_fatal() {
        let dir = temp_fixture("resolve-ambig");
        let path = write_unique_target(&dir);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .expect("read")
            .lines()
            .map(str::to_string)
            .collect();
        let token = lines[5].clone();
        lines[5] = "dummy_replaced_cited_line".to_string();
        lines.insert(7, token.clone());
        lines.insert(9, token);
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");
        let mut cache = HashMap::new();
        let citation = anchored_citation("target.py", 6, "unique_verify_anchor_token_alpha");
        let resolved = resolve_file_line(&citation, &path, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::AmbiguousAnchor);
        assert!(gate_failed(&fatal_report(resolved)));
    }

    #[test]
    fn resolve_moved_beyond_window_is_anchor_moved_far_and_fatal() {
        let dir = temp_fixture("resolve-far");
        let path = write_unique_target(&dir);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .expect("read")
            .lines()
            .map(str::to_string)
            .collect();
        let token = lines[5].clone();
        lines[5] = "dummy_replaced_cited_line".to_string();
        for _ in 0..60 {
            lines.push("far_pad".to_string());
        }
        lines.push(token);
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");
        let mut cache = HashMap::new();
        let citation = anchored_citation("target.py", 6, "unique_verify_anchor_token_alpha");
        let resolved = resolve_file_line(&citation, &path, &mut cache);
        assert_eq!(resolved.status, ResolveStatus::AnchorMovedFar);
        assert!(gate_failed(&fatal_report(resolved)));
    }

    #[test]
    fn mark_applied_relocations_clears_relocated_so_gate_passes() {
        let pardal_idx = catalog_data::SERVICES
            .iter()
            .position(|service| service.id == catalog_kernel::id::service::ServiceId::Pardal)
            .expect("pardal service");
        let mut citation = anchored_citation("core/foo.py", 10, "unique_anchor_text");
        citation.root = "catalog/bootstrap".to_string();
        citation.model_path = format!("/services/{pardal_idx}/observed/x");
        let site = catalog_site_for_citation(&citation).expect("pardal site");
        let mut report = fatal_report(ResolvedCitation {
            citation: citation.clone(),
            status: ResolveStatus::Relocated,
            detail: Some("anchor found at line 20".to_string()),
        });
        assert!(gate_failed(&report));
        mark_applied_relocations_resolved(
            &mut report,
            &[LineRelocation {
                site,
                file: "core/foo.py".to_string(),
                old_line: 10,
                new_line: 20,
                anchor: "unique_anchor_text".to_string(),
            }],
        );
        assert_eq!(report.citations[0].status, ResolveStatus::Resolved);
        assert_eq!(report.citations[0].citation.line, Some(20));
        assert!(!gate_failed(&report));
    }

    #[test]
    fn mark_applied_relocations_is_scoped_to_site_for_shared_triple() {
        let mut first = anchored_citation("core/foo.py", 10, "unique_anchor_text");
        first.root = "catalog/bootstrap".to_string();
        first.model_path = "/services/0/observed/x".to_string();
        let mut second = first.clone();
        second.model_path = "/pages/0/x".to_string();
        let site_first = catalog_site_for_citation(&first).expect("service site");
        let site_second = catalog_site_for_citation(&second).expect("page site");
        assert_ne!(site_first, site_second);
        let mut report = ProvenanceWalkReport {
            citations: vec![
                ResolvedCitation {
                    citation: first,
                    status: ResolveStatus::Relocated,
                    detail: Some("anchor found at line 20".to_string()),
                },
                ResolvedCitation {
                    citation: second,
                    status: ResolveStatus::Relocated,
                    detail: Some("anchor found at line 20".to_string()),
                },
            ],
            registries: Vec::new(),
            docs_root_present: true,
        };
        mark_applied_relocations_resolved(
            &mut report,
            &[LineRelocation {
                site: site_first,
                file: "core/foo.py".to_string(),
                old_line: 10,
                new_line: 20,
                anchor: "unique_anchor_text".to_string(),
            }],
        );
        assert_eq!(report.citations[0].status, ResolveStatus::Resolved);
        assert_eq!(report.citations[1].status, ResolveStatus::Relocated);
        assert!(gate_failed(&report));
        let _ = site_second;
    }

    #[test]
    fn mark_applied_relocations_skips_siteless_citation_for_shared_triple() {
        let citation = anchored_citation("core/foo.py", 10, "unique_anchor_text");
        assert!(catalog_site_for_citation(&citation).is_none());
        let mut report = fatal_report(ResolvedCitation {
            citation,
            status: ResolveStatus::Relocated,
            detail: Some("anchor found at line 20".to_string()),
        });
        mark_applied_relocations_resolved(
            &mut report,
            &[LineRelocation {
                site: PathBuf::from("services/foo.rs"),
                file: "core/foo.py".to_string(),
                old_line: 10,
                new_line: 20,
                anchor: "unique_anchor_text".to_string(),
            }],
        );
        assert_eq!(report.citations[0].status, ResolveStatus::Relocated);
        assert_eq!(report.citations[0].citation.line, Some(10));
        assert!(gate_failed(&report));
    }

    #[test]
    fn mark_applied_relocations_distinguishes_wifi_service_and_journey_sites() {
        let wifi_service_idx = catalog_data::SERVICES
            .iter()
            .position(|service| service.id == catalog_kernel::id::service::ServiceId::Wifi)
            .expect("wifi service");
        let wifi_journey_idx = catalog_data::journeys::all_journeys()
            .iter()
            .position(|journey| {
                catalog_data::journeys::source_file_for_journey(journey.id)
                    == Some("journeys/wifi.rs")
            })
            .expect("wifi journey");
        let mut service = anchored_citation("core/services/wifi/main.py", 10, "shared_anchor");
        service.root = "catalog/bootstrap".to_string();
        service.model_path = format!("/services/{wifi_service_idx}/observed/x");
        let mut journey = service.clone();
        journey.model_path = format!("/journeys/{wifi_journey_idx}/steps/0/route");
        let site_service = catalog_site_for_citation(&service).expect("service site");
        let site_journey = catalog_site_for_citation(&journey).expect("journey site");
        assert_eq!(site_service, PathBuf::from("services/wifi.rs"));
        assert_eq!(site_journey, PathBuf::from("journeys/wifi.rs"));
        assert_ne!(site_service, site_journey);
        let mut report = ProvenanceWalkReport {
            citations: vec![
                ResolvedCitation {
                    citation: service,
                    status: ResolveStatus::Relocated,
                    detail: Some("anchor found at line 20".to_string()),
                },
                ResolvedCitation {
                    citation: journey,
                    status: ResolveStatus::Relocated,
                    detail: Some("anchor found at line 20".to_string()),
                },
            ],
            registries: Vec::new(),
            docs_root_present: true,
        };
        mark_applied_relocations_resolved(
            &mut report,
            &[LineRelocation {
                site: site_service,
                file: "core/services/wifi/main.py".to_string(),
                old_line: 10,
                new_line: 20,
                anchor: "shared_anchor".to_string(),
            }],
        );
        assert_eq!(report.citations[0].status, ResolveStatus::Resolved);
        assert_eq!(report.citations[1].status, ResolveStatus::Relocated);
        assert!(gate_failed(&report));
    }

    #[test]
    fn relocation_site_matches_rejects_wifi_filename_suffix_collision() {
        let bare = PathBuf::from("wifi.rs");
        let service = PathBuf::from("services/wifi.rs");
        let journey = PathBuf::from("journeys/wifi.rs");
        assert!(!relocation_site_matches(&service, &journey));
        assert!(!relocation_site_matches(&service, &bare));
        assert!(!relocation_site_matches(&journey, &bare));
        assert!(service.ends_with(&bare) && journey.ends_with(&bare));
    }

    #[test]
    fn unrepaired_relocation_is_reported_and_keeps_gate_failed() {
        let dir = temp_fixture("unrepaired-fix");
        let site = dir.join("module.rs");
        std::fs::write(&site, "fn noop() {}\n").expect("write");
        let rel = LineRelocation {
            site: site.clone(),
            file: "core/foo.py".to_string(),
            old_line: 10,
            new_line: 20,
            anchor: "unique_anchor_text".to_string(),
        };
        let applied = crate::provenance_anchor::apply_relocated_line_fixes_in(
            &dir,
            std::slice::from_ref(&rel),
        )
        .expect("apply");
        assert!(applied.is_empty());
        let requested = [rel.clone()];
        let unrepaired = unrepaired_relocations(&requested, &applied);
        assert_eq!(unrepaired.len(), 1);
        let msg = format_unrepaired_fix(&rel);
        assert!(msg.contains("core/foo.py"), "{msg}");
        assert!(msg.contains(":10"), "{msg}");
        assert!(msg.contains("no repair pattern matched"), "{msg}");
        let citation = anchored_citation("core/foo.py", 10, "unique_anchor_text");
        let mut report = fatal_report(ResolvedCitation {
            citation,
            status: ResolveStatus::Relocated,
            detail: Some("anchor found at line 20".to_string()),
        });
        mark_applied_relocations_resolved(&mut report, &applied);
        assert_eq!(report.citations[0].status, ResolveStatus::Relocated);
        assert!(gate_failed(&report));
    }

    #[test]
    fn mark_applied_relocations_leaves_unrepaired_relocated_fatal() {
        let citation = anchored_citation("core/foo.py", 10, "unique_anchor_text");
        let report = fatal_report(ResolvedCitation {
            citation,
            status: ResolveStatus::Relocated,
            detail: Some("anchor found at line 20".to_string()),
        });
        assert!(gate_failed(&report));
    }

    #[test]
    fn path_constants_resolve_by_existence() {
        let report = build_report();
        for name in [
            "coverage/TRACKER_CSV_PATH",
            "api_contract/DEFAULT_API_CONTRACT_BASELINE",
            "harness_ratchet/DEFAULT_BASELINE_PATH",
            "harness_ratchet/FAILURE_MODE_LEDGER_PATH",
        ] {
            let hits: Vec<_> = report
                .citations
                .iter()
                .filter(|item| item.citation.root == name)
                .collect();
            assert_eq!(hits.len(), 1, "{name}");
            assert_eq!(hits[0].status, ResolveStatus::AnchorExempt, "{name}");
            assert!(hits[0].citation.line.is_none(), "{name}");
        }
    }
}
