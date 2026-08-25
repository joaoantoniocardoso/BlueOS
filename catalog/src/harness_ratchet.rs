use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use catalog_analysis::api_contract::api_coverage_counts;
use catalog_analysis::journey_matrix::PAGE_LOAD_UI;
use catalog_derive::requirement::RequirementCatalog;
use catalog_derive::requirements_report::requirements_json;
use catalog_harness::mutating_smoke::MUTATING_SMOKE_ENTRIES;
use catalog_harness::oracle::derive_oracle_class;
use catalog_harness::ui::{ui_plan, ui_typed_skip_reason};
use catalog_kernel::provenance::{Grounded, GroundedSet};
use catalog_model::journey::{blast_radius_is_unknown, BodyKind, OracleClass};
use catalog_provenance::observed_verification::{
    count_extractor_coverage_lapses, count_unverified_observed_evidence,
};

use crate::catalog::Catalog;

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
    pub unmapped_api_routes: usize,
    pub orphan_api_hits: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessRatchetRegression {
    pub field: &'static str,
    pub baseline: usize,
    pub current: usize,
}

impl HarnessRatchetCounts {
    pub fn named_fields(&self) -> [(&'static str, usize); 13] {
        [
            ("unknown_blast_radius", self.unknown_blast_radius),
            ("unknown_body_kind", self.unknown_body_kind),
            ("unprobed_failure_modes", self.unprobed_failure_modes),
            (
                "mutating_smoke_missing_effect_read",
                self.mutating_smoke_missing_effect_read,
            ),
            (
                "client_orchestrated_missing_ui_plan",
                self.client_orchestrated_missing_ui_plan,
            ),
            ("open_harness_gap", self.open_harness_gap),
            (
                "unverified_observed_evidence",
                self.unverified_observed_evidence,
            ),
            ("extractor_coverage_lapses", self.extractor_coverage_lapses),
            (
                "unknown_requirement_statements",
                self.unknown_requirement_statements,
            ),
            (
                "unknown_requirement_criteria",
                self.unknown_requirement_criteria,
            ),
            (
                "requirement_contamination_findings",
                self.requirement_contamination_findings,
            ),
            ("unmapped_api_routes", self.unmapped_api_routes),
            ("orphan_api_hits", self.orphan_api_hits),
        ]
    }
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
    let api_coverage = api_coverage_counts(repo_root, catalog);
    HarnessRatchetCounts {
        unknown_blast_radius: catalog
            .journeys()
            .iter()
            .filter(|journey| blast_radius_is_unknown(journey))
            .count(),
        unknown_body_kind: count_unknown_body_kinds(catalog),
        unprobed_failure_modes: count_unprobed_failure_modes(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(FAILURE_MODE_LEDGER_PATH),
        ),
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
        unmapped_api_routes: api_coverage
            .as_ref()
            .map_or(usize::MAX, |report| report.unmapped),
        orphan_api_hits: api_coverage
            .as_ref()
            .map_or(usize::MAX, |report| report.orphan_hits.len()),
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
    catalog.services().len() == catalog_data::all_services().len()
        && catalog.journeys().len() == catalog_data::all_journeys().len()
        && catalog.pages().len() == catalog_data::all_pages().len()
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
                && ui_typed_skip_reason(journey.id).is_none()
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

/// `usize::MAX` when the ledger cannot be read, so an unreadable ledger trips the ratchet instead
/// of counting zero gaps and inviting a snapshot that erases the dimension.
pub fn count_unprobed_failure_modes(ledger_path: &Path) -> usize {
    let Some(rows) = parse_failure_mode_ledger(ledger_path) else {
        return usize::MAX;
    };
    rows.into_iter()
        .filter(|row| row.status == "harness_gap")
        .count()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FailureModeLedgerRow {
    status: String,
}

fn parse_failure_mode_ledger(path: &Path) -> Option<Vec<FailureModeLedgerRow>> {
    let content = fs::read_to_string(path).ok()?;
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
    Some(rows)
}

pub fn compare_harness_ratchet(
    baseline: HarnessRatchetCounts,
    current: HarnessRatchetCounts,
) -> Vec<HarnessRatchetRegression> {
    baseline
        .named_fields()
        .into_iter()
        .zip(current.named_fields())
        .filter(|((_, base), (_, cur))| cur > base)
        .map(
            |((field, baseline), (_, current))| HarnessRatchetRegression {
                field,
                baseline,
                current,
            },
        )
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
    if let Some((field, _)) = counts
        .named_fields()
        .into_iter()
        .find(|(_, value)| *value == usize::MAX)
    {
        return Err(format!(
            "refusing to snapshot {field}=MAX (its input could not be read)"
        ));
    }
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
    use catalog_derive::requirement::RequirementStatement;

    fn sample_counts() -> HarnessRatchetCounts {
        HarnessRatchetCounts {
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
            unmapped_api_routes: 80,
            orphan_api_hits: 4,
        }
    }

    #[test]
    fn failure_mode_ledger_parses_harness_gap_rows() {
        let ledger = Path::new(env!("CARGO_MANIFEST_DIR")).join(FAILURE_MODE_LEDGER_PATH);
        let rows = parse_failure_mode_ledger(&ledger).expect("read ledger");
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
        assert_eq!(harness_gap, count_unprobed_failure_modes(&ledger));
        assert_eq!(
            count_unprobed_failure_modes(Path::new("no/such/ledger.md")),
            usize::MAX,
            "an unreadable ledger must not read as zero gaps"
        );
    }

    #[test]
    fn snapshot_refuses_a_sentinel_counter() {
        let counts = HarnessRatchetCounts {
            unprobed_failure_modes: usize::MAX,
            ..sample_counts()
        };
        let error = write_harness_ratchet_baseline(Path::new("/dev/null"), counts)
            .expect_err("sentinel must be refused");
        assert!(error.contains("unprobed_failure_modes"), "{error}");
    }

    #[test]
    fn compare_allows_equal_and_improvement() {
        let baseline = sample_counts();
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
        let baseline = sample_counts();
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
        assert_eq!(current.unknown_requirement_statements, 20);
        assert_eq!(current.unknown_requirement_criteria, 52);
        assert_eq!(current.requirement_contamination_findings, 13);
        assert!(catalog_supports_requirement_derivation(&catalog));
    }

    #[test]
    fn flipping_known_statement_to_unknown_raises_unknown_requirement_statements() {
        let catalog = Catalog::bootstrap();
        let baseline = harness_ratchet_counts(&catalog);
        let mut requirements = RequirementCatalog::from_catalog(&catalog);
        let requirement = requirements
            .requirements_mut()
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
