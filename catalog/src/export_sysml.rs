use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::capability::{capability_def, Aggregate};
use crate::catalog::Catalog;
use crate::domain::domain_of;
use crate::edge::{Bus, Connection};
use crate::function::{Action, ActionCatalog};
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::PortKind;
use crate::journey::{Actor, Precondition, UseCase};
use crate::provenance::{Asserted, AssertedSet, Evidence, GroundedSet, Observed, ObservedSet};
use crate::requirement::{Requirement, RequirementCatalog, RequirementClass, RequirementCriteria};
use crate::service::Service;
use crate::version::Availability;

pub const DEFAULT_SYSML_SUBSET_GOLDEN: &str = "goldens/sysml/catalog_subset.sysml";

#[derive(Debug, Clone, Default)]
pub struct SysmlExportFilter {
    pub services: BTreeSet<ServiceId>,
    pub journeys: BTreeSet<JourneyId>,
    pub actions: HashSet<String>,
    pub requirements: BTreeSet<String>,
    pub include_connection_endpoints: bool,
}

impl SysmlExportFilter {
    pub fn subset_golden() -> Self {
        let mut filter = Self {
            include_connection_endpoints: true,
            ..Self::default()
        };
        filter.services.insert(ServiceId::Ping);
        filter.journeys.insert(JourneyId::ConnectPingViewerToSonar);
        filter
            .actions
            .insert("connect_ping_viewer_to_sonar/cbf29ce484222325".to_string());
        filter.requirements.insert(
            "REQ/peripherals/sonar/FUN/connect_ping_viewer_to_sonar/cbf29ce484222325".to_string(),
        );
        filter
    }

    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
            && self.journeys.is_empty()
            && self.actions.is_empty()
            && self.requirements.is_empty()
    }
}

pub fn export_sysml(catalog: &Catalog, filter: Option<&SysmlExportFilter>) -> String {
    let requirements = RequirementCatalog::from_catalog(catalog);
    let actions = ActionCatalog::from_catalog(catalog);
    let mut writer = SysmlWriter::new();
    writer.emit_header();
    writer.line("package BlueOS_Catalog {");
    writer.indent_level();
    writer.emit_imports();
    writer.emit_local_metadata();
    writer.emit_local_requirement_defs();
    emit_domain_tree(catalog, &requirements, &actions, filter, &mut writer);
    emit_cross_connections(catalog, filter, &mut writer);
    writer.dedent();
    writer.line("}");
    writer.finish()
}

fn emit_domain_tree(
    catalog: &Catalog,
    requirements: &RequirementCatalog,
    actions: &ActionCatalog,
    filter: Option<&SysmlExportFilter>,
    writer: &mut SysmlWriter,
) {
    let selected_services = selected_services(catalog, filter);
    let selected_journeys = selected_journeys(catalog, filter);
    let selected_actions = selected_actions(actions, filter);
    let selected_requirements = selected_requirements(requirements, filter);
    let mut by_domain: BTreeMap<String, BTreeMap<String, DomainBucket>> = BTreeMap::new();

    for service in &selected_services {
        let aggregate = aggregate_for_service(catalog, service.id);
        by_domain
            .entry(domain_of(aggregate).as_str().to_string())
            .or_default()
            .entry(aggregate.as_str().to_string())
            .or_default()
            .services
            .push(service);
    }

    for journey in &selected_journeys {
        let aggregate = aggregate_for_journey(catalog, journey);
        by_domain
            .entry(domain_of(aggregate).as_str().to_string())
            .or_default()
            .entry(aggregate.as_str().to_string())
            .or_default()
            .journeys
            .push(journey);
    }

    for action in &selected_actions {
        let aggregate = aggregate_for_capability(action.capability);
        by_domain
            .entry(domain_of(aggregate).as_str().to_string())
            .or_default()
            .entry(aggregate.as_str().to_string())
            .or_default()
            .actions
            .push(action);
    }

    for requirement in &selected_requirements {
        by_domain
            .entry(requirement.domain.as_str().to_string())
            .or_default()
            .entry(requirement.aggregate.as_str().to_string())
            .or_default()
            .requirements
            .push(requirement);
    }

    for (domain, aggregates) in by_domain {
        writer.line(&format!("package {domain} {{"));
        writer.indent_level();
        for (aggregate, bucket) in aggregates {
            writer.line(&format!("package {aggregate} {{"));
            writer.indent_level();
            emit_port_defs(&bucket.services, writer);
            for service in &bucket.services {
                emit_part_def(service, writer);
            }
            for action in &bucket.actions {
                emit_action_def(action, writer);
            }
            for journey in &bucket.journeys {
                emit_use_case_def(journey, writer);
            }
            for requirement in &bucket.requirements {
                emit_requirement_def(requirement, writer);
            }
            writer.dedent();
            writer.line("}");
        }
        writer.dedent();
        writer.line("}");
    }
}

#[derive(Default)]
struct DomainBucket<'a> {
    services: Vec<&'a Service>,
    journeys: Vec<&'a UseCase>,
    actions: Vec<&'a Action>,
    requirements: Vec<&'a Requirement>,
}

struct SysmlWriter {
    output: String,
    indent: usize,
}

impl SysmlWriter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn finish(self) -> String {
        self.output
    }

    fn indent_level(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    fn line(&mut self, text: &str) {
        if text.is_empty() {
            self.output.push('\n');
            return;
        }
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(text);
        self.output.push('\n');
    }

    fn emit_header(&mut self) {
        self.line("// Generated from blueos-catalog. Rust catalog is source of truth.");
    }

    fn emit_imports(&mut self) {
        self.line("private import Requirements::*;");
        self.line("private import VerificationCases::*;");
        self.line("private import ModelingMetadata::*;");
        self.line("private import Actions::*;");
        self.line("private import UseCases::*;");
        self.line("private import Parts::*;");
        self.line("");
    }

    fn emit_local_metadata(&mut self) {
        self.line("metadata def Evidence {");
        self.indent_level();
        self.line("attribute file : String;");
        self.line("attribute line : Integer;");
        self.line("attribute quote : String;");
        self.dedent();
        self.line("}");
        self.line("");
        self.line("metadata def VersionPresence {");
        self.indent_level();
        self.line("attribute intro_commit : String;");
        self.line("attribute present_in_tags : String;");
        self.line("attribute present_on_master : Boolean;");
        self.line("attribute present_on_1_4_dev : Boolean;");
        self.dedent();
        self.line("}");
        self.line("");
    }

    fn emit_local_requirement_defs(&mut self) {
        self.line("requirement def SystemRequirementCheck :> RequirementCheck {");
        self.indent_level();
        self.line("subject subj : Part;");
        self.dedent();
        self.line("}");
        self.line("requirement def RobustnessRequirementCheck :> RequirementCheck {");
        self.indent_level();
        self.line("subject subj : Part;");
        self.dedent();
        self.line("}");
        self.line("");
    }
}

fn selected_services<'a>(
    catalog: &'a Catalog,
    filter: Option<&SysmlExportFilter>,
) -> Vec<&'a Service> {
    let mut services: Vec<&Service> = catalog
        .services()
        .iter()
        .filter(|service| match filter {
            Some(filter) if !filter.is_empty() => filter.services.contains(&service.id),
            _ => true,
        })
        .collect();
    if let Some(filter) = filter {
        if filter.include_connection_endpoints {
            for service in catalog.services() {
                if services.iter().any(|selected| selected.id == service.id) {
                    continue;
                }
                if connection_endpoint_needed(catalog, service.id, filter) {
                    services.push(service);
                }
            }
        }
    }
    services.sort_by_key(|service| service.id.as_str());
    services
}

fn connection_endpoint_needed(
    catalog: &Catalog,
    service_id: ServiceId,
    filter: &SysmlExportFilter,
) -> bool {
    for service in catalog.services() {
        if !filter.services.contains(&service.id) {
            continue;
        }
        if let AssertedSet::Established { items } = &service.definition.edges {
            for rationaled in items.iter() {
                if rationaled.value.from == service_id || rationaled.value.to == service_id {
                    return true;
                }
            }
        }
    }
    false
}

fn selected_journeys<'a>(
    catalog: &'a Catalog,
    filter: Option<&SysmlExportFilter>,
) -> Vec<&'a UseCase> {
    let mut journeys: Vec<&UseCase> = catalog
        .journeys()
        .iter()
        .filter(|journey| match filter {
            Some(filter) if !filter.is_empty() => filter.journeys.contains(&journey.id),
            _ => true,
        })
        .collect();
    journeys.sort_by_key(|journey| journey.id.as_str());
    journeys
}

fn selected_actions<'a>(
    actions: &'a ActionCatalog,
    filter: Option<&SysmlExportFilter>,
) -> Vec<&'a Action> {
    let mut selected: Vec<&Action> = actions
        .functions()
        .iter()
        .filter(|action| match filter {
            Some(filter) if !filter.is_empty() => filter.actions.contains(action.id.as_str()),
            _ => true,
        })
        .collect();
    selected.sort_by_key(|action| action.id.as_str());
    selected
}

fn selected_requirements<'a>(
    requirements: &'a RequirementCatalog,
    filter: Option<&SysmlExportFilter>,
) -> Vec<&'a Requirement> {
    let mut selected: Vec<&Requirement> = requirements
        .requirements()
        .iter()
        .filter(|requirement| match filter {
            Some(filter) if !filter.is_empty() => filter.requirements.contains(&requirement.id.0),
            _ => true,
        })
        .collect();
    selected.sort_by_key(|requirement| requirement.id.0.as_str());
    selected
}

fn emit_part_def(service: &Service, writer: &mut SysmlWriter) {
    let name = sysml_ident(service.id.as_str());
    writer.line(&format!("part def {name} {{"));
    writer.indent_level();
    emit_observed_scalar("tmux_name", &service.observed.tmux_name, writer);
    emit_observed_scalar("entrypoint", &service.observed.entrypoint, writer);
    emit_observed_path("git_path", &service.observed.git_path, writer);
    emit_observed_set("listen", &service.observed.listen, port_ref_label, writer);
    emit_asserted_scalar(
        "bounded_context",
        &service.definition.bounded_context,
        writer,
    );
    emit_asserted_tier("tier", &service.definition.tier, writer);
    emit_asserted_scalar("health", &service.definition.health, writer);
    emit_asserted_scalar("team", &service.definition.team, writer);
    writer.dedent();
    writer.line("}");
    writer.line("");
}

fn emit_port_defs(services: &[&Service], writer: &mut SysmlWriter) {
    let mut emitted: BTreeSet<String> = BTreeSet::new();
    for service in services {
        let ObservedSet::Known { items } = &service.observed.interfaces else {
            continue;
        };
        for evidenced in items.iter() {
            let label = port_kind_def_name(&evidenced.value);
            if !emitted.insert(label.clone()) {
                continue;
            }
            emit_port_kind_def(&label, &evidenced.value, writer);
        }
    }
}

fn emit_port_kind_def(name: &str, port_kind: &PortKind, writer: &mut SysmlWriter) {
    writer.line(&format!("port def {name} {{"));
    writer.indent_level();
    match port_kind {
        PortKind::Rest {
            path_prefix,
            port,
            versions,
        } => {
            writer.line(&format!(
                "attribute path_prefix : String = {};",
                sysml_string(path_ref_label(path_prefix))
            ));
            writer.line(&format!(
                "attribute port : String = {};",
                sysml_string(&port_ref_label(port))
            ));
            writer.line(&format!(
                "attribute versions : String = {};",
                sysml_string(&versions.join(","))
            ));
        }
        PortKind::Zenoh {
            topics_produced,
            topics_consumed,
        } => {
            writer.line(&format!(
                "attribute topics_produced : String = {};",
                sysml_string(&topics_produced.join(","))
            ));
            writer.line(&format!(
                "attribute topics_consumed : String = {};",
                sysml_string(&topics_consumed.join(","))
            ));
        }
        PortKind::Mavlink { role, connect } => {
            writer.line(&format!(
                "attribute role : String = {};",
                sysml_string(&format!("{role:?}"))
            ));
            writer.line(&format!(
                "attribute connect : String = {};",
                sysml_string(connect)
            ));
        }
        PortKind::Websocket { path, port } | PortKind::HttpStream { path, port } => {
            writer.line(&format!(
                "attribute path : String = {};",
                sysml_string(path_ref_label(path))
            ));
            writer.line(&format!(
                "attribute port : String = {};",
                sysml_string(&port_ref_label(port))
            ));
        }
        PortKind::OutboundHttp { url } => {
            writer.line(&format!("attribute url : String = {};", sysml_string(url)));
        }
        PortKind::Subprocess { command } => {
            writer.line(&format!(
                "attribute command : String = {};",
                sysml_string(command)
            ));
        }
        PortKind::File { path, mode } => {
            writer.line(&format!(
                "attribute path : String = {};",
                sysml_string(path_ref_label(path))
            ));
            writer.line(&format!(
                "attribute mode : String = {};",
                sysml_string(&format!("{mode:?}"))
            ));
        }
        PortKind::Settings { path } => {
            writer.line(&format!(
                "attribute path : String = {};",
                sysml_string(path_ref_label(path))
            ));
        }
        PortKind::Hardware { device } => {
            writer.line(&format!(
                "attribute device : String = {};",
                sysml_string(path_ref_label(device))
            ));
        }
        PortKind::Docker { image } => {
            writer.line(&format!(
                "attribute image : String = {};",
                sysml_string(image)
            ));
        }
    }
    writer.dedent();
    writer.line("}");
    writer.line("");
}

fn emit_cross_connections(
    catalog: &Catalog,
    filter: Option<&SysmlExportFilter>,
    writer: &mut SysmlWriter,
) {
    let selected_services = selected_services(catalog, filter);
    let selected: BTreeSet<ServiceId> =
        selected_services.iter().map(|service| service.id).collect();
    let mut emitted: BTreeSet<String> = BTreeSet::new();
    for service in selected_services {
        let AssertedSet::Established { items } = &service.definition.edges else {
            continue;
        };
        for rationaled in items.iter() {
            let edge = &rationaled.value;
            if !selected.contains(&edge.from) || !selected.contains(&edge.to) {
                continue;
            }
            let key = format!(
                "{}:{}:{}",
                edge.from.as_str(),
                bus_ident(edge.via),
                edge.to.as_str()
            );
            if !emitted.insert(key) {
                continue;
            }
            emit_connection(edge, rationaled.rationale, writer);
        }
    }
}

fn emit_connection(connection: &Connection, rationale: &str, writer: &mut SysmlWriter) {
    let bus = bus_ident(connection.via);
    writer.line(&format!("interface def {bus} {{}}"));
    writer.line(&format!(
        "@Rationale {{ text = {}; }}",
        sysml_string(rationale)
    ));
    writer.line(&format!(
        "connect {} to {} via {bus};",
        sysml_ident(connection.from.as_str()),
        sysml_ident(connection.to.as_str())
    ));
    writer.line("");
}

fn emit_action_def(action: &Action, writer: &mut SysmlWriter) {
    let name = sysml_ident(action.id.as_str());
    writer.line(&format!("action def {name} :> Action {{"));
    writer.indent_level();
    writer.line(&format!(
        "subject subj = {};",
        sysml_ident(action.capability.as_str())
    ));
    for journey_id in &action.verifying_journeys {
        writer.line(&format!(
            "ref verification {} : VerificationCase;",
            sysml_ident(journey_id.as_str())
        ));
    }
    writer.dedent();
    writer.line("}");
    writer.line("");
}

fn emit_use_case_def(journey: &UseCase, writer: &mut SysmlWriter) {
    let name = sysml_ident(journey.id.as_str());
    writer.line(&format!("use case def {name} :> UseCase {{"));
    writer.indent_level();
    emit_availability_on_element(&journey.availability, writer);
    if let GroundedSet::Known { items } = &journey.services {
        if let Some(first) = items.first() {
            writer.line(&format!(
                "subject subj = {};",
                sysml_ident(first.value.as_str())
            ));
        }
    }
    emit_use_case_actors(journey, writer);
    emit_use_case_preconditions(journey, writer);
    emit_use_case_steps(journey, writer);
    writer.dedent();
    writer.line("}");
    writer.line("");
}

fn emit_use_case_actors(journey: &UseCase, writer: &mut SysmlWriter) {
    let mut actors: BTreeSet<String> = BTreeSet::new();
    if let GroundedSet::Known { items } = &journey.steps {
        for item in items.iter() {
            let label = actor_label(&item.value.actor);
            actors.insert(label);
        }
    }
    for actor in actors {
        writer.line(&format!("actor {actor};"));
    }
}

fn emit_use_case_preconditions(journey: &UseCase, writer: &mut SysmlWriter) {
    let GroundedSet::Known { items } = &journey.preconditions else {
        return;
    };
    for item in items.iter() {
        writer.line(&format!(
            "assume constraint {{ doc /* {} */; }}",
            escape_comment(&precondition_text(&item.value))
        ));
    }
}

fn emit_use_case_steps(journey: &UseCase, writer: &mut SysmlWriter) {
    let GroundedSet::Known { items } = &journey.steps else {
        return;
    };
    for (index, item) in items.iter().enumerate() {
        let step_name = format!("step_{index}");
        writer.line(&format!(
            "action {step_name} {{ doc /* {} */; }}",
            escape_comment(item.value.description)
        ));
    }
}

fn emit_requirement_def(requirement: &Requirement, writer: &mut SysmlWriter) {
    let name = sysml_ident(&requirement.id.0);
    let base = requirement_base_type(requirement.kind);
    writer.line(&format!("requirement def {name} :> {base} {{"));
    writer.indent_level();
    emit_availability_on_element(&requirement.availability, writer);
    if let Some(subject) = requirement_subject(requirement) {
        writer.line(&format!("subject subj = {subject};"));
    }
    if let Some(rationale) = &requirement.rationale {
        writer.line(&format!(
            "@Rationale {{ text = {}; }}",
            sysml_string(rationale)
        ));
    }
    for assumption in &requirement.assumptions {
        writer.line(&format!(
            "assume constraint {{ doc /* {} */; }}",
            escape_comment(&assumption.statement)
        ));
    }
    if let RequirementCriteria::Known { items } = &requirement.criteria {
        for item in items.iter() {
            writer.line(&format!(
                "require constraint {{ doc /* {} */; }}",
                escape_comment(&item.text)
            ));
        }
    }
    for child in &requirement.subrequirements {
        writer.line(&format!(
            "requirement subrequirements :> {};",
            sysml_ident(&child.0)
        ));
    }
    for journey_id in &requirement.requirement_verifications {
        writer.line(&format!(
            "verify requirement {};",
            sysml_ident(journey_id.as_str())
        ));
    }
    writer.dedent();
    writer.line("}");
    writer.line("");
}

fn requirement_base_type(kind: RequirementClass) -> &'static str {
    match kind {
        RequirementClass::Functional => "FunctionalRequirementCheck",
        RequirementClass::Interface => "InterfaceRequirementCheck",
        RequirementClass::Performance => "PerformanceRequirementCheck",
        RequirementClass::System => "SystemRequirementCheck",
        RequirementClass::Robustness => "RobustnessRequirementCheck",
    }
}

fn requirement_subject(requirement: &Requirement) -> Option<String> {
    if let Some(function_id) = &requirement.function_id {
        return Some(sysml_ident(function_id.as_str()));
    }
    if let Some(capability) = requirement.feature_id {
        return Some(sysml_ident(capability.as_str()));
    }
    if let Some(journey_id) = requirement.journey_id {
        return Some(sysml_ident(journey_id.as_str()));
    }
    None
}

fn emit_observed_scalar<T: std::fmt::Display>(
    name: &str,
    field: &Observed<T>,
    writer: &mut SysmlWriter,
) {
    match field {
        Observed::Known { value, evidence } => {
            writer.line(&format!(
                "attribute {name} : String = {};",
                sysml_string(&value.to_string())
            ));
            emit_evidence_annotation(evidence, writer);
        }
        Observed::Unknown { reason } => emit_unknown(reason, writer),
    }
}

fn emit_observed_set<T, F>(name: &str, field: &ObservedSet<T>, map: F, writer: &mut SysmlWriter)
where
    F: Fn(&T) -> String,
{
    match field {
        ObservedSet::Known { items } => {
            let joined = items
                .iter()
                .map(|item| map(&item.value))
                .collect::<Vec<_>>()
                .join(", ");
            writer.line(&format!(
                "attribute {name} : String = {};",
                sysml_string(&joined)
            ));
            if let Some(first) = items.first() {
                emit_evidence_annotation(&first.evidence, writer);
            }
        }
        ObservedSet::Unknown { reason } => emit_unknown(reason, writer),
    }
}

fn emit_observed_path(name: &str, field: &Observed<PathRef>, writer: &mut SysmlWriter) {
    match field {
        Observed::Known { value, evidence } => {
            writer.line(&format!(
                "attribute {name} : String = {};",
                sysml_string(value.0)
            ));
            emit_evidence_annotation(evidence, writer);
        }
        Observed::Unknown { reason } => emit_unknown(reason, writer),
    }
}

fn emit_asserted_tier(
    name: &str,
    field: &Asserted<crate::criticality::CriticalityTier>,
    writer: &mut SysmlWriter,
) {
    match field {
        Asserted::Established { value, rationale } => {
            writer.line(&format!(
                "attribute {name} : String = {};",
                sysml_string(&format!("{value:?}"))
            ));
            writer.line(&format!(
                "@Rationale {{ text = {}; }}",
                sysml_string(rationale)
            ));
        }
        Asserted::Unknown { reason } => emit_unknown(reason, writer),
    }
}

fn emit_asserted_scalar<T: std::fmt::Display>(
    name: &str,
    field: &Asserted<T>,
    writer: &mut SysmlWriter,
) {
    match field {
        Asserted::Established { value, rationale } => {
            writer.line(&format!(
                "attribute {name} : String = {};",
                sysml_string(&value.to_string())
            ));
            writer.line(&format!(
                "@Rationale {{ text = {}; }}",
                sysml_string(rationale)
            ));
        }
        Asserted::Unknown { reason } => emit_unknown(reason, writer),
    }
}

fn emit_unknown(reason: &str, writer: &mut SysmlWriter) {
    writer.line("@StatusInfo { status = StatusKind::tbd; }");
    writer.line(&format!("@Issue {{ text = {}; }}", sysml_string(reason)));
}

fn emit_evidence_annotation(evidence: &Evidence, writer: &mut SysmlWriter) {
    writer.line(&format!(
        "@Evidence {{ file = {}; line = {}; quote = {}; }}",
        sysml_string(evidence.file),
        evidence.line,
        sysml_string(evidence.anchor)
    ));
}

fn emit_availability_on_element(availability: &Availability, writer: &mut SysmlWriter) {
    let tags = availability.present_in_tags.join(",");
    writer.line(&format!(
        "@VersionPresence {{ intro_commit = {}; present_in_tags = {}; present_on_master = {}; present_on_1_4_dev = {}; }}",
        sysml_string(availability.intro_commit),
        sysml_string(&tags),
        availability.present_on_master,
        availability.present_on_1_4_dev
    ));
}

fn port_kind_def_name(port_kind: &PortKind) -> String {
    match port_kind {
        PortKind::Rest { .. } => "RestPort".to_string(),
        PortKind::Zenoh { .. } => "ZenohPort".to_string(),
        PortKind::Mavlink { .. } => "MavlinkPort".to_string(),
        PortKind::Websocket { .. } => "WebsocketPort".to_string(),
        PortKind::HttpStream { .. } => "HttpStreamPort".to_string(),
        PortKind::OutboundHttp { .. } => "OutboundHttpPort".to_string(),
        PortKind::Subprocess { .. } => "SubprocessPort".to_string(),
        PortKind::File { .. } => "FilePort".to_string(),
        PortKind::Settings { .. } => "SettingsPort".to_string(),
        PortKind::Hardware { .. } => "HardwarePort".to_string(),
        PortKind::Docker { .. } => "DockerPort".to_string(),
    }
}

fn bus_ident(bus: Bus) -> String {
    match bus {
        Bus::Rest => "RestBus".to_string(),
        Bus::Zenoh => "ZenohBus".to_string(),
        Bus::Mavlink => "MavlinkBus".to_string(),
        Bus::Websocket => "WebsocketBus".to_string(),
        Bus::HttpStream => "HttpStreamBus".to_string(),
        Bus::Subprocess => "SubprocessBus".to_string(),
        Bus::File => "FileBus".to_string(),
        Bus::Settings => "SettingsBus".to_string(),
        Bus::Hardware => "HardwareBus".to_string(),
        Bus::Docker => "DockerBus".to_string(),
    }
}

fn aggregate_for_service(catalog: &Catalog, service_id: ServiceId) -> Aggregate {
    if let Some(service) = catalog.service_by_id(&service_id) {
        if let AssertedSet::Established { items } = &service.definition.capabilities {
            if let Some(first) = items.first() {
                if let Some(def) = capability_def(first.value) {
                    return def.aggregate;
                }
            }
        }
    }
    Aggregate::HostControl
}

fn aggregate_for_journey(catalog: &Catalog, journey: &UseCase) -> Aggregate {
    if let GroundedSet::Known { items } = &journey.capability_refs {
        if let Some(first) = items.first() {
            if let Some(def) = capability_def(first.value) {
                return def.aggregate;
            }
        }
    }
    if let GroundedSet::Known { items } = &journey.services {
        if let Some(first) = items.first() {
            return aggregate_for_service(catalog, first.value);
        }
    }
    Aggregate::HostControl
}

fn aggregate_for_capability(capability: CapabilityId) -> Aggregate {
    capability_def(capability)
        .map(|def| def.aggregate)
        .unwrap_or(Aggregate::HostControl)
}

fn actor_label(actor: &Actor) -> String {
    match actor {
        Actor::Operator => "operator".to_string(),
        Actor::Service(service) => sysml_ident(service.as_str()),
        Actor::Subprocess(name) => sysml_ident(name),
        Actor::Frontend(page) => sysml_ident(page.as_str()),
    }
}

fn precondition_text(precondition: &Precondition) -> String {
    match precondition {
        Precondition::ServiceState { service, state } => {
            format!("{} state={state}", service.as_str())
        }
        Precondition::Network(state) => format!("{state:?}"),
        Precondition::HardwarePresent(text) => (*text).to_string(),
        Precondition::ConfigClean(path) => format!("config clean: {}", path.0),
        Precondition::Other(text) => (*text).to_string(),
        Precondition::Hardware(assumption) => format!("{assumption:?}"),
        Precondition::Software(assumption) => format!("{assumption:?}"),
        Precondition::NetworkResource(resource) => format!("{resource:?}"),
        Precondition::Data(assumption) => format!("{assumption:?}"),
    }
}

fn port_ref_label(port: &PortRef) -> String {
    match port {
        PortRef::Literal(value) => value.to_string(),
        PortRef::Env(name) => format!("env:{name}"),
    }
}

fn path_ref_label(path: &PathRef) -> &str {
    path.0
}

pub fn sysml_ident(raw: &str) -> String {
    let mut ident = raw
        .chars()
        .map(|ch| match ch {
            '/' | '-' | '.' => '_',
            _ => ch,
        })
        .collect::<String>();
    if ident.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        ident.insert(0, '_');
    }
    ident
}

fn sysml_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn escape_comment(value: &str) -> String {
    value.replace("*/", "* /")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn subset_export_contains_expected_constructs() {
        let catalog = Catalog::bootstrap();
        let filter = SysmlExportFilter::subset_golden();
        let output = export_sysml(&catalog, Some(&filter));
        assert!(output.contains("action def"));
        assert!(output.contains("part def"));
        assert!(output.contains("use case def"));
        assert!(output.contains("requirement def"));
        assert!(output.contains("assume"));
        assert!(output.contains("@Rationale"));
        assert!(output.contains("@Evidence"));
        assert!(output.contains("@StatusInfo"));
        assert!(output.contains("connect"));
    }

    #[test]
    fn sysml_ident_replaces_slashes_and_leading_digits() {
        assert_eq!(
            sysml_ident("REQ/peripherals/sonar/FUN/list_detected_ping_sensors"),
            "REQ_peripherals_sonar_FUN_list_detected_ping_sensors"
        );
        assert_eq!(sysml_ident("9110"), "_9110");
        assert_eq!(
            sysml_ident("mavlink-camera-manager"),
            "mavlink_camera_manager"
        );
    }

    #[test]
    fn subset_export_is_deterministic() {
        let catalog = Catalog::bootstrap();
        let filter = SysmlExportFilter::subset_golden();
        let first = export_sysml(&catalog, Some(&filter));
        let second = export_sysml(&catalog, Some(&filter));
        assert_eq!(first, second);
    }
}
