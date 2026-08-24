use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::domain::Domain;
use crate::provenance::{Grounded, GroundedSet, Provenance};
use crate::requirement::{
    Assumption, Requirement, RequirementCatalog, RequirementClass, RequirementCriteria,
    RequirementStatement,
};
use crate::system_overlay::SYSTEM_OVERLAY_ENTRIES;

pub const REQUIREMENTS_BASELINE_DIR: &str = "requirements-baselines";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementsJson {
    pub version_tag: String,
    pub total: usize,
    pub unknown_statement_count: usize,
    pub unknown_criteria_count: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub requirements: Vec<RequirementJsonRow>,
    pub contamination_findings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementJsonRow {
    pub id: String,
    pub kind: RequirementClass,
    pub domain: Domain,
    pub statement: RequirementStatement,
    pub criteria: RequirementCriteria,
    pub feature_id: Option<String>,
    pub journey_id: Option<String>,
    #[serde(default)]
    pub function_id: Option<String>,
    #[serde(default)]
    pub verifying_journeys: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<Assumption>,
    pub availability_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequirementDiffReport {
    pub baseline_tag: String,
    pub current_tag: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed_statement: Vec<RequirementFieldChange>,
    pub changed_criteria: Vec<RequirementFieldChange>,
    pub changed_availability: Vec<RequirementAvailabilityChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequirementFieldChange {
    pub id: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequirementAvailabilityChange {
    pub id: String,
    pub before_present: bool,
    pub after_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtmRow {
    pub requirement_id: String,
    pub kind: RequirementClass,
    pub feature: String,
    pub function: String,
    pub journey: String,
    pub evidence: String,
}

pub fn requirements_json(catalog: &RequirementCatalog, version_tag: &str) -> RequirementsJson {
    let filtered = catalog.filter_by_version(version_tag);
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut unknown_statement_count = 0usize;
    let mut unknown_criteria_count = 0usize;
    let mut requirements = Vec::new();
    for requirement in filtered.requirements() {
        *by_kind
            .entry(format!("{:?}", requirement.kind).to_ascii_lowercase())
            .or_default() += 1;
        if matches!(requirement.statement, RequirementStatement::Unknown { .. }) {
            unknown_statement_count += 1;
        }
        if matches!(requirement.criteria, RequirementCriteria::Unknown { .. }) {
            unknown_criteria_count += 1;
        }
        requirements.push(RequirementJsonRow {
            id: requirement.id.0.clone(),
            kind: requirement.kind,
            domain: requirement.domain,
            statement: requirement.statement.clone(),
            criteria: requirement.criteria.clone(),
            feature_id: requirement
                .feature_id
                .as_ref()
                .map(|capability| capability.as_str().to_string()),
            journey_id: requirement.journey_id.map(|journey| journey.to_string()),
            function_id: requirement
                .function_id
                .as_ref()
                .map(|function| function.as_str().to_string()),
            verifying_journeys: requirement
                .requirement_verifications
                .iter()
                .map(|journey| journey.to_string())
                .collect(),
            assumptions: requirement.assumptions.clone(),
            availability_present: requirement.availability.present_on_dut(version_tag),
        });
    }
    RequirementsJson {
        version_tag: version_tag.to_string(),
        total: requirements.len(),
        unknown_statement_count,
        unknown_criteria_count,
        by_kind,
        requirements,
        contamination_findings: filtered.contamination_findings().len(),
    }
}

pub fn render_srs(catalog: &RequirementCatalog, version_tag: &str) -> String {
    let filtered = catalog.filter_by_version(version_tag);
    let mut output = String::new();
    output.push_str(&format!(
        "# Software Requirements Specification ({version_tag})\n\n"
    ));
    output.push_str(&format!(
        "Total requirements: {} ({} unknown statements, {} unknown criteria)\n\n",
        filtered.requirements().len(),
        filtered
            .requirements()
            .iter()
            .filter(|req| matches!(req.statement, RequirementStatement::Unknown { .. }))
            .count(),
        filtered
            .requirements()
            .iter()
            .filter(|req| matches!(req.criteria, RequirementCriteria::Unknown { .. }))
            .count()
    ));

    let mut by_domain: HashMap<Domain, BTreeMap<String, Vec<&Requirement>>> = HashMap::new();
    for requirement in filtered.requirements() {
        by_domain
            .entry(requirement.domain)
            .or_default()
            .entry(requirement.aggregate.as_str().to_string())
            .or_default()
            .push(requirement);
    }

    for (domain, aggregates) in {
        let mut domains: Vec<_> = by_domain.into_iter().collect();
        domains.sort_by_key(|(domain, _)| domain.as_str());
        domains
    } {
        output.push_str(&format!("## Domain: {}\n\n", domain.as_str()));
        for (aggregate, requirements) in aggregates {
            output.push_str(&format!("### Aggregate: {aggregate}\n\n"));
            for requirement in requirements {
                output.push_str(&format!("#### {}\n\n", requirement.id.0));
                output.push_str(&format!("Kind: {:?}\n\n", requirement.kind));
                match &requirement.statement {
                    RequirementStatement::Known { text } => {
                        output.push_str(text);
                        output.push('\n');
                    }
                    RequirementStatement::Unknown { reason } => {
                        output.push_str(&format!("**Unknown statement:** {reason}\n"));
                    }
                }
                output.push_str("\nVerification:\n");
                match &requirement.criteria {
                    RequirementCriteria::Known { items } => {
                        for item in items {
                            output.push_str(&format!("- {text}\n", text = item.text));
                        }
                    }
                    RequirementCriteria::Unknown { reason } => {
                        output.push_str(&format!("- **Unknown criteria:** {reason}\n"));
                    }
                }
                if !requirement.assumptions.is_empty() {
                    output.push_str("\nAssumptions:\n");
                    for assumption in &requirement.assumptions {
                        output.push_str(&format!(
                            "- [{kind:?}] {statement} (use case: {journey})\n",
                            kind = assumption.kind,
                            statement = assumption.statement,
                            journey = assumption.source_use_case
                        ));
                    }
                }
                output.push('\n');
            }
        }
    }
    output
}

pub fn build_rtm_rows(catalog: &Catalog, requirements: &RequirementCatalog) -> Vec<RtmRow> {
    let mut rows = Vec::new();
    for requirement in requirements.requirements() {
        rows.push(rtm_row_for_requirement(
            catalog,
            requirements.requirements(),
            requirement,
        ));
    }
    rows
}

pub fn render_rtm_csv(catalog: &Catalog, requirements: &RequirementCatalog) -> String {
    let mut output = String::from("requirement_id,kind,feature,function,journey,evidence\n");
    for row in build_rtm_rows(catalog, requirements) {
        output.push_str(&format!(
            "{},{:?},{},{},{},{}\n",
            csv_escape(&row.requirement_id),
            row.kind,
            csv_escape(&row.feature),
            csv_escape(&row.function),
            csv_escape(&row.journey),
            csv_escape(&row.evidence),
        ));
    }
    output
}

pub fn validate_rtm_completeness(
    catalog: &Catalog,
    requirements: &RequirementCatalog,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for (requirement, row) in requirements
        .requirements()
        .iter()
        .zip(build_rtm_rows(catalog, requirements))
    {
        if !rtm_evidence_is_complete(requirement, &row.evidence) {
            errors.push(format!(
                "requirement {} lacks a resolvable RTM citation (got {:?})",
                row.requirement_id, row.evidence
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Every overlay trace must resolve and live in a committed file: an overlay claim grounded in an
/// artifact a reader cannot open is indistinguishable from an invented one.
pub fn overlay_trace_failures() -> Vec<String> {
    let mut failures = Vec::new();
    let repo = match tracked_paths(&crate::provenance_walk::repo_root()) {
        Ok(paths) => paths,
        Err(err) => {
            failures.push(err);
            return failures;
        }
    };
    let docs_root = crate::provenance_walk::docs_root();
    let docs = if docs_root.is_dir() {
        match tracked_paths(&docs_root) {
            Ok(paths) => Some(paths),
            Err(err) => {
                failures.push(err);
                return failures;
            }
        }
    } else {
        None
    };

    for entry in SYSTEM_OVERLAY_ENTRIES {
        if entry.traces.is_empty() {
            failures.push(format!("{}: cites no trace", entry.suffix));
        }
        for trace in entry.traces {
            if !evidence_resolves(trace) {
                failures.push(format!("{}: trace does not resolve: {trace}", entry.suffix));
                continue;
            }
            let Some((file, _)) = split_file_line(trace) else {
                continue;
            };
            let tracked = if file.starts_with("content/") {
                match &docs {
                    Some(docs) => docs.contains(file),
                    None => continue,
                }
            } else {
                repo.contains(file)
            };
            if !tracked {
                failures.push(format!("{}: trace is not committed: {trace}", entry.suffix));
            }
        }
    }
    failures
}

fn tracked_paths(dir: &Path) -> Result<HashSet<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|err| format!("git ls-files in {} failed: {err}", dir.display()))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files in {} exited with {}",
            dir.display(),
            output.status
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_string)
        .collect())
}

pub fn requirements_baseline_path(tag: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(REQUIREMENTS_BASELINE_DIR)
        .join(format!("{tag}.json"))
}

pub fn load_requirements_baseline(tag: &str) -> Result<RequirementsJson, String> {
    let path = requirements_baseline_path(tag);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("missing requirements baseline {}: {err}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("invalid requirements baseline {}: {err}", path.display()))
}

pub fn write_requirements_baseline(tag: &str, catalog: &RequirementCatalog) -> Result<(), String> {
    let path = requirements_baseline_path(tag);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create baseline dir {}: {err}", parent.display()))?;
    }
    let snapshot = requirements_json(catalog, tag);
    let content = serde_json::to_string_pretty(&snapshot)
        .map_err(|err| format!("serialize requirements baseline: {err}"))?;
    let display = path.display().to_string();
    fs::write(&path, format!("{content}\n"))
        .map_err(|err| format!("write requirements baseline {display}: {err}"))
}

pub fn diff_requirement_reports(
    baseline: &RequirementsJson,
    current: &RequirementsJson,
) -> RequirementDiffReport {
    let baseline_map: HashMap<&str, &RequirementJsonRow> = baseline
        .requirements
        .iter()
        .map(|req| (req.id.as_str(), req))
        .collect();
    let current_map: HashMap<&str, &RequirementJsonRow> = current
        .requirements
        .iter()
        .map(|req| (req.id.as_str(), req))
        .collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed_statement = Vec::new();
    let mut changed_criteria = Vec::new();
    let mut changed_availability = Vec::new();

    for (id, requirement) in &current_map {
        if !baseline_map.contains_key(id) {
            added.push((*id).to_string());
        } else {
            let before = baseline_map[id];
            if statement_key(&before.statement) != statement_key(&requirement.statement) {
                changed_statement.push(RequirementFieldChange {
                    id: (*id).to_string(),
                    before: statement_key(&before.statement),
                    after: statement_key(&requirement.statement),
                });
            }
            if criteria_key(&before.criteria) != criteria_key(&requirement.criteria) {
                changed_criteria.push(RequirementFieldChange {
                    id: (*id).to_string(),
                    before: criteria_key(&before.criteria),
                    after: criteria_key(&requirement.criteria),
                });
            }
            if before.availability_present != requirement.availability_present {
                changed_availability.push(RequirementAvailabilityChange {
                    id: (*id).to_string(),
                    before_present: before.availability_present,
                    after_present: requirement.availability_present,
                });
            }
        }
    }
    for id in baseline_map.keys() {
        if !current_map.contains_key(id) {
            removed.push((*id).to_string());
        }
    }

    added.sort();
    removed.sort();
    changed_statement.sort_by(|left, right| left.id.cmp(&right.id));
    changed_criteria.sort_by(|left, right| left.id.cmp(&right.id));
    changed_availability.sort_by(|left, right| left.id.cmp(&right.id));

    RequirementDiffReport {
        baseline_tag: baseline.version_tag.clone(),
        current_tag: current.version_tag.clone(),
        added,
        removed,
        changed_statement,
        changed_criteria,
        changed_availability,
    }
}

pub fn render_diff_report(diff: &RequirementDiffReport) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "requirements diff {} -> {}\n",
        diff.baseline_tag, diff.current_tag
    ));
    output.push_str(&format!("added: {}\n", diff.added.len()));
    for id in diff.added.iter().take(10) {
        output.push_str(&format!("  + {id}\n"));
    }
    output.push_str(&format!("removed: {}\n", diff.removed.len()));
    for id in diff.removed.iter().take(10) {
        output.push_str(&format!("  - {id}\n"));
    }
    output.push_str(&format!(
        "changed_statement: {}\n",
        diff.changed_statement.len()
    ));
    for change in diff.changed_statement.iter().take(10) {
        output.push_str(&format!(
            "  ~ {} :: {} -> {}\n",
            change.id, change.before, change.after
        ));
    }
    output.push_str(&format!(
        "changed_criteria: {}\n",
        diff.changed_criteria.len()
    ));
    for change in diff.changed_criteria.iter().take(10) {
        output.push_str(&format!(
            "  ~ {} :: {} -> {}\n",
            change.id, change.before, change.after
        ));
    }
    output.push_str(&format!(
        "changed_availability: {}\n",
        diff.changed_availability.len()
    ));
    for change in diff.changed_availability.iter().take(10) {
        output.push_str(&format!(
            "  ~ {} :: present {} -> {}\n",
            change.id, change.before_present, change.after_present
        ));
    }
    output
}

fn rtm_row_for_requirement(
    catalog: &Catalog,
    requirements: &[Requirement],
    requirement: &Requirement,
) -> RtmRow {
    let feature = requirement
        .feature_id
        .as_ref()
        .map(|id| id.as_str().to_string())
        .unwrap_or_else(|| "-".to_string());
    let function = requirement
        .function_id
        .as_ref()
        .map(|id| id.as_str().to_string())
        .unwrap_or_else(|| "-".to_string());
    let journey = if requirement.kind == RequirementClass::Functional {
        requirement
            .requirement_verifications
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    } else {
        requirement
            .journey_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string())
    };
    let evidence = trace_evidence(catalog, requirements, requirement);
    RtmRow {
        requirement_id: requirement.id.0.clone(),
        kind: requirement.kind,
        feature,
        function,
        journey,
        evidence,
    }
}

fn trace_evidence(
    catalog: &Catalog,
    requirements: &[Requirement],
    requirement: &Requirement,
) -> String {
    if requirement.id.0.contains("/SYS-OVR/") {
        if let Some(trace) = overlay_resolvable_trace(requirement) {
            return trace.to_string();
        }
    }
    if requirement.kind == RequirementClass::Performance {
        if let Some(capture) = requirement.rationale.as_deref() {
            if evidence_resolves(capture) {
                return capture.to_string();
            }
        }
    }
    if !requirement.subrequirements.is_empty() {
        for child_id in &requirement.subrequirements {
            if let Some(child) = requirements.iter().find(|req| &req.id == child_id) {
                let child_evidence = trace_evidence(catalog, requirements, child);
                if evidence_resolves(&child_evidence) {
                    return child_evidence;
                }
            }
        }
    }
    if let Some(journey_id) = requirement.journey_id {
        let evidence = journey_resolvable_evidence(catalog, journey_id);
        if evidence_resolves(&evidence) {
            return evidence;
        }
    }
    for journey_id in &requirement.requirement_verifications {
        let evidence = journey_resolvable_evidence(catalog, *journey_id);
        if evidence_resolves(&evidence) {
            return evidence;
        }
    }
    match &requirement.statement {
        RequirementStatement::Unknown { reason } if !reason.is_empty() => {
            format!("unknown_statement:{reason}")
        }
        _ => String::new(),
    }
}

fn overlay_resolvable_trace(requirement: &Requirement) -> Option<&'static str> {
    let entry = SYSTEM_OVERLAY_ENTRIES
        .iter()
        .find(|entry| requirement.id.0.ends_with(&format!("/{}", entry.suffix)))?;
    entry
        .traces
        .iter()
        .copied()
        .find(|trace| evidence_resolves(trace))
}

fn journey_resolvable_evidence(catalog: &Catalog, journey_id: crate::id::JourneyId) -> String {
    let Some(journey) = catalog.journey_by_id(&journey_id) else {
        return String::new();
    };
    let mut candidates = Vec::new();
    push_grounded_label(&journey.summary, &mut candidates);
    if let GroundedSet::Known { items } = &journey.steps {
        for step in items.iter() {
            candidates.push(provenance_label(&step.provenance));
            if let Some(route) = &step.value.route {
                push_grounded_label(route, &mut candidates);
            }
            if let Some(outcome) = &step.value.outcome {
                push_grounded_label(outcome, &mut candidates);
            }
        }
    }
    candidates
        .into_iter()
        .find(|candidate| evidence_resolves(candidate))
        .unwrap_or_default()
}

fn push_grounded_label<T>(grounded: &Grounded<T>, candidates: &mut Vec<String>) {
    if let Grounded::Known { provenance, .. } = grounded {
        candidates.push(provenance_label(provenance));
    }
}

fn provenance_label(provenance: &Provenance) -> String {
    match provenance {
        Provenance::Source(evidence) => format!("{}:{}", evidence.file, evidence.line),
        Provenance::Doc { file, line, .. } => format!("{file}:{line}"),
        Provenance::Runtime { capture, .. } => capture.to_string(),
        Provenance::Asserted { .. } => String::new(),
    }
}

fn rtm_evidence_is_complete(requirement: &Requirement, evidence: &str) -> bool {
    if evidence_resolves(evidence) {
        return true;
    }
    matches!(
        (&requirement.statement, evidence.strip_prefix("unknown_statement:")),
        (RequirementStatement::Unknown { reason }, Some(got)) if !reason.is_empty() && got == reason
    )
}

fn evidence_resolves(evidence: &str) -> bool {
    let evidence = evidence.trim();
    if evidence.is_empty()
        || evidence.starts_with("statement:")
        || evidence.starts_with("feature:")
        || evidence.starts_with("overlay:")
        || evidence.starts_with("asserted:")
        || evidence.starts_with("unknown_statement:")
    {
        return false;
    }
    if let Some((file, key)) = evidence.split_once('#') {
        if file.starts_with("runtime-captures/") {
            return capture_resolves(file, key);
        }
    }
    if let Some((file, line)) = split_file_line(evidence) {
        return file_line_resolves(file, line);
    }
    false
}

fn split_file_line(evidence: &str) -> Option<(&str, u32)> {
    let (file, line) = evidence.rsplit_once(':')?;
    let line = line.parse().ok()?;
    if file.is_empty() || line == 0 {
        return None;
    }
    Some((file, line))
}

fn file_line_resolves(file: &str, line: u32) -> bool {
    let repo = crate::provenance_walk::repo_root();
    let repo_path = repo.join(file);
    if repo_path.is_file() {
        return line_in_range(&repo_path, line);
    }
    if file.starts_with("content/") {
        let docs = crate::provenance_walk::docs_root();
        if !docs.is_dir() {
            return true;
        }
        let docs_path = docs.join(file);
        return docs_path.is_file() && line_in_range(&docs_path, line);
    }
    false
}

fn line_in_range(path: &Path, line: u32) -> bool {
    fs::read_to_string(path)
        .ok()
        .map(|content| line as usize <= content.lines().count())
        .unwrap_or(false)
}

fn capture_resolves(file: &str, key: &str) -> bool {
    let path = crate::provenance_walk::catalog_dir().join(file);
    let Ok(text) = fs::read_to_string(&path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    value.as_object().is_some_and(|map| map.contains_key(key))
}

fn statement_key(statement: &RequirementStatement) -> String {
    match statement {
        RequirementStatement::Known { text } => format!("known:{text}"),
        RequirementStatement::Unknown { reason } => format!("unknown:{reason}"),
    }
}

fn criteria_key(criteria: &RequirementCriteria) -> String {
    match criteria {
        RequirementCriteria::Known { items } => {
            let parts: Vec<String> = items.iter().map(|item| item.text.clone()).collect();
            format!("known:{}", parts.join("|"))
        }
        RequirementCriteria::Unknown { reason } => format!("unknown:{reason}"),
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::catalog::Catalog;
    use crate::requirement::{RequirementCatalog, RequirementStatement};

    const FILTERED_COUNT_1_0_0: usize = 239;
    const FILTERED_COUNT_1_4_0: usize = 350;
    const UNKNOWN_STATEMENTS_1_4_DEV: usize = 16;
    const UNKNOWN_CRITERIA_1_4_DEV: usize = 50;
    const UNKNOWN_STATEMENTS_UNFILTERED: usize = 62;

    #[test]
    fn version_filter_counts_match_pins() {
        let catalog = RequirementCatalog::bootstrap();
        assert_eq!(
            catalog.filter_by_version("1.0.0").requirements().len(),
            FILTERED_COUNT_1_0_0
        );
        assert_eq!(
            catalog.filter_by_version("1.4.0").requirements().len(),
            FILTERED_COUNT_1_4_0
        );
    }

    /// Pins the escape hatch shut: downgrading a statement to `Unknown` to dodge a missing RTM
    /// trace changes these counts even though every per-kind count stays put.
    #[test]
    fn unknown_counts_match_pins() {
        let requirements = RequirementCatalog::bootstrap();
        let unfiltered = requirements
            .requirements()
            .iter()
            .filter(|req| matches!(req.statement, RequirementStatement::Unknown { .. }))
            .count();
        assert_eq!(unfiltered, UNKNOWN_STATEMENTS_UNFILTERED);
        let json = crate::requirements_report::requirements_json(&requirements, "1.4-dev");
        assert_eq!(json.unknown_statement_count, UNKNOWN_STATEMENTS_1_4_DEV);
        assert_eq!(json.unknown_criteria_count, UNKNOWN_CRITERIA_1_4_DEV);
    }

    #[test]
    fn bootstrap_rtm_is_complete() {
        let catalog = Catalog::bootstrap();
        let requirements = RequirementCatalog::from_catalog(&catalog);
        crate::requirements_report::validate_rtm_completeness(&catalog, &requirements)
            .expect("bootstrap RTM complete");
    }

    #[test]
    fn performance_rtm_rows_use_slo_capture_keys() {
        let catalog = Catalog::bootstrap();
        let requirements = RequirementCatalog::from_catalog(&catalog);
        let rows = crate::requirements_report::build_rtm_rows(&catalog, &requirements);
        let performance: Vec<_> = rows
            .iter()
            .filter(|row| matches!(row.kind, crate::requirement::RequirementClass::Performance))
            .collect();
        assert_eq!(performance.len(), 80);
        assert!(
            performance.iter().all(|row| {
                row.evidence.starts_with("runtime-captures/") && row.evidence.contains('#')
            }),
            "performance evidence must be a capture key, got {:?}",
            performance.first().map(|row| &row.evidence)
        );
    }

    #[test]
    fn persisted_baseline_diff_detects_statement_not_availability() {
        let current = crate::requirements_report::requirements_json(
            &RequirementCatalog::bootstrap(),
            "1.4-dev",
        );
        let baseline = crate::requirements_report::load_requirements_baseline("1.4-dev")
            .expect("committed 1.4-dev baseline");
        let identical = crate::requirements_report::diff_requirement_reports(&baseline, &current);
        assert!(identical.added.is_empty(), "{:?}", identical.added);
        assert!(identical.removed.is_empty(), "{:?}", identical.removed);
        assert!(identical.changed_statement.is_empty());
        assert!(
            identical.changed_criteria.is_empty(),
            "{:?}",
            identical.changed_criteria
        );
        assert!(identical.changed_availability.is_empty());

        let mut statement_changed = current.clone();
        let idx = statement_changed
            .requirements
            .iter()
            .position(|row| matches!(row.statement, RequirementStatement::Known { .. }))
            .expect("known statement");
        statement_changed.requirements[idx].statement = RequirementStatement::Known {
            text: "mutated statement for persisted diff probe".to_string(),
        };
        let statement_diff =
            crate::requirements_report::diff_requirement_reports(&baseline, &statement_changed);
        assert_eq!(statement_diff.changed_statement.len(), 1);
        assert!(statement_diff.changed_availability.is_empty());
        assert!(statement_diff
            .changed_statement
            .iter()
            .any(|change| change.after.contains("mutated statement")));

        let mut availability_changed = current;
        let avail_idx = availability_changed
            .requirements
            .iter()
            .position(|row| row.availability_present)
            .expect("present requirement");
        availability_changed.requirements[avail_idx].availability_present = false;
        let availability_diff =
            crate::requirements_report::diff_requirement_reports(&baseline, &availability_changed);
        assert!(availability_diff.changed_statement.is_empty());
        assert_eq!(availability_diff.changed_availability.len(), 1);
    }
}
