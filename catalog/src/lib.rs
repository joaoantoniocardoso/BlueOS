pub mod catalog;
pub mod cluster;
pub mod criticality;
pub mod drift;
pub mod edge;
pub mod export;
pub mod extract;
pub mod feature;
pub mod id;
pub mod interface;
pub mod journey;
pub mod journeys;
pub mod lifecycle;
pub mod observed;
pub mod page;
pub mod pages;
pub mod provenance;
pub mod resolve;
pub mod resource;
pub mod runtime;
pub mod service;
pub mod services;
pub mod state;
pub mod trust;
pub mod validate;

pub use catalog::{Catalog, CouplingMatrix};
pub use cluster::{
    ClusterPolicy, ClusterResult, CouplingWeights, StabilityReport, WEIGHTS_VERSION,
};
pub use criticality::CriticalityTier;
pub use drift::{
    diff, diff_catalog, diff_catalog_runtime, diff_runtime, DriftFinding, DriftReport,
};
pub use edge::{Bus, Edge, FailureImpact, SyncMode};
pub use export::{export_json, export_mermaid, export_proposals_json, export_schema};
pub use extract::{check_against_observed, extract_from_repo, ExtractedService};
pub use feature::{
    AggregateGroup, Divergence, Feature, FeatureCatalog, FeatureCommunity, FeatureId, JourneyView,
};
pub use id::{CapabilityId, JourneyId, PathRef, Port, PortRef, ServiceId};
pub use interface::{FileAccessMode, Interface, MavlinkRole};
pub use journey::{
    Actor, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef, StateTransition,
    StepOutcome, UserJourney, Visibility,
};
pub use lifecycle::{Lifecycle, ObservedLifecycle};
pub use observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
pub use page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
pub use provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, Grounded, GroundedItem, GroundedSet, Observed,
    ObservedSet, Provenance, Rationaled,
};
pub use resolve::{resolve, resolve_port_ref, resolve_service_ports, ResolveError, ResolvedPorts};
pub use resource::{Resource, ResourceOwnership};
pub use runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SettingsMutation, SloBaseline,
    StateContract,
};
pub use service::{Authority, Service, ServiceDefinition};
pub use state::StateMachine;
pub use trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};
pub use validate::{validate, ValidationError, COVERAGE_UNKNOWN_THRESHOLD};
