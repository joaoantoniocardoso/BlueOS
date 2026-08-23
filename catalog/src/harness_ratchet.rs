use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::journey::{blast_radius_is_unknown, derive_oracle_class, BodyKind, OracleClass};
use crate::journey_matrix::PAGE_LOAD_UI;
use crate::mutating_smoke::MUTATING_SMOKE_ENTRIES;
use crate::observed_verification::{
    count_extractor_coverage_lapses, count_unverified_observed_evidence,
};
use crate::provenance::{Grounded, GroundedSet};
use crate::requirement::RequirementCatalog;
use crate::requirements_report::requirements_json;
use crate::ui::ui_plan;

pub const DEFAULT_BASELINE_PATH: &str = "extras/qa-harness-improve/ratchet_baseline.json";
pub const FAILURE_MODE_LEDGER_PATH: &str = "extras/qa-1.4-full/FAILURE_MODE_LEDGER.md";
const RATCHET_REQUIREMENTS_TAG: &str = "1.4-dev";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessRatchetCounts {
    pub unknown_blast_radius: usize,
    pub unknown_body_kind: usize,
    pub unprobed_failure_modes: usize,
    pub mutating_smoke_missing_effect_read: usize,
    pub client_orchestrated_missing_ui_plan: usize,
    pub open_harness_gap: usize,
    pub unverified_observed_evidence: usize,
    pub extractor_coverage_lapses: usize,
    pub unknown_requirement_statements: usize,
    pub unknown_requirement_criteria: usize,
    pub requirement_contamination_findings: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessRatchetRegression {
    pub field: &'static str,
    pub baseline: usize,
    pub current: usize,
}

pub fn harness_ratchet_counts(catalog: &Catalog) -> HarnessRatchetCounts {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let (
        unknown_requirement_statements,
        unknown_requirement_criteria,
        requirement_contamination_findings,
    ) = if catalog_supports_requirement_derivation(catalog) {
        requirement_coverage_counts(&RequirementCatalog::from_catalog(catalog))
    } else {
        (0, 0, 0)
    };
    HarnessRatchetCounts {
        unknown_blast_radius: catalog
            .journeys()
            .iter()
            .filter(|journey| blast_radius_is_unknown(journey))
            .count(),
        unknown_body_kind: count_unknown_body_kinds(catalog),
        unprobed_failure_modes: count_unprobed_failure_modes(FAILURE_MODE_LEDGER_PATH),
        mutating_smoke_missing_effect_read: MUTATING_SMOKE_ENTRIES
            .iter()
            .filter(|entry| entry.effect_read.is_none())
            .count(),
        client_orchestrated_missing_ui_plan: count_client_orchestrated_missing_ui_plan(catalog),
        open_harness_gap: count_open_harness_gap(),
        unverified_observed_evidence: count_unverified_observed_evidence(repo_root, catalog),
        extractor_coverage_lapses: count_extractor_coverage_lapses(repo_root, catalog),
        unknown_requirement_statements,
        unknown_requirement_criteria,
        requirement_contamination_findings,
    }
}

fn requirement_coverage_counts(requirements: &RequirementCatalog) -> (usize, usize, usize) {
    let report = requirements_json(requirements, RATCHET_REQUIREMENTS_TAG);
    (
        report.unknown_statement_count,
        report.unknown_criteria_count,
        report.contamination_findings,
    )
}

fn catalog_supports_requirement_derivation(catalog: &Catalog) -> bool {
    catalog.services().len() == crate::services::all_services().len()
        && catalog.journeys().len() == crate::journeys::all_journeys().len()
        && catalog.pages().len() == crate::pages::all_pages().len()
}

pub fn count_unknown_body_kinds(catalog: &Catalog) -> usize {
    let mut count = 0;
    for journey in catalog.journeys() {
        if let GroundedSet::Known { items } = &journey.steps {
            for step in items.iter() {
                if let Some(Grounded::Known { value: outcome, .. }) = &step.value.outcome {
                    if outcome.body_kind == BodyKind::Unknown {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

pub fn count_client_orchestrated_missing_ui_plan(catalog: &Catalog) -> usize {
    catalog
        .journeys()
        .iter()
        .filter(|journey| {
            derive_oracle_class(catalog, journey) == OracleClass::ClientOrchestrated
                && ui_plan(journey.id).is_none()
                && !PAGE_LOAD_UI.contains(&journey.id)
        })
        .count()
}

pub fn count_open_harness_gap() -> usize {
    MUTATING_SMOKE_ENTRIES
        .iter()
        .filter(|entry| entry.notes.contains("HarnessGap"))
        .count()
}

pub fn count_unprobed_failure_modes(ledger_path: &str) -> usize {
    parse_failure_mode_ledger(ledger_path)
        .into_iter()
        .filter(|row| row.status == "harness_gap")
        .count()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FailureModeLedgerRow {
    status: String,
}

fn parse_failure_mode_ledger(path: &str) -> Vec<FailureModeLedgerRow> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };
    let mut rows = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with('|') || line.contains("---|") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 6 {
            continue;
        }
        let service = cells[1];
        let mode_id = cells[2];
        if service == "service" || mode_id == "mode_id" {
            continue;
        }
        rows.push(FailureModeLedgerRow {
            status: cells[4].to_string(),
        });
    }
    rows
}

pub fn compare_harness_ratchet(
    baseline: HarnessRatchetCounts,
    current: HarnessRatchetCounts,
) -> Vec<HarnessRatchetRegression> {
    let fields = [
        (
            "unknown_blast_radius",
            baseline.unknown_blast_radius,
            current.unknown_blast_radius,
        ),
        (
            "unknown_body_kind",
            baseline.unknown_body_kind,
            current.unknown_body_kind,
        ),
        (
            "unprobed_failure_modes",
            baseline.unprobed_failure_modes,
            current.unprobed_failure_modes,
        ),
        (
            "mutating_smoke_missing_effect_read",
            baseline.mutating_smoke_missing_effect_read,
            current.mutating_smoke_missing_effect_read,
        ),
        (
            "client_orchestrated_missing_ui_plan",
            baseline.client_orchestrated_missing_ui_plan,
            current.client_orchestrated_missing_ui_plan,
        ),
        (
            "open_harness_gap",
            baseline.open_harness_gap,
            current.open_harness_gap,
        ),
        (
            "unverified_observed_evidence",
            baseline.unverified_observed_evidence,
            current.unverified_observed_evidence,
        ),
        (
            "extractor_coverage_lapses",
            baseline.extractor_coverage_lapses,
            current.extractor_coverage_lapses,
        ),
        (
            "unknown_requirement_statements",
            baseline.unknown_requirement_statements,
            current.unknown_requirement_statements,
        ),
        (
            "unknown_requirement_criteria",
            baseline.unknown_requirement_criteria,
            current.unknown_requirement_criteria,
        ),
        (
            "requirement_contamination_findings",
            baseline.requirement_contamination_findings,
            current.requirement_contamination_findings,
        ),
    ];
    fields
        .into_iter()
        .filter(|(_, base, cur)| *cur > *base)
        .map(|(field, baseline, current)| HarnessRatchetRegression {
            field,
            baseline,
            current,
        })
        .collect()
}

pub fn load_harness_ratchet_baseline(path: &Path) -> Result<HarnessRatchetCounts, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("read baseline {}: {err}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|err| format!("parse baseline {}: {err}", path.display()))
}

pub fn write_harness_ratchet_baseline(
    path: &Path,
    counts: HarnessRatchetCounts,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("create baseline dir {}: {err}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(&counts)
        .map_err(|err| format!("serialize baseline: {err}"))?;
    fs::write(path, format!("{content}\n"))
        .map_err(|err| format!("write baseline {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::requirement::RequirementStatement;

    #[test]
    fn failure_mode_ledger_parses_harness_gap_rows() {
        let rows = parse_failure_mode_ledger(FAILURE_MODE_LEDGER_PATH);
        assert!(
            rows.len() >= 100,
            "expected full ledger, got {}",
            rows.len()
        );
        let harness_gap = rows
            .iter()
            .filter(|row| row.status == "harness_gap")
            .count();
        assert!(harness_gap > 0);
        assert_eq!(
            harness_gap,
            count_unprobed_failure_modes(FAILURE_MODE_LEDGER_PATH)
        );
    }

    #[test]
    fn compare_allows_equal_and_improvement() {
        let baseline = HarnessRatchetCounts {
            unknown_blast_radius: 5,
            unknown_body_kind: 10,
            unprobed_failure_modes: 49,
            mutating_smoke_missing_effect_read: 60,
            client_orchestrated_missing_ui_plan: 17,
            open_harness_gap: 60,
            unverified_observed_evidence: 1129,
            extractor_coverage_lapses: 0,
            unknown_requirement_statements: 16,
            unknown_requirement_criteria: 50,
            requirement_contamination_findings: 10,
        };
        let equal = baseline;
        assert!(compare_harness_ratchet(baseline, equal).is_empty());

        let improved = HarnessRatchetCounts {
            unknown_body_kind: 9,
            ..baseline
        };
        assert!(compare_harness_ratchet(baseline, improved).is_empty());
    }

    #[test]
    fn compare_fails_on_worsening_only() {
        let baseline = HarnessRatchetCounts {
            unknown_blast_radius: 0,
            unknown_body_kind: 10,
            unprobed_failure_modes: 49,
            mutating_smoke_missing_effect_read: 60,
            client_orchestrated_missing_ui_plan: 17,
            open_harness_gap: 60,
            unverified_observed_evidence: 1129,
            extractor_coverage_lapses: 0,
            unknown_requirement_statements: 16,
            unknown_requirement_criteria: 50,
            requirement_contamination_findings: 10,
        };
        let worse = HarnessRatchetCounts {
            mutating_smoke_missing_effect_read: 61,
            ..baseline
        };
        let regressions = compare_harness_ratchet(baseline, worse);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].field, "mutating_smoke_missing_effect_read");
    }

    #[test]
    fn requirement_coverage_counts_match_measured_pins() {
        let catalog = Catalog::bootstrap();
        let current = harness_ratchet_counts(&catalog);
        assert_eq!(current.unknown_requirement_statements, 16);
        assert_eq!(current.unknown_requirement_criteria, 50);
        assert_eq!(current.requirement_contamination_findings, 10);
        assert!(catalog_supports_requirement_derivation(&catalog));
    }

    #[test]
    fn flipping_known_statement_to_unknown_raises_unknown_requirement_statements() {
        let catalog = Catalog::bootstrap();
        let baseline = harness_ratchet_counts(&catalog);
        let mut requirements = RequirementCatalog::from_catalog(&catalog);
        let requirement = requirements
            .requirements
            .iter_mut()
            .find(|req| {
                matches!(req.statement, RequirementStatement::Known { .. })
                    && req.availability.present_on_dut(RATCHET_REQUIREMENTS_TAG)
            })
            .expect("known statement present on ratchet tag");
        requirement.statement = RequirementStatement::Unknown {
            reason: "ratchet mutation probe".to_string(),
        };
        let (statements, criteria, contamination) = requirement_coverage_counts(&requirements);
        assert_eq!(statements, baseline.unknown_requirement_statements + 1);
        let mutated = HarnessRatchetCounts {
            unknown_requirement_statements: statements,
            unknown_requirement_criteria: criteria,
            requirement_contamination_findings: contamination,
            ..baseline
        };
        let regressions = compare_harness_ratchet(baseline, mutated);
        assert_eq!(regressions.len(), 1);
        assert_eq!(regressions[0].field, "unknown_requirement_statements");
    }

    #[test]
    fn compare_fails_on_each_requirement_coverage_field() {
        let catalog = Catalog::bootstrap();
        let baseline = harness_ratchet_counts(&catalog);
        let cases = [
            (
                "unknown_requirement_statements",
                HarnessRatchetCounts {
                    unknown_requirement_statements: baseline.unknown_requirement_statements + 1,
                    ..baseline
                },
            ),
            (
                "unknown_requirement_criteria",
                HarnessRatchetCounts {
                    unknown_requirement_criteria: baseline.unknown_requirement_criteria + 1,
                    ..baseline
                },
            ),
            (
                "requirement_contamination_findings",
                HarnessRatchetCounts {
                    requirement_contamination_findings: baseline.requirement_contamination_findings
                        + 1,
                    ..baseline
                },
            ),
        ];
        for (field, worse) in cases {
            let regressions = compare_harness_ratchet(baseline, worse);
            assert_eq!(regressions.len(), 1, "{field}");
            assert_eq!(regressions[0].field, field);
        }
    }
}
