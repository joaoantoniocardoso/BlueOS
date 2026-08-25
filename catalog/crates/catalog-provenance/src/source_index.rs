use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::provenance_walk::{
    build_report, catalog_site_for_citation, Citation, CitationKind, ProvenanceWalkReport,
    ResolveStatus,
};
use catalog_core::frontend_routes::FRONTEND_API_ENDPOINTS;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::page::PageId;
use catalog_kernel::id::service::ServiceId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogEntity {
    Service { id: ServiceId },
    Journey { id: JourneyId },
    Page { id: PageId },
    Unknown { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceIndexEntry {
    pub entity: CatalogEntity,
    pub module: String,
    pub model_path: String,
    pub root: String,
    pub kind: CitationKind,
    pub status: ResolveStatus,
    pub line: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceIndex {
    pub entries: BTreeMap<String, Vec<SourceIndexEntry>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewUrgency {
    NeedsReview,
    ConfirmedDrifted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkOrderEntry {
    pub path: String,
    pub entity: CatalogEntity,
    pub model_path: String,
    pub line: Option<u32>,
    pub resolve_status: ResolveStatus,
    pub urgency: ReviewUrgency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkOrder {
    pub since: String,
    pub until: String,
    /// Indexed citation paths that changed in this repo and appear in the work order.
    pub impacted_paths: Vec<String>,
    /// Paths returned by `git diff` over repo-local index prefixes (includes non-cited files).
    pub repo_diff_path_count: usize,
    pub note: &'static str,
    pub doc_scope_note: &'static str,
    pub modules: BTreeMap<String, Vec<WorkOrderEntry>>,
}

pub const WORK_ORDER_NOTE: &str = "A cited file changing is enough to require re-examination even \
    when the anchor still matches. Anchor match proves the cited LINE is intact, not that \
    surrounding behaviour still supports the claim.";

pub const DOC_SCOPE_NOTE: &str = "--since..--until covers repo-local citation paths only. Doc \
    citations (content/* under ../BlueOS-docs) are not diffed; re-check those separately when \
    docs change.";

pub fn build_source_index(report: &ProvenanceWalkReport) -> SourceIndex {
    let mut entries: BTreeMap<String, Vec<SourceIndexEntry>> = BTreeMap::new();
    for item in &report.citations {
        let Some(path) = citation_source_path(&item.citation).map(str::to_string) else {
            continue;
        };
        let entity = catalog_entity_for_citation(&item.citation);
        let module = catalog_module_for_entity(&entity, &item.citation);
        entries.entry(path).or_default().push(SourceIndexEntry {
            entity,
            module,
            model_path: item.citation.model_path.clone(),
            root: item.citation.root.clone(),
            kind: item.citation.kind,
            status: item.status,
            line: item.citation.line,
        });
    }
    SourceIndex { entries }
}

pub fn build_source_index_from_walk() -> SourceIndex {
    build_source_index(&build_report())
}

pub fn indexable_source_paths(report: &ProvenanceWalkReport) -> HashSet<String> {
    linter_source_paths_from_fields(report)
}

/// Source paths derived only from citation struct fields. Does not call
/// [`citation_source_path`]; used to verify the index is complete.
pub fn linter_source_paths_from_fields(report: &ProvenanceWalkReport) -> HashSet<String> {
    report
        .citations
        .iter()
        .filter_map(|item| linter_source_path_from_fields(&item.citation))
        .collect()
}

fn linter_source_path_from_fields(citation: &Citation) -> Option<String> {
    if citation.kind == CitationKind::Asserted {
        return None;
    }
    match citation.kind {
        CitationKind::Runtime => {
            let capture = match citation.capture.as_deref() {
                Some(value) => value,
                None if citation.file.is_empty() => return None,
                None => citation.file.as_str(),
            };
            let file = capture
                .split_once('#')
                .map(|(path, _)| path)
                .unwrap_or(capture);
            if file.is_empty() {
                None
            } else {
                Some(file.to_string())
            }
        }
        _ if citation.file.is_empty() => None,
        _ => Some(citation.file.clone()),
    }
}

pub fn citation_source_path(citation: &Citation) -> Option<&str> {
    match citation.kind {
        CitationKind::Asserted => None,
        CitationKind::Runtime => {
            let capture = citation
                .capture
                .as_deref()
                .unwrap_or(citation.file.as_str());
            if capture.is_empty() {
                return None;
            }
            Some(
                capture
                    .split_once('#')
                    .map(|(file, _)| file)
                    .unwrap_or(capture),
            )
        }
        _ if citation.file.is_empty() => None,
        _ => Some(&citation.file),
    }
}

pub fn is_confirmed_drift(status: ResolveStatus) -> bool {
    matches!(
        status,
        ResolveStatus::Relocated
            | ResolveStatus::AmbiguousAnchor
            | ResolveStatus::AnchorMovedFar
            | ResolveStatus::AnchorLost
            | ResolveStatus::MissingFile
            | ResolveStatus::LineOutOfRange
            | ResolveStatus::Misclassified
            | ResolveStatus::MissingCaptureKey
    )
}

pub fn review_urgency(status: ResolveStatus) -> ReviewUrgency {
    if is_confirmed_drift(status) {
        ReviewUrgency::ConfirmedDrifted
    } else {
        ReviewUrgency::NeedsReview
    }
}

pub fn catalog_entity_for_citation(citation: &Citation) -> CatalogEntity {
    if citation.root == "catalog/bootstrap" {
        return bootstrap_entity(citation);
    }
    if citation.root == "frontend_routes/FRONTEND_API_ENDPOINTS" {
        return frontend_entity(citation);
    }
    if matches!(
        citation.root.as_str(),
        "coverage/TRACKER_CSV_PATH"
            | "api_contract/DEFAULT_API_CONTRACT_BASELINE"
            | "harness_ratchet/DEFAULT_BASELINE_PATH"
            | "harness_ratchet/FAILURE_MODE_LEDGER_PATH"
    ) {
        return CatalogEntity::Unknown {
            reason: format!("catalog path constant ({})", citation.root),
        };
    }
    CatalogEntity::Unknown {
        reason: format!("unattributed root {}", citation.root),
    }
}

pub fn catalog_module_for_entity(entity: &CatalogEntity, citation: &Citation) -> String {
    if let Some(site) = catalog_site_for_citation(citation) {
        return site.to_string_lossy().into_owned();
    }
    match entity {
        CatalogEntity::Service { id } => format!("services/{}.rs", service_file_stem(*id)),
        CatalogEntity::Journey { id } => catalog_data::journeys::source_file_for_journey(*id)
            .unwrap_or("journeys/unknown.rs")
            .to_string(),
        CatalogEntity::Page { id } => format!("pages/{}.rs", id.as_str()),
        CatalogEntity::Unknown { .. } => "unknown".to_string(),
    }
}

pub fn git_diff_repo_paths(
    repo: &Path,
    since: &str,
    until: &str,
    pathspecs: &[String],
) -> Result<Vec<String>, String> {
    let mut cmd = vec!["diff", "--name-only", since, until, "--"];
    for pathspec in pathspecs {
        cmd.push(pathspec.as_str());
    }
    let output = Command::new("git")
        .args(&cmd)
        .current_dir(repo)
        .output()
        .map_err(|err| format!("failed to spawn git: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            cmd.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(repo_path_to_index_key)
        .collect())
}

pub fn work_order_for_changed_paths(
    index: &SourceIndex,
    since: &str,
    until: &str,
    impacted_paths: &[String],
    repo_diff_path_count: usize,
) -> WorkOrder {
    let mut modules: BTreeMap<String, Vec<WorkOrderEntry>> = BTreeMap::new();
    for path in impacted_paths {
        let Some(entries) = index.entries.get(path) else {
            continue;
        };
        for entry in entries {
            modules
                .entry(entry.module.clone())
                .or_default()
                .push(WorkOrderEntry {
                    path: path.clone(),
                    entity: entry.entity.clone(),
                    model_path: entry.model_path.clone(),
                    line: entry.line,
                    resolve_status: entry.status,
                    urgency: review_urgency(entry.status),
                });
        }
    }
    WorkOrder {
        since: since.to_string(),
        until: until.to_string(),
        impacted_paths: impacted_paths.to_vec(),
        repo_diff_path_count,
        note: WORK_ORDER_NOTE,
        doc_scope_note: DOC_SCOPE_NOTE,
        modules,
    }
}

fn bootstrap_entity(citation: &Citation) -> CatalogEntity {
    let segs: Vec<&str> = citation
        .model_path
        .trim_start_matches('/')
        .split('/')
        .collect();
    match segs.as_slice() {
        ["services", idx, ..] => {
            let index: usize = match idx.parse() {
                Ok(value) => value,
                Err(_) => {
                    return CatalogEntity::Unknown {
                        reason: format!("invalid service index {idx}"),
                    };
                }
            };
            match catalog_data::SERVICES.get(index) {
                Some(service) => CatalogEntity::Service { id: service.id },
                None => CatalogEntity::Unknown {
                    reason: format!("service index {index} out of range"),
                },
            }
        }
        ["pages", idx, ..] => {
            let index: usize = match idx.parse() {
                Ok(value) => value,
                Err(_) => {
                    return CatalogEntity::Unknown {
                        reason: format!("invalid page index {idx}"),
                    };
                }
            };
            match catalog_data::frontend::PAGES.get(index) {
                Some(page) => CatalogEntity::Page { id: page.id },
                None => CatalogEntity::Unknown {
                    reason: format!("page index {index} out of range"),
                },
            }
        }
        ["journeys", idx, ..] => {
            let index: usize = match idx.parse() {
                Ok(value) => value,
                Err(_) => {
                    return CatalogEntity::Unknown {
                        reason: format!("invalid journey index {idx}"),
                    };
                }
            };
            match catalog_data::journeys::all_journeys().get(index) {
                Some(journey) => CatalogEntity::Journey { id: journey.id },
                None => CatalogEntity::Unknown {
                    reason: format!("journey index {index} out of range"),
                },
            }
        }
        _ => CatalogEntity::Unknown {
            reason: format!("unattributed bootstrap path {}", citation.model_path),
        },
    }
}

fn frontend_entity(citation: &Citation) -> CatalogEntity {
    let idx_str = citation
        .model_path
        .trim_start_matches('/')
        .split('/')
        .next();
    let Some(idx_str) = idx_str else {
        return CatalogEntity::Unknown {
            reason: "frontend endpoint index missing".to_string(),
        };
    };
    if idx_str.is_empty() {
        return CatalogEntity::Unknown {
            reason: "frontend endpoint index missing".to_string(),
        };
    }
    let index: usize = match idx_str.parse() {
        Ok(value) => value,
        Err(_) => {
            return CatalogEntity::Unknown {
                reason: format!("invalid frontend index {idx_str}"),
            };
        }
    };
    match FRONTEND_API_ENDPOINTS.get(index) {
        Some(endpoint) => CatalogEntity::Service {
            id: endpoint.base.service,
        },
        None => CatalogEntity::Unknown {
            reason: format!("frontend index {index} out of range"),
        },
    }
}

fn service_file_stem(id: ServiceId) -> &'static str {
    match id {
        ServiceId::MavlinkCameraManager => "mavlink_camera_manager",
        other => other.as_str(),
    }
}

pub fn index_key_to_repo_path(key: &str) -> String {
    if let Some(rest) = key.strip_prefix("runtime-captures/") {
        format!("catalog/runtime-captures/{rest}")
    } else {
        key.to_string()
    }
}

pub fn repo_path_to_index_key(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("catalog/runtime-captures/") {
        format!("runtime-captures/{rest}")
    } else {
        path.to_string()
    }
}

pub fn index_git_diff_pathspecs(index: &SourceIndex) -> Vec<String> {
    let mut pathspecs: HashSet<String> = HashSet::new();
    for key in index.entries.keys() {
        let repo_path = index_key_to_repo_path(key);
        let top = repo_path.split('/').next().unwrap_or(repo_path.as_str());
        if top == "content" {
            continue;
        }
        pathspecs.insert(format!("{top}/"));
    }
    let mut specs: Vec<_> = pathspecs.into_iter().collect();
    specs.sort();
    specs
}

#[cfg(test)]
fn unique_path_count_by_kind(report: &ProvenanceWalkReport, kind: CitationKind) -> usize {
    report
        .citations
        .iter()
        .filter(|item| item.citation.kind == kind)
        .filter_map(|item| linter_source_path_from_fields(&item.citation))
        .collect::<HashSet<_>>()
        .len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance_walk::build_report;
    use catalog_core::catalog::Catalog;
    use catalog_kernel::id::journey::JourneyId;

    #[test]
    fn bootstrap_slices_match_attribution_index_sources() {
        let catalog = Catalog::bootstrap();
        assert_eq!(catalog.services().len(), catalog_data::SERVICES.len());
        for (idx, service) in catalog.services().iter().enumerate() {
            assert_eq!(
                service.id,
                catalog_data::SERVICES[idx].id,
                "service index {idx} diverged from SERVICES slice"
            );
        }

        let journeys = catalog_data::journeys::all_journeys();
        assert_eq!(catalog.journeys().len(), journeys.len());
        for (idx, journey) in catalog.journeys().iter().enumerate() {
            assert_eq!(
                journey.id, journeys[idx].id,
                "journey index {idx} diverged from all_journeys() order"
            );
        }

        assert_eq!(catalog.pages().len(), catalog_data::frontend::PAGES.len());
        for (idx, page) in catalog.pages().iter().enumerate() {
            assert_eq!(
                page.id,
                catalog_data::frontend::PAGES[idx].id,
                "page index {idx} diverged from PAGES slice"
            );
        }
    }

    #[test]
    fn index_covers_linter_source_paths_and_key_count_is_pinned() {
        let report = build_report();
        let index = build_source_index(&report);
        let expected = linter_source_paths_from_fields(&report);
        let indexed: HashSet<_> = index.entries.keys().cloned().collect();

        assert_eq!(indexed.len(), 239, "index key count drifted");
        assert_eq!(
            indexed,
            expected,
            "index missing {} paths",
            expected.difference(&indexed).count()
        );
        const OBSERVED_PATH_COUNT: usize = 159;
        const SOURCE_PATH_COUNT: usize = 111;
        const DOC_PATH_COUNT: usize = 8;
        const RUNTIME_PATH_COUNT: usize = 27;

        assert_eq!(
            unique_path_count_by_kind(&report, CitationKind::Observed),
            OBSERVED_PATH_COUNT
        );
        assert_eq!(
            unique_path_count_by_kind(&report, CitationKind::Source),
            SOURCE_PATH_COUNT
        );
        assert_eq!(
            unique_path_count_by_kind(&report, CitationKind::Doc),
            DOC_PATH_COUNT
        );
        assert_eq!(
            unique_path_count_by_kind(&report, CitationKind::Runtime),
            RUNTIME_PATH_COUNT
        );
    }

    #[test]
    fn nmea_injector_main_py_maps_to_expected_entities() {
        let index = build_source_index(&build_report());
        let path = "core/services/nmea_injector/main.py";
        let entries = index
            .entries
            .get(path)
            .unwrap_or_else(|| panic!("path not indexed: {path}"));
        let entities: HashSet<CatalogEntity> =
            entries.iter().map(|entry| entry.entity.clone()).collect();
        assert!(
            entities.contains(&CatalogEntity::Service {
                id: ServiceId::NmeaInjector
            }),
            "missing Service(nmea_injector); got {entities:?}"
        );
        for journey in [
            JourneyId::ViewConfiguredNmeaSockets,
            JourneyId::AddExternalNmeaGpsSocket,
            JourneyId::RemoveConfiguredNmeaSocket,
        ] {
            assert!(
                entities.contains(&CatalogEntity::Journey { id: journey }),
                "missing Journey({journey}); got {entities:?}"
            );
        }
    }

    #[test]
    fn uncited_core_path_is_absent_from_index() {
        let index = build_source_index(&build_report());
        assert!(!index
            .entries
            .contains_key("core/services/nonexistent_probe/main.py"));
    }

    #[test]
    fn git_diff_until_ref_limits_range_not_head() {
        use catalog_paths::repo_root;

        let repo = repo_root();
        let index = build_source_index_from_walk();
        let pathspecs = index_git_diff_pathspecs(&index);
        let since = "1.4.4-beta.14";
        let until = "1.4.4-beta.20";
        for reference in [since, until, "HEAD"] {
            assert!(
                Command::new("git")
                    .args(["rev-parse", "--verify", reference])
                    .current_dir(&repo)
                    .status()
                    .expect("spawn git")
                    .success(),
                "missing git ref {reference}"
            );
        }

        let to_head = git_diff_repo_paths(&repo, since, "HEAD", &pathspecs).expect("diff to HEAD");
        let to_until = git_diff_repo_paths(&repo, since, until, &pathspecs).expect("diff to until");
        let to_head_again =
            git_diff_repo_paths(&repo, since, "HEAD", &pathspecs).expect("diff to HEAD again");

        assert_ne!(
            to_head.len(),
            to_until.len(),
            "--until must change repo_diff_path_count (HEAD={}, until={})",
            to_head.len(),
            to_until.len()
        );
        assert!(
            to_until.len() < to_head.len(),
            "tag-to-tag range should be narrower than tag-to-HEAD"
        );
        assert_eq!(
            to_head, to_head_again,
            "explicit HEAD must match implicit default"
        );
    }
}
