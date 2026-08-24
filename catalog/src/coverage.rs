#[path = "coverage_mappings.rs"]
mod coverage_mappings;

use std::collections::{BTreeMap, HashMap};

pub use coverage_mappings::TRACKER_MAPPINGS;

use crate::catalog::Catalog;
use crate::id::JourneyId;
use crate::journey::{
    derive_automatable, journey_requirements, Precondition, UseCase, VerificationMethod,
};
use crate::journey_matrix::has_frontend_step;

/// Path to the release testing tracker CSV, relative to the `catalog` crate root.
pub const TRACKER_CSV_PATH: &str =
    "extras/BlueOS Release Testing Tracker - BlueOS 1.x.x [TEMPLATE].csv";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoverageKind {
    /// Mapped to one or more catalog journeys.
    Journey,
    /// BlueOS-owned but not yet modeled as a journey.
    Unmodeled,
    /// External GCS / vehicle flight / physical flash — not BlueOS core automation.
    External,
    /// Duplicate / junk / empty tracker rows.
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackerTaskRef {
    pub category: &'static str,
    pub task: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct TrackerMapping {
    pub task: TrackerTaskRef,
    pub kind: CoverageKind,
    pub journeys: &'static [JourneyId],
    pub note: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KindCounts {
    pub journey: usize,
    pub unmodeled: usize,
    pub external: usize,
    pub ignore: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AutomatableCounts {
    pub http: usize,
    pub frontend: usize,
    pub hardware: usize,
    pub external_gcs: usize,
    pub manual: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JourneyCoverage {
    pub journey_id: JourneyId,
    pub automatable: VerificationMethod,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedTaskCoverage {
    pub task: TrackerTaskRef,
    pub note: &'static str,
    pub journeys: Vec<JourneyCoverage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageReport {
    pub total_tasks: usize,
    pub by_kind: KindCounts,
    pub by_automatable: AutomatableCounts,
    pub unmapped: Vec<TrackerTaskRef>,
    pub unmodeled: Vec<TrackerTaskRef>,
    pub external: Vec<TrackerTaskRef>,
    pub journey_mapped: Vec<MappedTaskCoverage>,
    pub top_requirements: Vec<(String, usize)>,
}

pub const fn mapping(
    category: &'static str,
    task: &'static str,
    kind: CoverageKind,
    journeys: &'static [JourneyId],
    note: &'static str,
) -> TrackerMapping {
    TrackerMapping {
        task: TrackerTaskRef { category, task },
        kind,
        journeys,
        note,
    }
}

pub fn clean_tracker_task(raw: &str) -> String {
    let mut s = raw.trim();
    s = s.trim_start();
    let bytes = s.as_bytes();
    let mut end = 0;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end > 0 && end < bytes.len() && bytes[end] == b'.' {
        end += 1;
        while end < bytes.len() && bytes[end] == b' ' {
            end += 1;
        }
        s = &s[end..];
    }
    s.trim().to_string()
}

pub fn coverage_report(catalog: &Catalog) -> CoverageReport {
    let journey_index: HashMap<JourneyId, &crate::journey::UseCase> = catalog
        .journeys()
        .iter()
        .map(|journey| (journey.id, journey))
        .collect();

    let mut by_kind = KindCounts::default();
    let mut by_automatable = AutomatableCounts::default();
    let mut unmodeled = Vec::new();
    let mut external = Vec::new();
    let mut journey_mapped = Vec::new();
    let mut requirement_counts: BTreeMap<String, usize> = BTreeMap::new();

    for entry in TRACKER_MAPPINGS {
        match entry.kind {
            CoverageKind::Journey => by_kind.journey += 1,
            CoverageKind::Unmodeled => {
                by_kind.unmodeled += 1;
                unmodeled.push(entry.task);
            }
            CoverageKind::External => {
                by_kind.external += 1;
                external.push(entry.task);
            }
            CoverageKind::Ignore => by_kind.ignore += 1,
        }

        if entry.kind != CoverageKind::Journey {
            continue;
        }

        let mut journeys = Vec::new();
        for &journey_id in entry.journeys {
            if let Some(journey) = journey_index.get(&journey_id) {
                let automatable = derive_automatable(journey);
                bump_automatable(&mut by_automatable, journey, automatable);
                let requirements: Vec<String> = journey_requirements(journey)
                    .iter()
                    .map(|precondition| precondition_label(precondition))
                    .collect();
                for label in &requirements {
                    *requirement_counts.entry(label.clone()).or_default() += 1;
                }
                journeys.push(JourneyCoverage {
                    journey_id,
                    automatable,
                    requirements,
                });
            }
        }

        journey_mapped.push(MappedTaskCoverage {
            task: entry.task,
            note: entry.note,
            journeys,
        });
    }

    let total_tasks = TRACKER_MAPPINGS.len();
    let mut top_requirements: Vec<(String, usize)> = requirement_counts.into_iter().collect();
    top_requirements.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    CoverageReport {
        total_tasks,
        by_kind,
        by_automatable,
        unmapped: Vec::new(),
        unmodeled,
        external,
        journey_mapped,
        top_requirements,
    }
}

fn bump_automatable(counts: &mut AutomatableCounts, journey: &UseCase, method: VerificationMethod) {
    match method {
        VerificationMethod::Test => {
            if has_frontend_step(journey) {
                counts.frontend += 1;
            } else {
                counts.http += 1;
            }
        }
        VerificationMethod::Demo => counts.hardware += 1,
        VerificationMethod::Inspect => counts.manual += 1,
        VerificationMethod::Analyze => counts.external_gcs += 1,
    }
}

pub fn precondition_label(precondition: &Precondition) -> String {
    match precondition {
        Precondition::ServiceState { service, state } => {
            format!("ServiceState::{}({})", service.as_str(), state)
        }
        Precondition::Network(network) => format!("Network::{}", network.as_str()),
        Precondition::HardwarePresent(label) => format!("HardwarePresent::{label}"),
        Precondition::ConfigClean(path) => format!("ConfigClean::{}", path.0),
        Precondition::Other(label) => format!("Other::{label}"),
        Precondition::Hardware(hardware) => format!("Hardware::{}", hardware.as_str()),
        Precondition::Software(software) => format!("Software::{}", software.as_str()),
        Precondition::NetworkResource(resource) => {
            format!("NetworkResource::{}", resource.as_str())
        }
        Precondition::Data(data) => format!("Data::{}", data.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;
    use std::path::PathBuf;

    use crate::catalog::Catalog;
    use crate::id::JourneyId;
    use crate::journey::{
        BoardKind, DataAssumption, HardwareAssumption, NetworkResource, NetworkState, Precondition,
        SoftwareAssumption,
    };

    use super::*;

    #[test]
    fn precondition_label_matches_debug_for_inner_enums() {
        for network in [NetworkState::Online, NetworkState::Offline] {
            assert_eq!(network.as_str(), format!("{network:?}"));
            let pre = Precondition::Network(network);
            assert_eq!(precondition_label(&pre), format!("Network::{network:?}"));
        }

        for software in [
            SoftwareAssumption::PirateMode,
            SoftwareAssumption::AdvancedMode,
            SoftwareAssumption::DevMode,
            SoftwareAssumption::ConfirmDangerousOp,
        ] {
            assert_eq!(software.as_str(), format!("{software:?}"));
            let pre = Precondition::Software(software);
            assert_eq!(precondition_label(&pre), format!("Software::{software:?}"));
        }

        for resource in [
            NetworkResource::WifiRadioPresent,
            NetworkResource::KnownWifiNetwork,
            NetworkResource::HotspotCapable,
            NetworkResource::WiredEthernetPresent,
            NetworkResource::UsbOtgPresent,
        ] {
            assert_eq!(resource.as_str(), format!("{resource:?}"));
            let pre = Precondition::NetworkResource(resource);
            assert_eq!(
                precondition_label(&pre),
                format!("NetworkResource::{resource:?}")
            );
        }

        for data in [
            DataAssumption::ExtensionInstalled,
            DataAssumption::LocalBlueosVersionAvailable,
            DataAssumption::SerialBridgeConfigured,
            DataAssumption::NmeaSocketConfigured,
            DataAssumption::RecordingListed,
            DataAssumption::WifiNetworkSaved,
            DataAssumption::WifiCurrentlyConnected,
            DataAssumption::OnboardDhcpServerActive,
        ] {
            assert_eq!(data.as_str(), format!("{data:?}"));
            let pre = Precondition::Data(data);
            assert_eq!(precondition_label(&pre), format!("Data::{data:?}"));
        }

        for hardware in [
            HardwareAssumption::UsbCamera,
            HardwareAssumption::Ping1d,
            HardwareAssumption::Ping360,
            HardwareAssumption::ExternalNmeaGps,
            HardwareAssumption::UsbSerialDevice,
        ] {
            assert_eq!(hardware.as_str(), format!("{hardware:?}"));
            let pre = Precondition::Hardware(hardware);
            assert_eq!(precondition_label(&pre), format!("Hardware::{hardware:?}"));
        }

        for board in [
            BoardKind::Any,
            BoardKind::Navigator,
            BoardKind::Pixhawk,
            BoardKind::Sitl,
        ] {
            assert_eq!(board.as_str(), format!("{board:?}"));
            let hardware = HardwareAssumption::FlightController(board);
            assert_eq!(hardware.as_str(), format!("{hardware:?}"));
            let pre = Precondition::Hardware(hardware);
            assert_eq!(precondition_label(&pre), format!("Hardware::{hardware:?}"));
        }
    }

    #[test]
    fn tracker_mappings_task_refs_are_unique() {
        let mut seen = HashSet::new();
        for entry in TRACKER_MAPPINGS {
            let key = (entry.task.category, entry.task.task);
            assert!(seen.insert(key), "duplicate tracker mapping: {key:?}");
        }
    }

    #[test]
    fn every_journey_id_in_mappings_exists_in_catalog() {
        let catalog = Catalog::bootstrap();
        let known: HashSet<JourneyId> = catalog.journeys().iter().map(|j| j.id).collect();
        for entry in TRACKER_MAPPINGS {
            for &journey_id in entry.journeys {
                assert!(
                    known.contains(&journey_id),
                    "unknown journey {:?} in mapping {:?}",
                    journey_id,
                    entry.task
                );
            }
        }
    }

    #[test]
    fn coverage_report_runs_on_bootstrap_without_panic() {
        let catalog = Catalog::bootstrap();
        let _ = coverage_report(&catalog);
    }

    #[test]
    fn coverage_report_kind_counts_sum_to_total() {
        let catalog = Catalog::bootstrap();
        let report = coverage_report(&catalog);
        let sum = report.by_kind.journey
            + report.by_kind.unmodeled
            + report.by_kind.external
            + report.by_kind.ignore;
        assert_eq!(sum, report.total_tasks);
        assert_eq!(report.total_tasks, TRACKER_MAPPINGS.len());
    }

    #[test]
    fn every_csv_tracker_task_is_mapped() {
        let csv_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TRACKER_CSV_PATH);
        let raw = fs::read_to_string(csv_path).expect("tracker CSV should exist");
        let rows = parse_csv_records(&raw);
        let header = rows
            .iter()
            .find(|row| row.first().is_some_and(|c| c == "Category"));
        let header_idx = header
            .and_then(|row| rows.iter().position(|candidate| candidate == row))
            .expect("Category header row");

        let mapped: HashSet<(String, String)> = TRACKER_MAPPINGS
            .iter()
            .map(|entry| (entry.task.category.to_string(), entry.task.task.to_string()))
            .collect();

        let mut seen = HashSet::new();
        for row in &rows[header_idx + 1..] {
            let category = row.first().map(|s| s.trim()).unwrap_or("").to_string();
            let task_raw = row.get(1).map(String::as_str).unwrap_or("");
            if task_raw.trim().is_empty() {
                continue;
            }
            let task = clean_tracker_task(task_raw);
            let key = (category.clone(), task.clone());
            if !seen.insert(key.clone()) {
                continue;
            }
            assert!(
                mapped.contains(&key),
                "tracker task missing from TRACKER_MAPPINGS: category={category:?} task={task:?}"
            );
        }
    }

    fn parse_csv_records(content: &str) -> Vec<Vec<String>> {
        let mut rows = Vec::new();
        let mut row = Vec::new();
        let mut field = String::new();
        let mut in_quotes = false;

        let mut chars = content.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '"' if in_quotes => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        field.push('"');
                    } else {
                        in_quotes = false;
                    }
                }
                '"' => in_quotes = true,
                ',' if !in_quotes => row.push(std::mem::take(&mut field)),
                '\n' | '\r' if !in_quotes => {
                    if ch == '\r' && chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    row.push(std::mem::take(&mut field));
                    if row.iter().any(|cell| !cell.is_empty()) {
                        rows.push(row);
                    }
                    row = Vec::new();
                }
                _ => field.push(ch),
            }
        }

        if !field.is_empty() || !row.is_empty() {
            row.push(field);
            if row.iter().any(|cell| !cell.is_empty()) {
                rows.push(row);
            }
        }

        rows
    }
}
