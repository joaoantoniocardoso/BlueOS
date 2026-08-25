use std::collections::{BTreeSet, HashMap, HashSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::feature::{capability_journey_view, declared_capabilities, DeclaredCapability};
use crate::function::{Action, ActionCatalog, ActionId};
use crate::system_overlay::SYSTEM_OVERLAY_ENTRIES;
use catalog_analysis::coverage::precondition_label;
use catalog_core::catalog::Catalog;
use catalog_core::http::resolve_http_path;
use catalog_data::capability_registry::{capability_def, frontend_capability_def};
use catalog_harness::negative_probes::{NegativeProbe, NEGATIVE_PROBES};
use catalog_kernel::aggregate::Aggregate;
use catalog_kernel::domain::domain_of;
use catalog_kernel::domain::Domain;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedSet, Provenance};
use catalog_kernel::version::Availability;
use catalog_model::http::http_method_label;
use catalog_model::journey::{
    http_automatable, journey_requirements, DataAssumption, HardwareAssumption, HttpMethod,
    NetworkResource, NetworkState, Precondition, RouteRef, SoftwareAssumption, StepOutcome,
    UseCase,
};
use catalog_model::runtime::SloBaseline;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum RequirementClass {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "functional")]
    Functional,
    #[serde(rename = "interface")]
    Interface,
    #[serde(rename = "performance")]
    Performance,
    #[serde(rename = "robustness")]
    Robustness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssumptionKind {
    Hardware,
    Software,
    Data,
    Network,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Assumption {
    pub statement: String,
    pub source_use_case: String,
    pub kind: AssumptionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct RequirementId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status")]
pub enum RequirementStatement {
    #[serde(rename = "known")]
    Known { text: String },
    #[serde(rename = "unknown")]
    Unknown { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCriterion {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status")]
pub enum RequirementCriteria {
    #[serde(rename = "known")]
    Known { items: Vec<AcceptanceCriterion> },
    #[serde(rename = "unknown")]
    Unknown { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ContaminationSource {
    Capability { id: CapabilityId },
    Journey { id: JourneyId },
    Overlay { suffix: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ContaminationFinding {
    pub source: ContaminationSource,
    pub substring: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Requirement {
    pub id: RequirementId,
    pub kind: RequirementClass,
    pub domain: Domain,
    pub aggregate: Aggregate,
    pub availability: Availability,
    pub statement: RequirementStatement,
    pub criteria: RequirementCriteria,
    pub feature_id: Option<CapabilityId>,
    pub journey_id: Option<JourneyId>,
    pub function_id: Option<ActionId>,
    pub requirement_verifications: Vec<JourneyId>,
    pub subrequirements: Vec<RequirementId>,
    pub assumptions: Vec<Assumption>,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct RequirementCatalog {
    pub(crate) requirements: Vec<Requirement>,
    contamination_findings: Vec<ContaminationFinding>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RequirementValidationError {
    #[error("feature {feature} has no system requirement")]
    MissingSystemRequirement { feature: String },
    #[error("system requirement {id} has no functional child and no Unknown coverage")]
    SystemWithoutFunctional { id: String },
    #[error("functional requirement {id} has no acceptance criterion and no Unknown coverage")]
    FunctionalWithoutCriterion { id: String },
    #[error("overlay system requirement {id} has no acceptance criterion")]
    SystemOverlayWithoutCriterion { id: String },
    #[error("journey {journey} is Http-automatable but step {step} lacks runtime evidence")]
    HttpAutomatableWithoutRuntime { journey: String, step: usize },
    #[error("requirement {id} statement is contaminated: {substring}")]
    ContaminatedStatementEmitted { id: String, substring: String },
}

impl RequirementCatalog {
    pub fn bootstrap() -> Self {
        Self::from_catalog(&Catalog::bootstrap())
    }

    pub fn from_catalog(catalog: &Catalog) -> Self {
        let capabilities = declared_capabilities(catalog);
        let mut contamination_findings = Vec::new();
        let mut requirements = Vec::new();
        let functions = ActionCatalog::from_catalog(catalog);
        let mut functional_by_capability: HashMap<CapabilityId, Vec<RequirementId>> =
            HashMap::new();

        for capability in &capabilities {
            let id = system_requirement_id(capability);
            let availability = availability_for_capability(catalog, capability.id);
            requirements.push(Requirement {
                id,
                kind: RequirementClass::System,
                domain: domain_of(capability.aggregate),
                aggregate: capability.aggregate,
                availability,
                statement: RequirementStatement::Unknown {
                    reason: "pending functional child composition".to_string(),
                },
                criteria: RequirementCriteria::Unknown {
                    reason: "system requirements link to functional children".to_string(),
                },
                feature_id: Some(capability.id),
                journey_id: None,
                function_id: None,
                requirement_verifications: Vec::new(),
                subrequirements: Vec::new(),
                assumptions: Vec::new(),
                rationale: Some(capability.rationale.clone()),
            });
        }

        for function in functions.functions() {
            let aggregate = aggregate_for_capability(function.capability);
            let domain = domain_of(aggregate);
            let id = functional_requirement_id_for_function(domain, aggregate, &function.id);
            let statement =
                compose_function_statement(catalog, function, &mut contamination_findings);
            let criteria = compose_function_criteria(catalog, function);
            let availability = availability_for_capability(catalog, function.capability);
            requirements.push(Requirement {
                id: id.clone(),
                kind: RequirementClass::Functional,
                domain,
                aggregate,
                availability,
                statement,
                criteria,
                feature_id: Some(function.capability),
                journey_id: None,
                function_id: Some(function.id.clone()),
                requirement_verifications: function.verifying_journeys.clone(),
                subrequirements: Vec::new(),
                assumptions: Vec::new(),
                rationale: None,
            });
            functional_by_capability
                .entry(function.capability)
                .or_default()
                .push(id);
        }

        let journey_view = capability_journey_view(catalog);
        for capability in &capabilities {
            let system_id = system_requirement_id(capability);
            let children = functional_by_capability
                .get(&capability.id)
                .cloned()
                .unwrap_or_default();
            let children_empty = children.is_empty();
            let idx = requirements
                .iter()
                .position(|req| req.id == system_id)
                .expect("system requirement exists");
            requirements[idx].subrequirements = children.clone();
            if children_empty {
                requirements[idx].statement = RequirementStatement::Unknown {
                    reason: if journey_view
                        .unreferenced_capabilities
                        .contains(&capability.id)
                    {
                        "no journey references this capability".to_string()
                    } else {
                        "no functional requirement derived for this capability".to_string()
                    },
                };
                requirements[idx].criteria = RequirementCriteria::Unknown {
                    reason: if journey_view
                        .unreferenced_capabilities
                        .contains(&capability.id)
                    {
                        "no journey references this capability".to_string()
                    } else {
                        "no functional requirement derived for this capability".to_string()
                    },
                };
            } else {
                requirements[idx].statement = compose_system_statement(
                    &children,
                    &requirements,
                    ContaminationSource::Capability { id: capability.id },
                    &mut contamination_findings,
                );
                requirements[idx].criteria = compose_system_criteria(&children, &requirements);
            }
        }

        let mut seen_routes: BTreeSet<String> = BTreeSet::new();
        for journey in catalog.journeys() {
            if let GroundedSet::Known { items: steps } = &journey.steps {
                for step in steps.iter() {
                    if let Some(Grounded::Known { value: route, .. }) = &step.value.route {
                        if let Some(resolved) = resolve_http_path(catalog, route) {
                            let key = format!("{} {}", http_method_label(&route.method), resolved);
                            if seen_routes.insert(key.clone()) {
                                add_interface_requirement(
                                    catalog,
                                    &functions,
                                    &mut requirements,
                                    &mut contamination_findings,
                                    journey,
                                    route,
                                    &resolved,
                                    step.value.description,
                                );
                            }
                        }
                    }
                }
            }
        }

        for service in catalog.services() {
            if let GroundedSet::Known { items } = &service.runtime.slo_baselines {
                for item in items.iter() {
                    add_performance_requirement(catalog, &mut requirements, service.id, item);
                }
            }
        }

        for probe in NEGATIVE_PROBES {
            add_robustness_requirement(catalog, &mut requirements, probe);
        }

        append_overlay_requirements(catalog, &mut requirements, &mut contamination_findings);
        attach_assumptions(catalog, &mut requirements);

        Self {
            requirements,
            contamination_findings,
        }
    }

    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }

    pub fn requirements_mut(&mut self) -> &mut Vec<Requirement> {
        &mut self.requirements
    }

    pub fn contamination_findings(&self) -> &[ContaminationFinding] {
        &self.contamination_findings
    }

    pub fn count_by_kind(&self) -> HashMap<RequirementClass, usize> {
        let mut counts = HashMap::new();
        for requirement in &self.requirements {
            *counts.entry(requirement.kind).or_default() += 1;
        }
        counts
    }

    pub fn filter_by_version(&self, dut_tag: &str) -> RequirementCatalog {
        let requirements = self
            .requirements
            .iter()
            .filter(|req| req.availability.present_on_dut(dut_tag))
            .cloned()
            .collect();
        RequirementCatalog {
            requirements,
            contamination_findings: self.contamination_findings.clone(),
        }
    }

    pub fn validate_emitted_statements(&self) -> Result<(), Vec<RequirementValidationError>> {
        let mut errors = Vec::new();
        for requirement in &self.requirements {
            if let RequirementStatement::Known { text } = &requirement.statement {
                if let Some(substring) = find_contamination(text) {
                    errors.push(RequirementValidationError::ContaminatedStatementEmitted {
                        id: requirement.id.0.clone(),
                        substring,
                    });
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_structure(
        &self,
        catalog: &Catalog,
    ) -> Result<(), Vec<RequirementValidationError>> {
        let mut errors = Vec::new();
        if let Err(stmt_errors) = self.validate_emitted_statements() {
            errors.extend(stmt_errors);
        }

        let system_ids: HashSet<&str> = self
            .requirements
            .iter()
            .filter(|req| req.kind == RequirementClass::System)
            .map(|req| req.id.0.as_str())
            .collect();

        for capability in declared_capabilities(catalog) {
            let expected = system_requirement_id(&capability).0;
            if !system_ids.contains(expected.as_str()) {
                errors.push(RequirementValidationError::MissingSystemRequirement {
                    feature: capability.id.as_str().to_string(),
                });
            }
        }

        for requirement in &self.requirements {
            // Overlay system claims are not composed from journeys, so they have no functional children.
            if requirement.kind == RequirementClass::System
                && requirement.subrequirements.is_empty()
                && !requirement.id.0.contains("/SYS-OVR/")
                && !matches!(requirement.criteria, RequirementCriteria::Unknown { .. })
            {
                errors.push(RequirementValidationError::SystemWithoutFunctional {
                    id: requirement.id.0.clone(),
                });
            }
            if requirement.kind == RequirementClass::Functional
                && matches!(
                    &requirement.criteria,
                    RequirementCriteria::Known { items } if items.is_empty()
                )
            {
                errors.push(RequirementValidationError::FunctionalWithoutCriterion {
                    id: requirement.id.0.clone(),
                });
            }
            if requirement.id.0.contains("/SYS-OVR/")
                && matches!(
                    &requirement.criteria,
                    RequirementCriteria::Known { items } if items.is_empty()
                )
            {
                errors.push(RequirementValidationError::SystemOverlayWithoutCriterion {
                    id: requirement.id.0.clone(),
                });
            }
            if requirement.kind == RequirementClass::Functional {
                if requirement.requirement_verifications.is_empty() {
                    errors.push(RequirementValidationError::FunctionalWithoutCriterion {
                        id: requirement.id.0.clone(),
                    });
                }
                for journey_id in &requirement.requirement_verifications {
                    let journey = catalog
                        .journey_by_id(journey_id)
                        .expect("functional verifying journey exists");
                    if http_automatable(journey)
                        && matches!(
                            &requirement.criteria,
                            RequirementCriteria::Known { items }
                                if !items.is_empty()
                                    && !items.iter().any(|item| item.text.contains("capture="))
                        )
                    {
                        errors.push(RequirementValidationError::HttpAutomatableWithoutRuntime {
                            journey: journey_id.to_string(),
                            step: 0,
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn add_interface_requirement(
    catalog: &Catalog,
    functions: &ActionCatalog,
    requirements: &mut Vec<Requirement>,
    contamination_findings: &mut Vec<ContaminationFinding>,
    journey: &UseCase,
    route: &RouteRef,
    resolved: &str,
    description: &str,
) {
    let aggregate = aggregate_for_journey(journey);
    let domain = domain_of(aggregate);
    let id = interface_requirement_id(domain, aggregate, &route.method, resolved);
    let statement = derive_statement(
        description,
        ContaminationSource::Journey { id: journey.id },
        contamination_findings,
    );
    let capture = runtime_capture_for_route(catalog, journey, route);
    let criteria = interface_criteria(&route.method, resolved, capture);
    let function_id = resolve_function_id_for_interface(catalog, functions, journey, route);
    requirements.push(Requirement {
        id,
        kind: RequirementClass::Interface,
        domain,
        aggregate,
        availability: journey.availability,
        statement,
        criteria,
        feature_id: primary_capability_for_journey(journey),
        journey_id: Some(journey.id),
        function_id,
        requirement_verifications: Vec::new(),
        subrequirements: Vec::new(),
        assumptions: Vec::new(),
        rationale: None,
    });
}

fn add_performance_requirement(
    catalog: &Catalog,
    requirements: &mut Vec<Requirement>,
    service_id: ServiceId,
    item: &catalog_kernel::provenance::GroundedItem<SloBaseline>,
) {
    let slo = &item.value;
    let aggregate = aggregate_for_service(catalog, service_id);
    let domain = domain_of(aggregate);
    let resolved =
        resolve_http_path(catalog, &slo.route).unwrap_or_else(|| slo.route.path.to_string());
    let id = performance_requirement_id(domain, aggregate, &slo.route.method, &resolved);
    let statement_text = match capability_phrase_for_service(catalog, service_id) {
        Some(phrase) => {
            format!("Captured {phrase} endpoint latency stays within measured baseline")
        }
        None => "Captured endpoint latency stays within measured baseline".to_string(),
    };
    let capture = match &item.provenance {
        Provenance::Runtime { capture, .. } => Some((*capture).to_string()),
        other => Some(format!("{other:?}")),
    };
    requirements.push(Requirement {
        id,
        kind: RequirementClass::Performance,
        domain,
        aggregate,
        availability: availability_for_service(catalog, service_id),
        statement: RequirementStatement::Known {
            text: statement_text,
        },
        criteria: RequirementCriteria::Known {
            items: vec![AcceptanceCriterion {
                text: format!(
                    "{} {} p50={}ms p95={}ms p99={}ms n={}",
                    http_method_label(&slo.route.method),
                    resolved,
                    slo.latency_p50_ms,
                    slo.latency_p95_ms,
                    slo.latency_p99_ms,
                    slo.sample_size
                ),
            }],
        },
        feature_id: None,
        journey_id: None,
        function_id: None,
        requirement_verifications: Vec::new(),
        subrequirements: Vec::new(),
        assumptions: Vec::new(),
        rationale: capture,
    });
}

fn add_robustness_requirement(
    catalog: &Catalog,
    requirements: &mut Vec<Requirement>,
    probe: &NegativeProbe,
) {
    let journey = catalog
        .journey_by_id(&probe.journey_id)
        .expect("negative probe journey exists");
    let aggregate = aggregate_for_journey(journey);
    let domain = domain_of(aggregate);
    let id = robustness_requirement_id(domain, aggregate, probe.id);
    let statement_text = match primary_capability_for_journey(journey) {
        Some(capability) => format!(
            "Invalid or unsafe {} request is rejected safely",
            capability.as_str().replace('_', " ")
        ),
        None => "Invalid or unsafe request is rejected safely".to_string(),
    };
    requirements.push(Requirement {
        id,
        kind: RequirementClass::Robustness,
        domain,
        aggregate,
        availability: journey.availability,
        statement: RequirementStatement::Known {
            text: statement_text,
        },
        criteria: RequirementCriteria::Known {
            items: vec![AcceptanceCriterion {
                text: format_negative_probe_criterion(probe),
            }],
        },
        feature_id: primary_capability_for_journey(journey),
        journey_id: Some(probe.journey_id),
        function_id: None,
        requirement_verifications: Vec::new(),
        subrequirements: Vec::new(),
        assumptions: Vec::new(),
        rationale: None,
    });
}

fn attach_assumptions(catalog: &Catalog, requirements: &mut [Requirement]) {
    for requirement in requirements.iter_mut() {
        if requirement.kind == RequirementClass::Functional {
            requirement.assumptions =
                assumptions_for_journeys(catalog, &requirement.requirement_verifications);
        }
    }

    let system_journeys: Vec<Vec<JourneyId>> = requirements
        .iter()
        .map(|requirement| {
            if requirement.kind != RequirementClass::System {
                return Vec::new();
            }
            let mut journey_ids = BTreeSet::new();
            for child_id in &requirement.subrequirements {
                if let Some(child) = requirements.iter().find(|req| &req.id == child_id) {
                    journey_ids.extend(child.requirement_verifications.iter().copied());
                }
            }
            if let Some(entry) = overlay_entry_for_requirement_id(&requirement.id.0) {
                journey_ids.extend(entry.journey_ids.iter().copied());
            }
            journey_ids.into_iter().collect()
        })
        .collect();

    for (requirement, journey_ids) in requirements.iter_mut().zip(system_journeys) {
        if requirement.kind == RequirementClass::System {
            requirement.assumptions = assumptions_for_journeys(catalog, &journey_ids);
        }
    }
}

fn assumptions_for_journeys(catalog: &Catalog, journey_ids: &[JourneyId]) -> Vec<Assumption> {
    let mut seen = BTreeSet::new();
    let mut assumptions = Vec::new();
    for journey_id in journey_ids {
        let journey = catalog
            .journey_by_id(journey_id)
            .unwrap_or_else(|| panic!("assumption cites unknown journey {journey_id}"));
        for precondition in journey_requirements(journey) {
            let label = precondition_label(precondition);
            if seen.insert(label) {
                assumptions.push(Assumption {
                    statement: precondition_statement(precondition),
                    source_use_case: journey.id.to_string(),
                    kind: assumption_kind(precondition),
                });
            }
        }
    }
    assumptions
}

fn assumption_kind(precondition: &Precondition) -> AssumptionKind {
    match precondition {
        Precondition::Hardware(_) | Precondition::HardwarePresent(_) => AssumptionKind::Hardware,
        Precondition::Software(_) => AssumptionKind::Software,
        Precondition::Data(_) => AssumptionKind::Data,
        Precondition::Network(_) | Precondition::NetworkResource(_) => AssumptionKind::Network,
        Precondition::Other(_)
        | Precondition::ServiceState { .. }
        | Precondition::ConfigClean(_) => AssumptionKind::Other,
    }
}

fn overlay_entry_for_requirement_id(
    id: &str,
) -> Option<&'static crate::system_overlay::SystemOverlayEntry> {
    SYSTEM_OVERLAY_ENTRIES
        .iter()
        .find(|entry| id.ends_with(&format!("/SYS-OVR/{}", entry.suffix)))
}

fn compose_function_statement(
    catalog: &Catalog,
    function: &Action,
    findings: &mut Vec<ContaminationFinding>,
) -> RequirementStatement {
    let mut texts = Vec::new();
    let mut seen = HashSet::new();
    for journey_id in &function.verifying_journeys {
        let journey = catalog
            .journey_by_id(journey_id)
            .expect("verifying journey exists");
        let text = grounded_text(&journey.summary);
        if seen.insert(text) {
            texts.push(text);
        }
    }
    if texts.is_empty() {
        return RequirementStatement::Unknown {
            reason: "no verifying journey summaries".to_string(),
        };
    }
    let source = ContaminationSource::Journey {
        id: function.verifying_journeys[0],
    };
    derive_statement(&texts.join("; "), source, findings)
}

fn compose_function_criteria(catalog: &Catalog, function: &Action) -> RequirementCriteria {
    let mut items = Vec::new();
    let mut seen = HashSet::new();
    for journey_id in &function.verifying_journeys {
        let journey = catalog
            .journey_by_id(journey_id)
            .expect("verifying journey exists");
        match functional_criteria(catalog, journey) {
            RequirementCriteria::Known {
                items: journey_items,
            } => {
                for item in journey_items {
                    if seen.insert(item.text.clone()) {
                        items.push(item);
                    }
                }
            }
            RequirementCriteria::Unknown { reason } => {
                return RequirementCriteria::Unknown {
                    reason: format!(
                        "verifying journey {} has unknown criteria: {}",
                        journey_id, reason
                    ),
                };
            }
        }
    }
    if items.is_empty() {
        return RequirementCriteria::Unknown {
            reason: "verifying journeys have no acceptance criteria".to_string(),
        };
    }
    for journey_id in &function.verifying_journeys {
        let journey = catalog
            .journey_by_id(journey_id)
            .expect("verifying journey exists");
        if http_automatable(journey) && !items.iter().any(|item| item.text.contains("capture=")) {
            return RequirementCriteria::Unknown {
                reason: "http-automatable verifying journey lacks runtime capture evidence"
                    .to_string(),
            };
        }
    }
    RequirementCriteria::Known { items }
}

fn functional_criteria(catalog: &Catalog, journey: &UseCase) -> RequirementCriteria {
    if let GroundedSet::Known { items: steps } = &journey.steps {
        let mut items = Vec::new();
        for (step_index, step) in steps.iter().enumerate() {
            if let Some(criterion) = step_criterion(catalog, journey, step_index, step) {
                items.push(criterion);
            }
        }
        if items.is_empty() {
            return RequirementCriteria::Unknown {
                reason: "journey has no step-level acceptance criteria".to_string(),
            };
        }
        if http_automatable(journey) && !items.iter().any(|item| item.text.contains("capture=")) {
            return RequirementCriteria::Unknown {
                reason: "http-automatable journey lacks runtime capture evidence".to_string(),
            };
        }
        return RequirementCriteria::Known { items };
    }
    RequirementCriteria::Unknown {
        reason: "journey steps are not grounded".to_string(),
    }
}

fn step_criterion(
    catalog: &Catalog,
    _journey: &UseCase,
    step_index: usize,
    step: &catalog_kernel::provenance::GroundedItem<catalog_model::journey::JourneyStep>,
) -> Option<AcceptanceCriterion> {
    let route = step.value.route.as_ref();
    let outcome = step.value.outcome.as_ref();
    match (route, outcome) {
        (
            Some(Grounded::Known { value: route, .. }),
            Some(Grounded::Known {
                value: outcome,
                provenance,
            }),
        ) => {
            let resolved =
                resolve_http_path(catalog, route).unwrap_or_else(|| route.path.to_string());
            let capture = match provenance {
                Provenance::Runtime {
                    capture,
                    environment,
                } => {
                    format!("capture={capture} env={environment}")
                }
                other => format!("provenance={other:?}"),
            };
            Some(AcceptanceCriterion {
                text: format_step_criterion(route, &resolved, outcome, &capture),
            })
        }
        _ => {
            if step.value.description.is_empty() {
                None
            } else {
                Some(AcceptanceCriterion {
                    text: format!("step {}: {}", step_index, step.value.description),
                })
            }
        }
    }
}

fn interface_criteria(
    method: &HttpMethod,
    resolved: &str,
    capture: Option<String>,
) -> RequirementCriteria {
    let mut text = format!("{} {}", http_method_label(method), resolved);
    if let Some(capture) = capture {
        text.push_str(&format!(" ({capture})"));
    }
    RequirementCriteria::Known {
        items: vec![AcceptanceCriterion { text }],
    }
}

fn format_step_criterion(
    route: &RouteRef,
    resolved: &str,
    outcome: &StepOutcome,
    capture: &str,
) -> String {
    let mut text = format!("{} {}", http_method_label(&route.method), resolved);
    if let Some(status) = outcome.expected_status {
        text.push_str(&format!(" status={status}"));
    }
    if let Some(predicate) = outcome.body_predicate {
        text.push_str(&format!(" body~={predicate}"));
    }
    if let Some(transition) = &outcome.transition {
        text.push_str(&format!(
            " transition {}:{}->{}",
            transition.machine, transition.from, transition.to
        ));
    }
    text.push_str(&format!(" ({capture})"));
    text
}

fn format_negative_probe_criterion(probe: &NegativeProbe) -> String {
    let mut text = format!("{} {}", http_method_label(&probe.method), probe.path);
    if let Some(query) = probe.query {
        text.push_str(&format!("?{query}"));
    }
    if let Some(status) = probe.expected_status {
        text.push_str(&format!(" status={status}"));
    }
    text.push_str(&format!(" class={:?} blast={:?}", probe.class, probe.blast));
    text
}

fn runtime_capture_for_route(
    catalog: &Catalog,
    journey: &UseCase,
    route: &RouteRef,
) -> Option<String> {
    if let GroundedSet::Known { items: steps } = &journey.steps {
        for step in steps.iter() {
            if let Some(Grounded::Known {
                value: step_route, ..
            }) = &step.value.route
            {
                if step_route == route {
                    if let Some(Grounded::Known {
                        provenance:
                            Provenance::Runtime {
                                capture,
                                environment,
                            },
                        ..
                    }) = &step.value.outcome
                    {
                        return Some(format!("capture={capture} env={environment}"));
                    }
                }
            }
        }
    }
    let _ = catalog;
    None
}

fn derive_statement(
    text: &str,
    source: ContaminationSource,
    findings: &mut Vec<ContaminationFinding>,
) -> RequirementStatement {
    if text.trim().is_empty() {
        return RequirementStatement::Unknown {
            reason: "source text is empty".to_string(),
        };
    }
    if let Some(substring) = find_contamination(text) {
        findings.push(ContaminationFinding { source, substring });
        return RequirementStatement::Unknown {
            reason: "source text is contaminated; awaiting re-grounding".to_string(),
        };
    }
    RequirementStatement::Known {
        text: text.trim().to_string(),
    }
}

fn compose_system_statement(
    subrequirements: &[RequirementId],
    requirements: &[Requirement],
    source: ContaminationSource,
    findings: &mut Vec<ContaminationFinding>,
) -> RequirementStatement {
    let mut texts = Vec::new();
    let mut seen = HashSet::new();
    for child_id in subrequirements {
        let child = requirements
            .iter()
            .find(|req| &req.id == child_id)
            .expect("functional child requirement");
        if let RequirementStatement::Known { text } = &child.statement {
            if seen.insert(text.as_str()) {
                texts.push(text.as_str());
            }
        }
    }
    if texts.is_empty() {
        return RequirementStatement::Unknown {
            reason: "all functional child statements are unknown".to_string(),
        };
    }
    derive_statement(&texts.join("; "), source, findings)
}

fn precondition_statement(precondition: &Precondition) -> String {
    match precondition {
        Precondition::ServiceState { service, state } => {
            format!("{} service is in the {} state", service.as_str(), state)
        }
        Precondition::Network(NetworkState::Online) => {
            "network connectivity is available".to_string()
        }
        Precondition::Network(NetworkState::Offline) => {
            "network connectivity is unavailable".to_string()
        }
        Precondition::HardwarePresent(label) => format!("{label} hardware is present"),
        Precondition::ConfigClean(path) => format!("{} configuration is clean", path.0),
        Precondition::Other(label) => (*label).to_string(),
        Precondition::Hardware(hardware) => match hardware {
            HardwareAssumption::FlightController(board) => {
                format!("a {board:?} flight controller is connected")
            }
            HardwareAssumption::UsbCamera => "a USB camera is connected".to_string(),
            HardwareAssumption::Ping1d => "a Ping1D sonar is connected".to_string(),
            HardwareAssumption::Ping360 => "a Ping360 sonar is connected".to_string(),
            HardwareAssumption::ExternalNmeaGps => "an external NMEA GPS is connected".to_string(),
            HardwareAssumption::UsbSerialDevice => "a USB serial device is connected".to_string(),
            HardwareAssumption::RaspberryPi5 => "a Raspberry Pi 5 host is in use".to_string(),
        },
        Precondition::Software(software) => match software {
            SoftwareAssumption::PirateMode => "pirate mode is enabled".to_string(),
            SoftwareAssumption::AdvancedMode => "advanced mode is enabled".to_string(),
            SoftwareAssumption::DevMode => "developer mode is enabled".to_string(),
            SoftwareAssumption::ConfirmDangerousOp => {
                "the operator confirmed a dangerous operation".to_string()
            }
        },
        Precondition::NetworkResource(resource) => match resource {
            NetworkResource::WifiRadioPresent => "a Wi-Fi radio is present".to_string(),
            NetworkResource::KnownWifiNetwork => "a known Wi-Fi network is available".to_string(),
            NetworkResource::HotspotCapable => "hotspot capability is available".to_string(),
            NetworkResource::WiredEthernetPresent => "wired Ethernet is present".to_string(),
            NetworkResource::UsbOtgPresent => "USB OTG is present".to_string(),
        },
        Precondition::Data(data) => match data {
            DataAssumption::ExtensionInstalled => "an extension is installed".to_string(),
            DataAssumption::LocalBlueosVersionAvailable => {
                "a local BlueOS version image is available".to_string()
            }
            DataAssumption::SerialBridgeConfigured => "a serial bridge is configured".to_string(),
            DataAssumption::NmeaSocketConfigured => "an NMEA socket is configured".to_string(),
            DataAssumption::RecordingListed => "a video recording is listed".to_string(),
            DataAssumption::WifiNetworkSaved => "a Wi-Fi network is saved".to_string(),
            DataAssumption::WifiCurrentlyConnected => "Wi-Fi is currently connected".to_string(),
            DataAssumption::OnboardDhcpServerActive => {
                "the onboard DHCP server is active".to_string()
            }
        },
    }
}

pub fn find_contamination(text: &str) -> Option<String> {
    if let Some(verb) = find_http_verb_token(text) {
        return Some(verb);
    }
    if let Some(path) = find_route_path_contamination(text) {
        return Some(path);
    }
    if let Some(service) = find_multi_token_service_id(text) {
        return Some(service);
    }
    if let Some(service) = find_single_token_service_id(text) {
        return Some(service);
    }
    for token in text.split(|c: char| !c.is_ascii_digit()) {
        if is_likely_port_token(token) {
            return Some(token.to_string());
        }
    }
    for token in text.split(|c: char| !c.is_ascii_digit()) {
        if token.len() == 3
            && token.chars().all(|ch| ch.is_ascii_digit())
            && matches!(
                token.parse::<u16>(),
                Ok(400 | 401 | 403 | 404 | 405 | 409 | 422 | 500 | 502 | 503)
            )
        {
            return Some(token.to_string());
        }
    }
    None
}

const HTTP_VERBS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

const MULTI_TOKEN_SERVICE_IDS: &[&str] = &[
    "ardupilot_manager",
    "bag_of_holding",
    "cable_guy",
    "disk_usage",
    "mavlink-camera-manager",
    "nmea_injector",
    "recorder_extractor",
    "user_terminal",
];

const SINGLE_TOKEN_SERVICE_IDS: &[&str] = &[
    "bridget",
    "commander",
    "filebrowser",
    "iperf3",
    "kraken",
    "linux2rest",
    "mavlink2rest",
    "nginx",
    "pardal",
    "ttyd",
    "versionchooser",
    "zenohd",
];

fn find_single_token_service_id(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    for token in lower.split(|c: char| !c.is_ascii_alphanumeric()) {
        if SINGLE_TOKEN_SERVICE_IDS.contains(&token) {
            return Some(token.to_string());
        }
    }
    None
}

fn find_http_verb_token(text: &str) -> Option<String> {
    for token in text.split(|c: char| !c.is_ascii_alphabetic()) {
        if HTTP_VERBS.contains(&token) {
            return Some(token.to_string());
        }
    }
    None
}

fn is_likely_port_token(token: &str) -> bool {
    if token.len() < 4 || token.len() > 5 {
        return false;
    }
    if !token.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    let Ok(port) = token.parse::<u32>() else {
        return false;
    };
    if !(1024..=65535).contains(&port) {
        return false;
    }
    !(token.len() == 4 && (1900..=2100).contains(&port))
}

fn find_route_path_contamination(text: &str) -> Option<String> {
    for (idx, ch) in text.char_indices() {
        if ch != '/' {
            continue;
        }
        if idx > 0 {
            let prev = text[..idx].chars().last();
            if prev.is_some_and(|p| p.is_ascii_alphanumeric()) {
                continue;
            }
        }
        let rest = &text[idx..];
        let end = rest
            .find(|ch: char| ch.is_whitespace() || ch == ',' || ch == ';' || ch == ')')
            .unwrap_or(rest.len());
        let path = rest[..end].to_string();
        if path.len() > 1 && !is_host_filesystem_path(&path) {
            return Some(path);
        }
    }
    None
}

fn is_host_filesystem_path(path: &str) -> bool {
    path.starts_with("/etc/")
        || path.starts_with("/home/")
        || path.starts_with("/usr/")
        || path.starts_with("/root/")
        || path.starts_with("/dev/")
        || path.starts_with("/var/")
}

fn find_multi_token_service_id(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    for id in MULTI_TOKEN_SERVICE_IDS {
        if let Some(idx) = lower.find(id) {
            let before_ok = idx == 0 || !is_service_id_char(lower.as_bytes()[idx - 1]);
            let end = idx + id.len();
            let after_ok = end == lower.len() || !is_service_id_char(lower.as_bytes()[end]);
            if before_ok && after_ok {
                return Some(id.to_string());
            }
        }
    }
    None
}

fn is_service_id_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn grounded_text<T>(grounded: &Grounded<T>) -> &str
where
    T: AsRef<str>,
{
    match grounded {
        Grounded::Known { value, .. } => value.as_ref(),
        Grounded::Unknown { reason } => reason,
    }
}

fn system_requirement_id(capability: &DeclaredCapability) -> RequirementId {
    let domain = domain_of(capability.aggregate);
    RequirementId(format!(
        "REQ/{}/{}/SYS/{}",
        domain.as_str(),
        capability.aggregate.as_str(),
        capability.id.as_str()
    ))
}

fn functional_requirement_id_for_function(
    domain: Domain,
    aggregate: Aggregate,
    function_id: &ActionId,
) -> RequirementId {
    RequirementId(format!(
        "REQ/{}/{}/FUN/{}",
        domain.as_str(),
        aggregate.as_str(),
        function_id.as_str()
    ))
}

fn aggregate_for_capability(capability: CapabilityId) -> Aggregate {
    if let Some(def) = capability_def(capability) {
        return def.aggregate;
    }
    if let Some(def) = frontend_capability_def(capability) {
        return def.aggregate;
    }
    Aggregate::HostControl
}

fn route_signature_key(catalog: &Catalog, route: &RouteRef) -> Option<String> {
    resolve_http_path(catalog, route)?;
    Some(format!(
        "{}:{}:{}:{}",
        route.service.as_str(),
        http_method_label(&route.method),
        route.path,
        route.version.unwrap_or("")
    ))
}

fn function_contains_route_signature(
    catalog: &Catalog,
    function: &Action,
    route: &RouteRef,
) -> bool {
    let Some(target) = route_signature_key(catalog, route) else {
        return false;
    };
    for journey_id in &function.verifying_journeys {
        let journey = catalog
            .journey_by_id(journey_id)
            .expect("verifying journey exists");
        let GroundedSet::Known { items } = &journey.steps else {
            continue;
        };
        for step in items.iter() {
            let Some(Grounded::Known {
                value: step_route, ..
            }) = &step.value.route
            else {
                continue;
            };
            if route_signature_key(catalog, step_route).as_deref() == Some(target.as_str()) {
                return true;
            }
        }
    }
    false
}

fn resolve_function_id_for_interface(
    catalog: &Catalog,
    functions: &ActionCatalog,
    journey: &UseCase,
    route: &RouteRef,
) -> Option<ActionId> {
    let mut candidates: Vec<ActionId> = functions
        .functions()
        .iter()
        .filter(|function| function_contains_route_signature(catalog, function, route))
        .filter(|function| function.verifying_journeys.contains(&journey.id))
        .map(|function| function.id.clone())
        .collect();
    if candidates.is_empty() {
        candidates = functions
            .functions()
            .iter()
            .filter(|function| function_contains_route_signature(catalog, function, route))
            .map(|function| function.id.clone())
            .collect();
    }
    candidates.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    candidates.into_iter().next()
}

fn interface_requirement_id(
    domain: Domain,
    aggregate: Aggregate,
    method: &HttpMethod,
    resolved: &str,
) -> RequirementId {
    RequirementId(format!(
        "REQ/{}/{}/IF/{}/{}",
        domain.as_str(),
        aggregate.as_str(),
        http_method_label(method),
        sanitize_path_for_id(resolved)
    ))
}

fn performance_requirement_id(
    domain: Domain,
    aggregate: Aggregate,
    method: &HttpMethod,
    resolved: &str,
) -> RequirementId {
    RequirementId(format!(
        "REQ/{}/{}/PERF/{}/{}",
        domain.as_str(),
        aggregate.as_str(),
        http_method_label(method),
        sanitize_path_for_id(resolved)
    ))
}

fn robustness_requirement_id(
    domain: Domain,
    aggregate: Aggregate,
    probe_id: &str,
) -> RequirementId {
    RequirementId(format!(
        "REQ/{}/{}/ROB/{}",
        domain.as_str(),
        aggregate.as_str(),
        probe_id
    ))
}

fn sanitize_path_for_id(path: &str) -> String {
    path.trim_start_matches('/')
        .chars()
        .map(|ch| match ch {
            '/' | '?' | '&' | '=' => '_',
            other => other,
        })
        .collect()
}

fn primary_capability_for_journey(journey: &UseCase) -> Option<CapabilityId> {
    capability_attributions_for_journey(journey)
        .into_iter()
        .next()
}

fn capability_attributions_for_journey(journey: &UseCase) -> Vec<CapabilityId> {
    let GroundedSet::Known { items } = &journey.capability_refs else {
        return Vec::new();
    };
    if items.is_empty() {
        return Vec::new();
    }
    if items.len() == 1 {
        let capability = items[0].value;
        if journey_summary_fits_capability(journey, capability) {
            return vec![capability];
        }
        return Vec::new();
    }
    let Some(primary) = select_primary_capability(journey, items) else {
        return Vec::new();
    };
    if journey_summary_fits_capability(journey, primary) {
        vec![primary]
    } else {
        Vec::new()
    }
}

fn select_primary_capability(
    journey: &UseCase,
    items: &[catalog_kernel::provenance::GroundedItem<CapabilityId>],
) -> Option<CapabilityId> {
    let summary = grounded_text(&journey.summary).to_ascii_lowercase();
    items
        .iter()
        .map(|item| item.value)
        .max_by_key(|capability| capability_summary_overlap(*capability, &summary))
        .filter(|capability| capability_summary_overlap(*capability, &summary) > 0)
}

fn capability_summary_overlap(capability: CapabilityId, summary: &str) -> usize {
    let id = capability.as_str();
    let mut score = 0usize;
    for token in id.split('_') {
        if token.len() >= 4 && summary.contains(token) {
            score += 1;
        }
    }
    if summary.contains("firmware") && id.contains("firmware") {
        score += 2;
    }
    if summary.contains("install") && id.contains("flash") {
        score += 2;
    }
    score
}

fn journey_summary_fits_capability(journey: &UseCase, capability: CapabilityId) -> bool {
    let summary = grounded_text(&journey.summary).to_ascii_lowercase();
    match capability {
        CapabilityId::AdvertiseMdnsDomains => {
            summary.contains("mdns")
                || summary.contains("advertis")
                || summary.contains("hostname")
                || summary.contains("discover")
        }
        CapabilityId::DetectFlightControllers => {
            summary.contains("detect")
                || summary.contains("discover")
                || summary.contains("connected board")
        }
        _ => capability_summary_overlap(capability, &summary) > 0,
    }
}

fn compose_system_criteria(
    subrequirements: &[RequirementId],
    requirements: &[Requirement],
) -> RequirementCriteria {
    let mut items = Vec::new();
    for child_id in subrequirements {
        let child = requirements
            .iter()
            .find(|req| &req.id == child_id)
            .expect("functional child requirement");
        match &child.criteria {
            RequirementCriteria::Known { items: child_items } => {
                items.extend(child_items.iter().cloned());
            }
            RequirementCriteria::Unknown { reason } => {
                return RequirementCriteria::Unknown {
                    reason: format!(
                        "functional child {} has unknown criteria: {}",
                        child_id.0, reason
                    ),
                };
            }
        }
    }
    if items.is_empty() {
        RequirementCriteria::Unknown {
            reason: "functional children have no acceptance criteria".to_string(),
        }
    } else {
        RequirementCriteria::Known { items }
    }
}

fn append_overlay_requirements(
    catalog: &Catalog,
    requirements: &mut Vec<Requirement>,
    contamination_findings: &mut Vec<ContaminationFinding>,
) {
    for entry in SYSTEM_OVERLAY_ENTRIES {
        let domain = domain_of(entry.aggregate);
        let id = RequirementId(format!(
            "REQ/{}/{}/SYS-OVR/{}",
            domain.as_str(),
            entry.aggregate.as_str(),
            entry.suffix
        ));
        let statement = derive_statement(
            entry.statement,
            ContaminationSource::Overlay {
                suffix: entry.suffix.to_string(),
            },
            contamination_findings,
        );
        let criteria = RequirementCriteria::Known {
            items: entry
                .criteria
                .iter()
                .map(|text| AcceptanceCriterion {
                    text: (*text).to_string(),
                })
                .collect(),
        };
        let rationale = match entry.provenance {
            Provenance::Asserted { rationale } => Some(rationale.to_string()),
            _ => None,
        };
        requirements.push(Requirement {
            id,
            kind: RequirementClass::System,
            domain,
            aggregate: entry.aggregate,
            availability: overlay_availability_from_ids(catalog, entry.journey_ids),
            statement,
            criteria,
            feature_id: None,
            journey_id: None,
            function_id: None,
            requirement_verifications: Vec::new(),
            subrequirements: Vec::new(),
            assumptions: Vec::new(),
            rationale,
        });
    }
}

pub(crate) fn overlay_availability_from_ids(
    catalog: &Catalog,
    journey_ids: &[JourneyId],
) -> Availability {
    let journeys: Vec<&UseCase> = journey_ids
        .iter()
        .map(|id| {
            catalog
                .journey_by_id(id)
                .unwrap_or_else(|| panic!("overlay cites unknown journey {id}"))
        })
        .collect();
    merge_journey_availabilities(journeys)
}

fn capability_phrase_for_service(catalog: &Catalog, service_id: ServiceId) -> Option<String> {
    let service = catalog.service_by_id(&service_id)?;
    match &service.definition.capabilities {
        catalog_kernel::provenance::AssertedSet::Established { items } => items
            .first()
            .map(|item| item.value.as_str().replace('_', " ")),
        catalog_kernel::provenance::AssertedSet::Unknown { .. } => None,
    }
}

fn aggregate_for_journey(journey: &UseCase) -> Aggregate {
    if let GroundedSet::Known { items } = &journey.capability_refs {
        if let Some(first) = items.first() {
            if let Some(def) = capability_def(first.value) {
                return def.aggregate;
            }
            if let Some(def) = frontend_capability_def(first.value) {
                return def.aggregate;
            }
        }
    }
    Aggregate::HostControl
}

fn aggregate_for_service(catalog: &Catalog, service_id: ServiceId) -> Aggregate {
    if let Some(service) = catalog.service_by_id(&service_id) {
        if let catalog_kernel::provenance::AssertedSet::Established { items } =
            &service.definition.capabilities
        {
            if let Some(first) = items.first() {
                if let Some(def) = capability_def(first.value) {
                    return def.aggregate;
                }
            }
        }
    }
    Aggregate::HostControl
}

fn availability_for_capability(catalog: &Catalog, capability: CapabilityId) -> Availability {
    let journeys: Vec<&UseCase> = catalog
        .journeys()
        .iter()
        .filter(|journey| journey_references_capability(journey, capability))
        .collect();
    merge_journey_availabilities(journeys)
}

fn availability_for_service(catalog: &Catalog, service_id: ServiceId) -> Availability {
    let journeys: Vec<&UseCase> = catalog
        .journeys()
        .iter()
        .filter(|journey| journey_participates_service(journey, service_id))
        .collect();
    merge_journey_availabilities(journeys)
}

fn merge_journey_availabilities(journeys: Vec<&UseCase>) -> Availability {
    if journeys.is_empty() {
        return Availability::unknown();
    }
    let anchor = journeys
        .iter()
        .min_by(|left, right| {
            availability_first_tag_key(&left.availability)
                .cmp(&availability_first_tag_key(&right.availability))
                .then_with(|| {
                    left.availability
                        .intro_commit
                        .cmp(right.availability.intro_commit)
                })
        })
        .expect("non-empty journeys");
    Availability {
        intro_commit: anchor.availability.intro_commit,
        present_in_tags: anchor.availability.present_in_tags,
        present_on_master: journeys
            .iter()
            .any(|journey| journey.availability.present_on_master),
        present_on_1_4_dev: journeys
            .iter()
            .any(|journey| journey.availability.present_on_1_4_dev),
    }
}

fn availability_first_tag_key(availability: &Availability) -> (u32, u32, u32, u32) {
    let Some(tag) = availability.first_tag() else {
        return (u32::MAX, u32::MAX, u32::MAX, u32::MAX);
    };
    let base = tag.strip_prefix('v').unwrap_or(tag);
    let mut parts = base.split('.');
    let major = parts
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(u32::MAX);
    let minor = parts
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(u32::MAX);
    let patch = parts
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(u32::MAX);
    (major, minor, patch, 0)
}

fn journey_references_capability(journey: &UseCase, capability: CapabilityId) -> bool {
    matches!(
        &journey.capability_refs,
        GroundedSet::Known { items } if items.iter().any(|item| item.value == capability)
    )
}

fn journey_participates_service(journey: &UseCase, service_id: ServiceId) -> bool {
    matches!(
        &journey.services,
        GroundedSet::Known { items } if items.iter().any(|item| item.value == service_id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::{ActionCatalog, ACTION_COUNT};
    use crate::system_overlay::SYSTEM_OVERLAY_ENTRIES;
    use catalog_core::catalog::Catalog;
    use catalog_kernel::id::journey::JourneyId;
    use catalog_kernel::version::Availability;

    const EXPECTED_DERIVED_SYSTEM_COUNT: usize = 152;
    const EXPECTED_OVERLAY_SYSTEM_COUNT: usize = 4;

    const EXPECTED_SYSTEM_COUNT: usize = 156;
    const EXPECTED_FUNCTIONAL_COUNT: usize = ACTION_COUNT;
    const EXPECTED_INTERFACE_COUNT: usize = 91;
    const EXPECTED_PERFORMANCE_COUNT: usize = 80;
    const EXPECTED_ROBUSTNESS_COUNT: usize = 63;

    const TEST_PRESENCE: Availability = Availability {
        intro_commit: "0000000000000000000000000000000000000001",
        present_in_tags: &["1.0.0"],
        present_on_master: true,
        present_on_1_4_dev: true,
    };

    #[test]
    fn derived_and_overlay_system_counts_match_pins() {
        let catalog = RequirementCatalog::bootstrap();
        let derived = catalog
            .requirements()
            .iter()
            .filter(|req| req.kind == RequirementClass::System && !req.id.0.contains("/SYS-OVR/"))
            .count();
        let overlay = catalog
            .requirements()
            .iter()
            .filter(|req| req.id.0.contains("/SYS-OVR/"))
            .count();
        assert_eq!(derived, EXPECTED_DERIVED_SYSTEM_COUNT);
        assert_eq!(overlay, EXPECTED_OVERLAY_SYSTEM_COUNT);
        assert_eq!(overlay, SYSTEM_OVERLAY_ENTRIES.len());
    }

    #[test]
    fn bootstrap_requirement_counts_match_pins() {
        let catalog = RequirementCatalog::bootstrap();
        let counts = catalog.count_by_kind();
        assert_eq!(
            counts.get(&RequirementClass::System),
            Some(&EXPECTED_SYSTEM_COUNT)
        );
        assert_eq!(
            counts.get(&RequirementClass::Functional),
            Some(&EXPECTED_FUNCTIONAL_COUNT)
        );
        assert_eq!(
            counts.get(&RequirementClass::Interface),
            Some(&EXPECTED_INTERFACE_COUNT)
        );
        assert_eq!(
            counts.get(&RequirementClass::Performance),
            Some(&EXPECTED_PERFORMANCE_COUNT)
        );
        assert_eq!(
            counts.get(&RequirementClass::Robustness),
            Some(&EXPECTED_ROBUSTNESS_COUNT)
        );
    }

    #[test]
    fn assumptions_replace_constraint_requirements() {
        let catalog = Catalog::bootstrap();
        let requirements = RequirementCatalog::from_catalog(&catalog);
        assert!(
            requirements
                .requirements()
                .iter()
                .all(|req| !req.id.0.contains("/CON/")),
            "journey preconditions must not be emitted as CON requirement rows"
        );

        let unique_preconditions: BTreeSet<String> = catalog
            .journeys()
            .iter()
            .flat_map(|journey| {
                journey_requirements(journey)
                    .into_iter()
                    .map(precondition_label)
            })
            .collect();
        let covered: BTreeSet<String> = requirements
            .requirements()
            .iter()
            .flat_map(|req| req.assumptions.iter())
            .map(|assumption| {
                let journey_id = JourneyId::from_str_id(&assumption.source_use_case)
                    .unwrap_or_else(|| {
                        panic!("unknown assumption journey {}", assumption.source_use_case)
                    });
                catalog
                    .journey_by_id(&journey_id)
                    .and_then(|journey| {
                        journey_requirements(journey)
                            .into_iter()
                            .find_map(|precondition| {
                                let statement = precondition_statement(precondition);
                                if statement == assumption.statement {
                                    Some(precondition_label(precondition))
                                } else {
                                    None
                                }
                            })
                    })
                    .expect("assumption maps to a journey precondition")
            })
            .collect();
        assert_eq!(covered.len(), unique_preconditions.len());
        assert_eq!(covered, unique_preconditions);
    }

    #[test]
    fn independent_functional_count_from_function_catalog() {
        let catalog = Catalog::bootstrap();
        let functions = ActionCatalog::from_catalog(&catalog);
        let requirements = RequirementCatalog::from_catalog(&catalog);
        let fun_count = requirements
            .requirements()
            .iter()
            .filter(|req| req.kind == RequirementClass::Functional)
            .count();
        assert_eq!(catalog.journeys().len(), 109);
        assert_eq!(functions.functions().len(), EXPECTED_FUNCTIONAL_COUNT);
        assert_eq!(fun_count, EXPECTED_FUNCTIONAL_COUNT);
    }

    #[test]
    fn bootstrap_fun_count_differs_from_journey_count() {
        let catalog = Catalog::bootstrap();
        let functions = ActionCatalog::from_catalog(&catalog);
        assert_ne!(functions.functions().len(), catalog.journeys().len());
    }

    #[test]
    fn empty_verifying_journeys_fails_structure_even_when_unknown() {
        let catalog = Catalog::bootstrap();
        let mut requirements = RequirementCatalog::from_catalog(&catalog);
        let fun = requirements
            .requirements
            .iter_mut()
            .find(|req| req.kind == RequirementClass::Functional)
            .expect("functional requirement");
        fun.requirement_verifications.clear();
        fun.statement = RequirementStatement::Unknown {
            reason: "no verifying journey summaries".to_string(),
        };
        fun.criteria = RequirementCriteria::Unknown {
            reason: "test".to_string(),
        };
        let errors = requirements
            .validate_structure(&catalog)
            .expect_err("empty verifying journeys must fail");
        assert!(errors.iter().any(|err| matches!(
            err,
            RequirementValidationError::FunctionalWithoutCriterion { .. }
        )));
    }

    #[test]
    fn independent_robustness_count_from_negative_probes() {
        assert_eq!(NEGATIVE_PROBES.len(), EXPECTED_ROBUSTNESS_COUNT);
    }

    #[test]
    fn independent_performance_count_from_slo_baselines() {
        let catalog = Catalog::bootstrap();
        let mut count = 0usize;
        for service in catalog.services() {
            if let GroundedSet::Known { items } = &service.runtime.slo_baselines {
                count += items.len();
            }
        }
        assert_eq!(count, EXPECTED_PERFORMANCE_COUNT);
    }

    #[test]
    fn bootstrap_requirements_validate_structure() {
        let catalog = Catalog::bootstrap();
        let requirements = RequirementCatalog::from_catalog(&catalog);
        requirements
            .validate_structure(&catalog)
            .expect("bootstrap requirements should validate");
    }

    #[test]
    fn contamination_lint_catches_impl_notes_and_spares_english() {
        assert_eq!(
            find_contamination("POST /vehicle_name"),
            Some("POST".to_string())
        );
        assert_eq!(
            find_contamination("listen on port 8000"),
            Some("8000".to_string())
        );
        assert!(find_contamination("Connect BlueOS to a wifi network").is_none());
        assert!(find_contamination("Ping family sonar devices").is_none());
        assert!(find_contamination("Delete a recording").is_none());
        assert!(find_contamination("Get the disk usage tree").is_none());
        assert_eq!(
            find_contamination("reads from mavlink2rest and nginx on port 80"),
            Some("mavlink2rest".to_string())
        );
        assert_eq!(
            find_contamination("Run an arbitrary bash command on the host through commander"),
            Some("commander".to_string())
        );

        let contaminated = "POST /vehicle_name";
        let statement = derive_statement(
            contaminated,
            ContaminationSource::Journey {
                id: JourneyId::MonitorInternetConnectivity,
            },
            &mut Vec::new(),
        );
        assert!(matches!(statement, RequirementStatement::Unknown { .. }));
        let catalog = RequirementCatalog {
            requirements: vec![Requirement {
                id: RequirementId("REQ/test".to_string()),
                kind: RequirementClass::Functional,
                domain: Domain::OnboardComputer,
                aggregate: Aggregate::HostControl,
                availability: TEST_PRESENCE,
                statement: RequirementStatement::Known {
                    text: contaminated.to_string(),
                },
                criteria: RequirementCriteria::Unknown {
                    reason: "test".to_string(),
                },
                feature_id: None,
                journey_id: None,
                function_id: None,
                requirement_verifications: Vec::new(),
                subrequirements: Vec::new(),
                assumptions: Vec::new(),
                rationale: None,
            }],
            contamination_findings: Vec::new(),
        };
        let errors = catalog
            .validate_emitted_statements()
            .expect_err("contaminated emitted statement must fail");
        assert!(errors.iter().any(|err| matches!(
            err,
            RequirementValidationError::ContaminatedStatementEmitted { .. }
        )));
    }

    fn functional_id_for_journey(catalog: &Catalog, journey_id: JourneyId) -> String {
        RequirementCatalog::from_catalog(catalog)
            .requirements()
            .iter()
            .filter(|req| req.kind == RequirementClass::Functional)
            .filter(|req| req.requirement_verifications.contains(&journey_id))
            .min_by_key(|req| req.id.0.as_str())
            .expect("functional requirement for journey")
            .id
            .0
            .clone()
    }

    #[test]
    fn requirement_ids_stable_when_journey_slice_reordered() {
        let catalog_a = Catalog::bootstrap();
        let journeys_a = catalog_a.journeys().to_vec();
        let mut journeys_b = journeys_a.clone();
        if journeys_b.len() >= 2 {
            journeys_b.swap(0, 1);
        }
        let catalog_b = Catalog::with_parts(
            catalog_a.services().to_vec(),
            journeys_b,
            catalog_a.pages().to_vec(),
        );
        let anchor = journeys_a[0].id;
        assert_eq!(
            functional_id_for_journey(&catalog_a, anchor),
            functional_id_for_journey(&catalog_b, anchor)
        );

        for catalog in [&catalog_a, &catalog_b] {
            let requirements = RequirementCatalog::from_catalog(catalog);
            let functional_ids: Vec<&str> = requirements
                .requirements()
                .iter()
                .filter(|req| req.kind == RequirementClass::Functional)
                .map(|req| req.id.0.as_str())
                .collect();
            let unique: HashSet<&str> = functional_ids.iter().copied().collect();
            assert_eq!(
                functional_ids.len(),
                unique.len(),
                "functional requirement ids must be unique"
            );
        }

        let ids_a: BTreeSet<String> = RequirementCatalog::from_catalog(&catalog_a)
            .requirements()
            .iter()
            .map(|req| req.id.0.clone())
            .collect();
        let ids_b: BTreeSet<String> = RequirementCatalog::from_catalog(&catalog_b)
            .requirements()
            .iter()
            .map(|req| req.id.0.clone())
            .collect();
        assert_eq!(ids_a, ids_b);
    }

    #[test]
    fn rtm_completeness_fails_when_real_traces_dropped() {
        use crate::requirements_report::validate_rtm_completeness;

        let catalog = Catalog::bootstrap();
        let mut requirements = RequirementCatalog::from_catalog(&catalog);
        let mut dropped = 0usize;
        for requirement in &mut requirements.requirements {
            if requirement.journey_id.is_some()
                && matches!(requirement.statement, RequirementStatement::Known { .. })
            {
                requirement.journey_id = None;
                requirement.feature_id = None;
                requirement.function_id = None;
                requirement.requirement_verifications.clear();
                dropped += 1;
            } else if requirement.kind == RequirementClass::Functional
                && matches!(requirement.statement, RequirementStatement::Known { .. })
            {
                requirement.function_id = None;
                requirement.requirement_verifications.clear();
                dropped += 1;
            }
        }
        assert!(dropped > 100, "must drop real traces on many requirements");
        let errors = validate_rtm_completeness(&catalog, &requirements)
            .expect_err("dropping journey traces must fail RTM completeness");
        assert!(
            errors.len() >= 100,
            "expected many unresolved RTM rows, got {}",
            errors.len()
        );
        assert!(
            errors[0].contains("lacks a resolvable RTM citation"),
            "{}",
            errors[0]
        );
    }

    #[test]
    fn overlay_availability_rejects_unrelated_journey() {
        let catalog = Catalog::bootstrap();
        let disk = overlay_availability_from_ids(&catalog, &[JourneyId::InspectDiskUsage]);
        let discover =
            overlay_availability_from_ids(&catalog, &[JourneyId::DiscoverBlueosOnNetwork]);
        assert_ne!(
            disk.present_on_dut("1.4-dev"),
            discover.present_on_dut("1.4-dev"),
            "InspectDiskUsage and DiscoverBlueosOnNetwork must differ on 1.4-dev"
        );
        let requirements = RequirementCatalog::from_catalog(&catalog);
        let storage = requirements
            .requirements()
            .iter()
            .find(|req| req.id.0.ends_with("/storage_pressure_visibility"))
            .expect("storage overlay");
        assert_eq!(
            storage.availability.present_on_dut("1.4-dev"),
            disk.present_on_dut("1.4-dev")
        );
        assert_ne!(
            overlay_availability_from_ids(&catalog, &[JourneyId::InspectDiskUsage]),
            overlay_availability_from_ids(&catalog, &[JourneyId::DiscoverBlueosOnNetwork])
        );
    }

    #[test]
    fn overlay_contamination_injected_route_fails_validation() {
        let catalog = Catalog::bootstrap();
        let mut requirements = RequirementCatalog::from_catalog(&catalog);
        let idx = requirements
            .requirements
            .iter()
            .position(|req| req.id.0.contains("/SYS-OVR/"))
            .expect("overlay requirement");
        requirements.requirements[idx].statement = RequirementStatement::Known {
            text: "GET /injected overlay route".to_string(),
        };
        let errors = requirements
            .validate_emitted_statements()
            .expect_err("contaminated overlay");
        assert!(errors.iter().any(|err| matches!(
            err,
            RequirementValidationError::ContaminatedStatementEmitted { .. }
        )));
        let _ = catalog;
    }

    #[test]
    fn missing_system_requirement_fails_validate() {
        let catalog = Catalog::bootstrap();
        let mut requirements = RequirementCatalog::from_catalog(&catalog);
        requirements
            .requirements
            .retain(|req| req.kind != RequirementClass::System);
        let errors = requirements
            .validate_structure(&catalog)
            .expect_err("dropping system requirements must fail");
        assert!(errors.iter().any(|err| matches!(
            err,
            RequirementValidationError::MissingSystemRequirement { .. }
        )));
    }
}
