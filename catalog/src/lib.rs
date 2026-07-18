pub mod capability;
pub mod catalog;
pub mod cluster;
pub mod coverage;
pub mod criticality;
pub mod domain;
pub mod drift;
pub mod edge;
pub mod export;
pub mod extract;
pub mod feature;
pub mod fixture;
pub mod frontend_routes;
pub mod id;
pub mod interface;
pub mod journey;
pub mod journey_group;
pub mod journeys;
pub mod lifecycle;
pub mod observed;
pub mod page;
pub mod pages;
pub mod provenance;
pub mod resolve;
pub mod resource;
pub mod runner;
pub mod runtime;
pub mod service;
pub mod services;
pub mod split;
pub mod state;
pub mod trust;
pub mod validate;

pub use capability::{Aggregate, CapabilityDef, CAPABILITIES};
pub use catalog::{Catalog, CouplingMatrix};
pub use cluster::{
    ClusterPolicy, ClusterResult, CouplingWeights, LensPartition, PairAgreement, SplitConsensus,
    StabilityReport, WEIGHTS_VERSION,
};
pub use coverage::{
    clean_tracker_task, coverage_report, precondition_label, AutomatableCounts, CoverageKind,
    CoverageReport, JourneyCoverage, KindCounts, MappedTaskCoverage, TrackerMapping,
    TrackerTaskRef, TRACKER_CSV_PATH, TRACKER_MAPPINGS,
};
pub use criticality::CriticalityTier;
pub use domain::{domain_of, Domain, DomainDef, DOMAINS};
pub use drift::{
    diff, diff_catalog, diff_catalog_runtime, diff_runtime, DriftFinding, DriftReport,
};
pub use edge::{Bus, Edge, FailureImpact, SyncMode};
pub use export::{export_json, export_mermaid, export_proposals_json, export_schema};
pub use extract::{check_against_observed, extract_from_repo, ExtractedService};
pub use feature::{
    AggregateGroup, Divergence, Feature, FeatureCatalog, FeatureCommunity, FeatureId, FeatureLens,
    FeaturePairAgreement, FeatureSplitConsensus, JourneyView, Origin,
};
pub use fixture::{
    evaluate_journey, evaluate_precondition, journey_fixtures_ready, parse_fixture_list,
    FixtureInventory, PreconditionStatus,
};
pub use id::{CapabilityId, Entity, JourneyId, PathRef, Port, PortRef, ServiceId};
pub use interface::{FileAccessMode, Interface, MavlinkRole};
pub use journey::{
    derive_automatable, journey_requirements, precondition_is_typed, Actor, Automatable, BoardKind,
    DataRequirement, HardwareRequirement, HttpMethod, JourneyStep, NetworkResource, NetworkState,
    Precondition, RouteRef, SoftwareRequirement, StateTransition, StepOutcome, UserJourney,
    Visibility,
};
pub use journey_group::{JourneyLens, JourneyPairAgreement, JourneySplitConsensus};
pub use lifecycle::{Lifecycle, ObservedLifecycle};
pub use observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
pub use page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
pub use provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, Grounded, GroundedItem, GroundedSet, Observed,
    ObservedSet, Provenance, Rationaled,
};
pub use resolve::{resolve, resolve_port_ref, resolve_service_ports, ResolveError, ResolvedPorts};
pub use resource::{Resource, ResourceOwnership};
pub use runner::{
    evaluate_http_response, execute_curl, format_dry_run, format_http_fail, http_journeys,
    http_method_label, http_smoke_steps, http_steps, join_url, journey_http_requires_base,
    resolve_http_path, run_http_step, summarize_journey, JourneyResult, RunCounts, RunnableStep,
    StepResult, SMOKE_DEFAULT_FIXTURES,
};
pub use runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SettingsMutation, SloBaseline,
    StateContract,
};
pub use service::{Authority, Service, ServiceDefinition};
pub use state::StateMachine;
pub use trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};
pub use validate::{validate, ValidationError, COVERAGE_UNKNOWN_THRESHOLD};
