use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::catalog::Catalog;
use crate::id::JourneyId;
use crate::journey::{derive_automatable, Actor, Automatable, UserJourney};
use crate::provenance::{Grounded, GroundedSet};
use crate::ui::ui_plan;

pub const HARD_EXCLUDED: &[JourneyId] = &[JourneyId::Deploy, JourneyId::ShutdownOnboardComputer];

/// Page-load may satisfy the UI cell only for journeys whose summary is "open/view this page".
pub const PAGE_LOAD_UI: &[JourneyId] = &[
    JourneyId::BrowseAvailableWebServices,
    JourneyId::ViewConfiguredSerialBridges,
    JourneyId::ViewSystemInformation,
];

const COMPASS_FINDING: &str = "F-068";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Empty,
    Planned,
    Skip,
    Pass,
    PassWithFinding,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub state: CellState,
    pub reason: Option<String>,
    pub dut: Option<String>,
    pub report_path: Option<String>,
    pub utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JourneyMatrixRow {
    pub journey_id: JourneyId,
    pub present_on_1_4_dev: bool,
    pub automatable: Automatable,
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
    pub state: CellState,
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
    journeys: Vec<RawJourney>,
}

#[derive(Deserialize)]
struct RawDut {
    #[serde(default)]
    tag: String,
}

#[derive(Deserialize)]
struct RawJourney {
    id: String,
    result: String,
    skip_reason: Option<String>,
}

impl CellState {
    fn rank(self) -> u8 {
        match self {
            Self::Empty => 0,
            Self::Planned | Self::Skip => 1,
            Self::Pass => 3,
            Self::PassWithFinding => 4,
            Self::Fail => 5,
        }
    }
}

impl Cell {
    fn empty() -> Self {
        Self {
            state: CellState::Empty,
            reason: None,
            dut: None,
            report_path: None,
            utc: None,
        }
    }

    fn planned() -> Self {
        Self {
            state: CellState::Planned,
            reason: None,
            dut: None,
            report_path: None,
            utc: None,
        }
    }

    fn skip(reason: &str) -> Self {
        Self {
            state: CellState::Skip,
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
        let incoming_rank = incoming.state.rank();
        let current_rank = self.state.rank();
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

pub fn has_known_route(journey: &UserJourney) -> bool {
    let GroundedSet::Known { items: steps } = &journey.steps else {
        return false;
    };
    steps
        .iter()
        .any(|step| matches!(&step.value.route, Some(Grounded::Known { .. })))
}

pub fn has_frontend_step(journey: &UserJourney) -> bool {
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
    if parsed.journeys.is_empty() {
        return Ok(());
    }
    let suite_is_ui = parsed.suite == "ui";
    let dut = dut_label(&parsed.base, parsed.dut.as_ref());
    let utc = if parsed.finished_at.is_empty() {
        None
    } else {
        Some(parsed.finished_at)
    };
    for journey in parsed.journeys {
        let Some(journey_id) = JourneyId::from_str_id(&journey.id) else {
            continue;
        };
        let mut state = match journey.result.as_str() {
            "pass" => CellState::Pass,
            "fail" => CellState::Fail,
            "skip" => CellState::Skip,
            _ => continue,
        };
        let mut reason = journey.skip_reason;
        if suite_is_ui && journey_id == JourneyId::CalibrateCompass && state == CellState::Pass {
            state = CellState::PassWithFinding;
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
    let mut rows: Vec<JourneyMatrixRow> = catalog
        .journeys()
        .iter()
        .map(|journey| catalog_row(journey))
        .collect();
    rows.sort_by_key(|row| row.journey_id.as_str());

    for hit in hits {
        let Some(row) = rows.iter_mut().find(|row| row.journey_id == hit.journey_id) else {
            continue;
        };
        let incoming = Cell {
            state: hit.state,
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
        if !row.present_on_1_4_dev
            && matches!(
                hit.state,
                CellState::Pass | CellState::PassWithFinding | CellState::Fail
            )
        {
            row.presence_contradiction = true;
        }
    }

    JourneyMatrix { rows }
}

fn catalog_row(journey: &UserJourney) -> JourneyMatrixRow {
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
                && row.backend.state == CellState::Empty
                && row.ui.state == CellState::Empty
        })
        .map(|row| row.journey_id)
        .collect()
}

pub fn format_matrix(matrix: &JourneyMatrix) -> String {
    let mut out = String::new();
    let present = matrix.rows.iter().filter(|r| r.present_on_1_4_dev).count();
    let backend_pass = count_state(matrix, true, CellState::Pass)
        + count_state(matrix, true, CellState::PassWithFinding);
    let ui_pass = count_state(matrix, false, CellState::Pass)
        + count_state(matrix, false, CellState::PassWithFinding);
    let ui_empty: Vec<_> = matrix
        .rows
        .iter()
        .filter(|r| r.present_on_1_4_dev && r.ui.state == CellState::Empty)
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

fn count_state(matrix: &JourneyMatrix, backend: bool, state: CellState) -> usize {
    matrix
        .rows
        .iter()
        .filter(|row| {
            let cell = if backend { &row.backend } else { &row.ui };
            cell.state == state
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
    let mut label = match cell.state {
        CellState::Empty => "empty".to_string(),
        CellState::Planned => "planned".to_string(),
        CellState::Skip => match &cell.reason {
            Some(reason) => format!("skip:{reason}"),
            None => "skip".to_string(),
        },
        CellState::Pass => "pass".to_string(),
        CellState::PassWithFinding => match &cell.reason {
            Some(reason) => format!("pass+{reason}"),
            None => "pass+finding".to_string(),
        },
        CellState::Fail => "fail".to_string(),
    };
    if let Some(dut) = &cell.dut {
        if matches!(
            cell.state,
            CellState::Pass | CellState::PassWithFinding | CellState::Fail
        ) {
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
        assert_eq!(derive_automatable(journey), Automatable::Hardware);
        assert!(has_known_route(journey));
        let matrix = build_journey_matrix(&catalog, &[]);
        let row = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::CreateSerialToUdpBridge)
            .unwrap();
        assert_eq!(row.backend.state, CellState::Planned);
        assert_eq!(row.automatable, Automatable::Hardware);
    }

    #[test]
    fn page_load_ui_is_only_view_journeys() {
        assert_eq!(PAGE_LOAD_UI.len(), 3);
        let catalog = Catalog::bootstrap();
        let matrix = build_journey_matrix(&catalog, &[]);
        for row in &matrix.rows {
            if PAGE_LOAD_UI.contains(&row.journey_id) && row.present_on_1_4_dev {
                assert_eq!(row.ui.state, CellState::Planned);
            }
        }
        let apply = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::ApplyParameterFile)
            .unwrap();
        assert!(apply.has_frontend_step);
        assert!(!apply.has_ui_plan);
        assert_eq!(apply.ui.state, CellState::Empty);
    }

    #[test]
    fn merge_ui_report_marks_compass_finding_and_level_horizon_contradiction() {
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
              "journeys": [
                {"id": "calibrate_compass", "result": "pass"},
                {"id": "level_horizon", "result": "pass"}
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
        assert_eq!(compass.ui.state, CellState::PassWithFinding);
        assert_eq!(compass.ui.reason.as_deref(), Some(COMPASS_FINDING));
        assert_eq!(compass.ui.dut.as_deref(), Some("192.168.0.177"));
        let level = matrix
            .rows
            .iter()
            .find(|row| row.journey_id == JourneyId::LevelHorizon)
            .unwrap();
        assert!(!level.present_on_1_4_dev);
        assert!(level.presence_contradiction);
        assert_eq!(level.ui.state, CellState::Pass);
        let _ = fs::remove_file(&path);
    }
}
