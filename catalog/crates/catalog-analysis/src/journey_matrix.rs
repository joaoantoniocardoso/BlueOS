use std::fs;
use std::path::Path;

use serde::Deserialize;

use catalog_core::catalog::Catalog;
use catalog_harness::runner::Verdict;
use catalog_harness::ui::{ui_plan, ui_typed_skip_reason};
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::provenance::{Grounded, GroundedSet};
use catalog_model::journey::{derive_automatable, Actor, UseCase, VerificationMethod};

pub const HARD_EXCLUDED: &[JourneyId] = &[JourneyId::Deploy, JourneyId::ShutdownOnboardComputer];

const BACKEND_TYPED_SKIP: &[(JourneyId, &str)] = &[
    (JourneyId::CreateSerialToUdpBridge, "usb_serial_device"),
    (JourneyId::EnablePing1dRangefinderMavlink, "no_sonar"),
];

fn backend_typed_skip_reason(id: JourneyId) -> Option<&'static str> {
    BACKEND_TYPED_SKIP
        .iter()
        .find(|(journey_id, _)| *journey_id == id)
        .map(|(_, reason)| *reason)
}

/// Page-load may satisfy the UI cell only for journeys whose summary is "open/view this page".
pub const PAGE_LOAD_UI: &[JourneyId] = &[
    JourneyId::BrowseAvailableWebServices,
    JourneyId::ViewConfiguredSerialBridges,
    JourneyId::ViewSystemInformation,
];

const COMPASS_FINDING: &str = "F-068";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub state: Verdict,
    pub reason: Option<String>,
    pub dut: Option<String>,
    pub report_path: Option<String>,
    pub utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JourneyMatrixRow {
    pub journey_id: JourneyId,
    pub present_on_1_4_dev: bool,
    pub automatable: VerificationMethod,
    pub has_known_route: bool,
    pub has_frontend_step: bool,
    pub has_ui_plan: bool,
    pub page_load_ui: bool,
    pub presence_contradiction: bool,
    pub backend: Cell,
    pub ui: Cell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JourneyMatrix {
    pub rows: Vec<JourneyMatrixRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportHit {
    pub journey_id: JourneyId,
    pub suite_is_ui: bool,
    pub state: Verdict,
    pub reason: Option<String>,
    pub dut: Option<String>,
    pub report_path: String,
    pub utc: Option<String>,
}

#[derive(Deserialize)]
struct RawReport {
    #[serde(default)]
    suite: String,
    #[serde(default)]
    base: String,
    dut: Option<RawDut>,
    #[serde(default)]
    finished_at: String,
    #[serde(default)]
    journeys: Vec<RawVerification>,
    #[serde(default)]
    verifications: Vec<RawVerification>,
}

#[derive(Deserialize)]
struct RawDut {
    #[serde(default)]
    tag: String,
}

#[derive(Deserialize)]
struct RawVerification {
    #[serde(default)]
    id: String,
    #[serde(default)]
    use_case: String,
    #[serde(default)]
    result: String,
    #[serde(default)]
    verdict: String,
    skip_reason: Option<String>,
}

impl RawVerification {
    fn journey_key(&self) -> &str {
        if !self.use_case.is_empty() {
            &self.use_case
        } else {
            &self.id
        }
    }

    fn verdict_key(&self) -> &str {
        if !self.verdict.is_empty() {
            &self.verdict
        } else {
            &self.result
        }
    }
}

impl Cell {
    fn empty() -> Self {
        Self {
            state: Verdict::Error,
            reason: None,
            dut: None,
            report_path: None,
            utc: None,
        }
    }

    fn planned() -> Self {
        Self {
            state: Verdict::Inconclusive(String::new()),
            reason: None,
            dut: None,
            report_path: None,
            utc: None,
        }
    }

    fn skip(reason: &str) -> Self {
        Self {
            state: Verdict::Inconclusive(String::new()),
            reason: Some(reason.to_string()),
            dut: None,
            report_path: None,
            utc: None,
        }
    }

    fn overlay(&mut self, incoming: Cell) {
        if self.reason.as_deref() == Some("hard_exclude") {
            return;
        }
        let incoming_rank = incoming.state.matrix_rank(incoming.reason.as_deref());
        let current_rank = self.state.matrix_rank(self.reason.as_deref());
        if incoming_rank > current_rank {
            *self = incoming;
            return;
        }
        if incoming_rank == current_rank && incoming_rank >= 3 {
            match (&incoming.utc, &self.utc) {
                (Some(new), Some(old)) if new >= old => *self = incoming,
                (Some(_), None) => *self = incoming,
                _ => {}
            }
            return;
        }
        if incoming_rank == current_rank
            && incoming_rank == 1
            && incoming.report_path.is_some()
            && self.report_path.is_none()
        {
            *self = incoming;
        }
    }
}

pub fn has_known_route(journey: &UseCase) -> bool {
    let GroundedSet::Known { items: steps } = &journey.steps else {
        return false;
    };
    steps
        .iter()
        .any(|step| matches!(&step.value.route, Some(Grounded::Known { .. })))
}

pub fn has_frontend_step(journey: &UseCase) -> bool {
    let GroundedSet::Known { items: steps } = &journey.steps else {
        return false;
    };
    steps
        .iter()
        .any(|step| matches!(step.value.actor, Actor::Frontend(_)))
}

fn is_hard_excluded(id: JourneyId) -> bool {
    HARD_EXCLUDED.contains(&id)
}

pub fn load_report_hits(paths: &[impl AsRef<Path>]) -> Result<Vec<ReportHit>, String> {
    let mut hits = Vec::new();
    for path in paths {
        collect_hits(path.as_ref(), &mut hits)?;
    }
    Ok(hits)
}

fn collect_hits(path: &Path, hits: &mut Vec<ReportHit>) -> Result<(), String> {
    if path.is_dir() {
        let entries =
            fs::read_dir(path).map_err(|err| format!("read {}: {err}", path.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("read {}: {err}", path.display()))?;
            collect_hits(&entry.path(), hits)?;
        }
        return Ok(());
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
        return Ok(());
    }
    let body = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let parsed: RawReport = match serde_json::from_str(&body) {
        Ok(parsed) => parsed,
        Err(_) => return Ok(()),
    };
    if parsed.journeys.is_empty() && parsed.verifications.is_empty() {
        return Ok(());
    }
    let entries = if parsed.verifications.is_empty() {
        &parsed.journeys
    } else {
        &parsed.verifications
    };
    let suite_is_ui = parsed.suite == "ui" || parsed.suite == "page_load";
    let dut = dut_label(&parsed.base, parsed.dut.as_ref());
    let utc = if parsed.finished_at.is_empty() {
        None
    } else {
        Some(parsed.finished_at)
    };
    for journey in entries {
        let journey_key = journey.journey_key();
        if journey_key.is_empty() {
            continue;
        }
        let Some(journey_id) = JourneyId::from_str_id(journey_key) else {
            continue;
        };
        let Some(state) = Verdict::from_report_str(journey.verdict_key()) else {
            continue;
        };
        let mut reason = journey.skip_reason.clone();
        if suite_is_ui && journey_id == JourneyId::CalibrateCompass && state == Verdict::Pass {
            reason = Some(COMPASS_FINDING.to_string());
        }
        hits.push(ReportHit {
            journey_id,
            suite_is_ui,
            state,
            reason,
            dut: dut.clone(),
            report_path: path.display().to_string(),
            utc: utc.clone(),
        });
    }
    Ok(())
}

fn dut_label(base: &str, dut: Option<&RawDut>) -> Option<String> {
    let host = base
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split(['/', ':'])
        .next()
        .unwrap_or("")
        .to_string();
    if !host.is_empty() {
        return Some(host);
    }
    dut.and_then(|d| {
        if d.tag.is_empty() {
            None
        } else {
            Some(d.tag.clone())
        }
    })
}

pub fn build_journey_matrix(catalog: &Catalog, hits: &[ReportHit]) -> JourneyMatrix {
    let mut rows: Vec<JourneyMatrixRow> = catalog.journeys().iter().map(catalog_row).collect();
    rows.sort_by_key(|row| row.journey_id.as_str());

    for hit in hits {
        let Some(row) = rows.iter_mut().find(|row| row.journey_id == hit.journey_id) else {
            continue;
        };
        let incoming = Cell {
            state: hit.state.clone(),
            reason: hit.reason.clone(),
            dut: hit.dut.clone(),
            report_path: Some(hit.report_path.clone()),
            utc: hit.utc.clone(),
        };
        if hit.suite_is_ui {
            row.ui.overlay(incoming);
        } else {
            row.backend.overlay(incoming);
        }
        if !row.present_on_1_4_dev && matches!(hit.state, Verdict::Pass | Verdict::Fail(_)) {
            row.presence_contradiction = true;
        }
    }

    JourneyMatrix { rows }
}

fn catalog_row(journey: &UseCase) -> JourneyMatrixRow {
    let present = journey.availability.present_on_1_4_dev;
    let excluded = is_hard_excluded(journey.id);
    let route = has_known_route(journey);
    let frontend = has_frontend_step(journey);
    let plan = ui_plan(journey.id).is_some();
    let page_load = PAGE_LOAD_UI.contains(&journey.id);
    let operator_ui = matches!(&journey.steps, GroundedSet::Known { items: steps } if steps
        .iter()
        .any(|step| matches!(step.value.actor, Actor::Operator | Actor::Frontend(_))));

    let backend = if excluded {
        Cell::skip("hard_exclude")
    } else if !present {
        Cell::skip("not_on_1.4-dev")
    } else if let Some(reason) = backend_typed_skip_reason(journey.id) {
        Cell::skip(reason)
    } else if route {
        Cell::planned()
    } else {
        Cell::skip("no_route")
    };

    let ui = if excluded {
        Cell::skip("hard_exclude")
    } else if !present {
        Cell::skip("not_on_1.4-dev")
    } else if plan || page_load {
        Cell::planned()
    } else if let Some(reason) = ui_typed_skip_reason(journey.id) {
        Cell::skip(reason)
    } else if operator_ui {
        Cell::empty()
    } else {
        Cell::skip("no_operator_ui")
    };

    JourneyMatrixRow {
        journey_id: journey.id,
        present_on_1_4_dev: present,
        automatable: derive_automatable(journey),
        has_known_route: route,
        has_frontend_step: frontend,
        has_ui_plan: plan,
        page_load_ui: page_load,
        presence_contradiction: false,
        backend,
        ui,
    }
}

pub fn blank_both_violations(matrix: &JourneyMatrix) -> Vec<JourneyId> {
    matrix
        .rows
        .iter()
        .filter(|row| {
            row.present_on_1_4_dev
                && !is_hard_excluded(row.journey_id)
                && row.backend.state == Verdict::Error
                && row.ui.state == Verdict::Error
        })
        .map(|row| row.journey_id)
        .collect()
}

pub fn format_matrix(matrix: &JourneyMatrix) -> String {
    let mut out = String::new();
    let present = matrix.rows.iter().filter(|r| r.present_on_1_4_dev).count();
    let backend_pass = count_pass(matrix, true);
    let ui_pass = count_pass(matrix, false);
    let ui_empty: Vec<_> = matrix
        .rows
        .iter()
        .filter(|r| r.present_on_1_4_dev && r.ui.state == Verdict::Error)
        .map(|r| r.journey_id.as_str())
        .collect();
    let contradictions: Vec<_> = matrix
        .rows
        .iter()
        .filter(|r| r.presence_contradiction)
        .map(|r| r.journey_id.as_str())
        .collect();
    let blanks = blank_both_violations(matrix);

    out.push_str("BlueOS catalog — journey coverage matrix (1.4-dev)\n");
    out.push_str(
        "Oracle: per-step route/frontend detection. derive_automatable is a hint only.\n\n",
    );
    out.push_str(&format!(
        "Journeys: {} (present on 1.4-dev: {present})\n",
        matrix.rows.len()
    ));
    out.push_str(&format!(
        "Backend measured pass: {backend_pass}   UI measured pass: {ui_pass}\n"
    ));
    out.push_str(&format!(
        "UI empty (needs plan): {}   presence contradictions: {}\n",
        ui_empty.len(),
        contradictions.len()
    ));
    out.push_str(&format!("Both cells empty (gate): {}\n\n", blanks.len()));

    if !blanks.is_empty() {
        out.push_str("Both cells empty:\n");
        for id in &blanks {
            out.push_str(&format!("  {id}\n"));
        }
        out.push('\n');
    }
    if !contradictions.is_empty() {
        out.push_str("Presence map vs live report:\n");
        for id in &contradictions {
            out.push_str(&format!("  {id}\n"));
        }
        out.push('\n');
    }
    if !ui_empty.is_empty() {
        out.push_str("UI cell empty (operator/frontend, no ui_plan, not page-load):\n");
        for id in &ui_empty {
            out.push_str(&format!("  {id}\n"));
        }
        out.push('\n');
    }

    out.push_str(&format!(
        "{:<42} {:<4} {:<5} {:<3} {:<5} {:<22} {}\n",
        "id", "1.4", "route", "fe", "plan", "backend", "ui"
    ));
    for row in &matrix.rows {
        out.push_str(&format!(
            "{:<42} {:<4} {:<5} {:<3} {:<5} {:<22} {}\n",
            row.journey_id.as_str(),
            yn(row.present_on_1_4_dev),
            yn(row.has_known_route),
            yn(row.has_frontend_step),
            yn(row.has_ui_plan || row.page_load_ui),
            format_cell(&row.backend),
            format_cell(&row.ui)
        ));
    }
    out
}

fn count_pass(matrix: &JourneyMatrix, backend: bool) -> usize {
    matrix
        .rows
        .iter()
        .filter(|row| {
            let cell = if backend { &row.backend } else { &row.ui };
            matches!(cell.state, Verdict::Pass)
        })
        .count()
}

fn yn(value: bool) -> &'static str {
    if value {
        "y"
    } else {
        "."
    }
}

fn format_cell(cell: &Cell) -> String {
    let mut label = match &cell.state {
        Verdict::Error => "empty".to_string(),
        Verdict::Inconclusive(_) if cell.reason.is_none() => "planned".to_string(),
        Verdict::Inconclusive(_) => match &cell.reason {
            Some(reason) => format!("skip:{reason}"),
            None => "skip".to_string(),
        },
        Verdict::Pass if cell.reason.as_deref().is_some_and(|r| !r.is_empty()) => {
            match &cell.reason {
                Some(reason) => format!("pass+{reason}"),
                None => "pass+finding".to_string(),
            }
        }
        Verdict::Pass => "pass".to_string(),
        Verdict::Fail(_) => "fail".to_string(),
    };
    if let Some(dut) = &cell.dut {
        if matches!(cell.state, Verdict::Pass | Verdict::Fail(_)) {
            label.push('@');
            label.push_str(dut);
        }
    }
    label
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_journeys_have_no_blank_both_cells() {
        let catalog = Catalog::bootstrap();
        let matrix = build_journey_matrix(&catalog, &[]);
        assert!(
            blank_both_violations(&matrix).is_empty(),
            "present journeys with both cells empty: {:?}",
            blank_both_violations(&matrix)
        );
    }

    #[test]
    fn derive_automatable_is_not_the_backend_oracle() {
        let catalog = Catalog::bootstrap();
        let journey = catalog
            .journey_by_id(&JourneyId::CreateSerialToUdpBridge)
            .expect("create_serial_to_udp_bridge");
        assert_eq!(derive_automatable(journey), VerificationMethod::Demo);
        assert!(has_known_route(journey));
        let matrix = build_journey_matrix(&catalog, &[]);
        let row = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::CreateSerialToUdpBridge)
            .unwrap();
        assert!(matches!(row.backend.state, Verdict::Inconclusive(_)));
        assert_eq!(row.backend.reason.as_deref(), Some("usb_serial_device"));
        assert!(matches!(row.ui.state, Verdict::Inconclusive(_)));
        assert!(row.ui.reason.is_none());
        assert_eq!(row.automatable, VerificationMethod::Demo);
    }

    #[test]
    fn enable_ping1d_backend_is_typed_no_sonar_skip() {
        let catalog = Catalog::bootstrap();
        let matrix = build_journey_matrix(&catalog, &[]);
        let row = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::EnablePing1dRangefinderMavlink)
            .unwrap();
        assert!(matches!(row.backend.state, Verdict::Inconclusive(_)));
        assert_eq!(row.backend.reason.as_deref(), Some("no_sonar"));
        assert!(matches!(row.ui.state, Verdict::Inconclusive(_)));
        assert_eq!(row.ui.reason.as_deref(), Some("no_sonar"));
    }

    #[test]
    fn page_load_ui_is_only_view_journeys() {
        assert_eq!(PAGE_LOAD_UI.len(), 3);
        let catalog = Catalog::bootstrap();
        let matrix = build_journey_matrix(&catalog, &[]);
        for row in &matrix.rows {
            if PAGE_LOAD_UI.contains(&row.journey_id) && row.present_on_1_4_dev {
                assert!(matches!(row.ui.state, Verdict::Inconclusive(_)));
                assert!(row.ui.reason.is_none());
            }
        }
        let video = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::ConfigureVideoStream)
            .unwrap();
        assert!(video.has_frontend_step);
        assert!(video.has_ui_plan);
        assert!(matches!(video.ui.state, Verdict::Inconclusive(_)));
        assert!(video.ui.reason.is_none());
    }

    #[test]
    fn merge_ui_report_marks_compass_finding_and_level_horizon_present() {
        let dir = std::env::temp_dir().join("blueos-catalog-matrix-test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("ui.json");
        fs::write(
            &path,
            r#"{
              "schema_version": 1,
              "suite": "ui",
              "base": "http://192.168.0.177",
              "finished_at": "2026-08-14T03:18:29Z",
              "verifications": [
                {"use_case": "calibrate_compass", "verdict": "pass"},
                {"use_case": "level_horizon", "verdict": "pass"}
              ]
            }"#,
        )
        .unwrap();
        let hits = load_report_hits(&[path.as_path()]).unwrap();
        let catalog = Catalog::bootstrap();
        let matrix = build_journey_matrix(&catalog, &hits);
        let compass = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::CalibrateCompass)
            .unwrap();
        assert_eq!(compass.ui.state, Verdict::Pass);
        assert_eq!(compass.ui.reason.as_deref(), Some(COMPASS_FINDING));
        assert_eq!(compass.ui.dut.as_deref(), Some("192.168.0.177"));
        let level = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::LevelHorizon)
            .unwrap();
        assert!(level.present_on_1_4_dev);
        assert!(!level.presence_contradiction);
        assert_eq!(level.ui.state, Verdict::Pass);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn merge_legacy_report_id_and_result() {
        let dir = std::env::temp_dir().join("blueos-catalog-matrix-legacy-test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("legacy.json");
        fs::write(
            &path,
            r#"{
              "suite": "smoke",
              "base": "http://192.168.0.1",
              "finished_at": "2026-08-14T03:18:29Z",
              "journeys": [
                {"id": "browse_available_web_services", "result": "pass"}
              ]
            }"#,
        )
        .unwrap();
        let hits = load_report_hits(&[path.as_path()]).unwrap();
        let catalog = Catalog::bootstrap();
        let matrix = build_journey_matrix(&catalog, &hits);
        let row = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::BrowseAvailableWebServices)
            .unwrap();
        assert_eq!(row.backend.state, Verdict::Pass);
        let _ = fs::remove_file(&path);
    }
}
