pub mod api_contract;
pub mod capability;
pub mod capture_env;
pub mod catalog;
pub mod cluster;
pub mod coverage;
pub mod criticality;
pub mod domain;
pub mod drift;
pub mod edge;
pub mod export;
pub mod export_sysml;
pub mod extension_lifecycle;
pub mod extract;
pub mod extract_fastapi;
pub mod extract_frontend_router;
pub mod extract_nginx;
pub mod feature;
pub mod feature_intro;
pub mod feature_trace;
pub mod fixture;
pub mod frontend_cache;
pub mod frontend_routes;
pub mod frontend_smoke;
pub mod function;
pub mod harness_ratchet;
pub mod id;
pub mod interface;
pub mod journey;
pub mod journey_group;
pub mod journey_matrix;
pub mod journey_presence;
pub mod journeys;
pub mod lifecycle;
pub mod mcm_restore;
pub mod mutating_smoke;
pub mod negative_probes;
pub mod observed;
pub mod observed_verification;
pub mod page;
pub mod pages;
pub mod provenance;
pub mod provenance_anchor;
pub mod provenance_walk;
pub mod report;
pub mod requirement;
pub mod requirements_report;
pub mod resolve;
pub mod resource;
pub mod runner;
pub mod runtime;
pub mod service;
pub mod services;
pub mod sitl_cal;
pub mod source_index;
pub mod split;
pub mod state;
pub mod system_overlay;
pub mod tools;
pub mod trust;
pub mod ui;
pub mod validate;
pub mod version;
pub mod wifi_endpoints;
pub mod wifi_rf;

pub use api_contract::{
    api_coverage_counts, api_coverage_report, diff_snapshots, load_api_contract_baseline,
    write_api_contract_baseline, ApiContractDiff, ApiContractFieldChange, ApiContractKey,
    ApiContractSnapshot, ApiCoverageReport, ApiCoverageSource, ApiRouteContract,
    DEFAULT_API_CONTRACT_BASELINE,
};
pub use capability::{Aggregate, CapabilityDef, CAPABILITIES};
pub use capture_env::{
    format_runtime_env, RUNTIME_CAPTURE_CORE_DIGEST, RUNTIME_CAPTURE_CORE_REPO,
    RUNTIME_CAPTURE_CORE_REPO_DIGEST, RUNTIME_CAPTURE_CORE_TAG, RUNTIME_CAPTURE_ENV_PI4,
    RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR, RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_ARDUSUB,
    RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST, RUNTIME_CAPTURE_ENV_PI4_SITL,
    TIER2_SMOKE_DUT_CORE_DIGEST,
};
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
pub use edge::{Bus, Connection, FailureImpact, SyncMode};
pub use export::{export_json, export_mermaid, export_proposals_json, export_schema};
pub use extension_lifecycle::{run_extension_lifecycle, write_extension_lifecycle_report};
pub use extract::{
    check_against_observed, check_nginx_against_observed, extract_from_repo,
    extract_nginx_from_repo, ExtractedNginxLocation, ExtractedService,
};
pub use extract_fastapi::{
    check_fastapi_against_observed, extract_fastapi_from_repo, ExtractedFastApiRoute,
};
pub use extract_frontend_router::{
    check_frontend_router_against_observed, extract_frontend_router_from_repo,
    ExtractedFrontendMenu, ExtractedFrontendRoute,
};
pub use feature::{
    capability_aggregate_view, capability_journey_view, capability_split_consensus,
    capability_view_divergence, declared_capabilities, validate_capabilities, AggregateGroup,
    CapabilityCommunity, CapabilityJourneyView, CapabilityLens, CapabilityPairAgreement,
    CapabilitySplitConsensus, DeclaredCapability, Divergence, Origin,
};
pub use feature_intro::{feature_map_for_version, journeys_for_version, presence_for_journey};
pub use feature_trace::{
    cluster_for_journey, commit, discovery_for_journey, feature_traces, intro_commit_for_journey,
    issue, landing_pr_for_journey, pull_request, ClusterIssueRef, IntroCluster, IntroTraces,
    IssueSource, IssueSourceKind, JourneyDiscovery, TraceCommit, TraceIssue, TraceJourneyRef,
    TracePullRequest,
};
pub use fixture::{
    evaluate_journey, evaluate_precondition, journey_fixtures_ready, journey_mutating_smoke_ready,
    parse_fixture_list, FixtureInventory, PreconditionStatus,
};
pub use frontend_cache::run_frontend_cache;
pub use frontend_smoke::{
    calibration_smoke_targets, concrete_page_path, frontend_smoke_targets, FrontendSmokeTarget,
};
pub use function::{Action, ActionCatalog, ActionId, ACTION_COUNT};
pub use harness_ratchet::{
    compare_harness_ratchet, count_client_orchestrated_missing_ui_plan, count_open_harness_gap,
    count_unknown_body_kinds, count_unprobed_failure_modes, harness_ratchet_counts,
    load_harness_ratchet_baseline, write_harness_ratchet_baseline, HarnessRatchetCounts,
    HarnessRatchetRegression, DEFAULT_BASELINE_PATH, FAILURE_MODE_LEDGER_PATH,
};
pub use id::{CapabilityId, Entity, JourneyId, PathRef, PortRef, ServiceId, TcpPort};
pub use interface::{FileAccessMode, MavlinkRole, PortKind};
pub use journey::{
    blast_radius_is_unknown, derive_automatable, derive_oracle_class, http_automatable,
    journey_requirements, precondition_is_typed, Actor, BlastRadius, BoardKind, BodyKind,
    DataAssumption, HardwareAssumption, HttpMethod, JourneyStep, NetworkResource, NetworkState,
    OracleClass, Precondition, RouteRef, SoftwareAssumption, StateTransition, StepOutcome, UseCase,
    VerificationMethod, Visibility, BLAST_RADIUS_UNKNOWN,
};
pub use journey_group::{JourneyLens, JourneyPairAgreement, JourneySplitConsensus};
pub use journey_matrix::{
    blank_both_violations, build_journey_matrix, format_matrix, has_frontend_step, has_known_route,
    load_report_hits, Cell, JourneyMatrix, JourneyMatrixRow, ReportHit, HARD_EXCLUDED,
    PAGE_LOAD_UI,
};
pub use journey_presence::ALL_JOURNEY_PRESENCE;
pub use lifecycle::{Lifecycle, ObservedLifecycle};
pub use mcm_restore::{McmStreamRestore, McmV4lRestore, SMOKE_CATALOG_STREAM_JSON};
pub use mutating_smoke::{
    effect_read_probe_journey, is_camera_mutating_smoke_journey, is_mutating_smoke_journey,
    is_tier2_mutating_eligible, is_tier2_mutating_hard_excluded, journey_has_mutating_http_route,
    mutating_smoke_journey_ids, tier2_mutating_coverage, EffectReadRef, MutatingSmokeEntry,
    SmokeRepair, Tier2MutatingCoverage, MUTATING_SMOKE_ENTRIES,
};
pub use negative_probes::{
    format_negative_dry_run, negative_probe_url, NegativeProbe, ProbeBlast, ProbeClass,
    NEGATIVE_PROBES,
};
pub use observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
pub use observed_verification::{
    count_extractor_coverage_lapses, count_unverified_observed_evidence,
};
pub use page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
pub use provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, Grounded, GroundedItem, GroundedSet, Observed,
    ObservedSet, Provenance, Rationaled,
};
pub use report::{
    count_journey_steps, utc_rfc3339_now, write_journey_http_report, ConflictKind,
    ReportAvailability, ReportConflict, ReportCounts, ReportDut, ReportTrace, SuiteKind,
    VerificationHttpReport, VerificationRecord, SCHEMA_VERSION,
};
pub use requirement::{
    find_contamination, AcceptanceCriterion, Assumption, AssumptionKind, ContaminationFinding,
    ContaminationSource, Requirement, RequirementCatalog, RequirementClass, RequirementCriteria,
    RequirementId, RequirementStatement, RequirementValidationError,
};
pub use requirements_report::{
    build_rtm_rows, diff_requirement_reports, load_requirements_baseline, overlay_trace_failures,
    render_diff_report, render_rtm_csv, render_srs, requirements_json, validate_rtm_completeness,
    write_requirements_baseline, RequirementDiffReport, RequirementsJson, RtmRow,
};
pub use resolve::{resolve, resolve_port_ref, resolve_service_ports, ResolveError, ResolvedPorts};
pub use resource::{Resource, ResourceOwnership};
pub use runner::{
    dut_profile_for_host, dut_version_current_json, effect_observation_changed, effect_read_after,
    effect_read_after_observed, effect_read_before, effect_read_enabled, evaluate_http_response,
    execute_curl, fetch_dut_version, format_dry_run, format_http_fail, http_journeys,
    http_method_label, http_mutating_smoke_steps, http_smoke_steps, http_steps, join_url,
    journey_availability_skip, journey_http_mode_conflict, journey_http_requires_base,
    journey_profile_skip, mutating_effect_read_phases, mutating_smoke_body,
    mutating_smoke_path_bind, mutating_smoke_setup_calls, mutating_smoke_skip_reason,
    mutating_smoke_teardown_calls, resolve_http_path, run_core_image_switch, run_http_step,
    run_negative_probe, run_smoke_http_call, summarize_journey, tier1_get_coverage,
    wait_for_blueos, DutProfile, DutVersion, EffectReadAfter, EffectReadBefore,
    EffectReadBeforeResult, JourneyResult, RunCounts, RunnableStep, Tier1GetCoverage, Verdict,
    MUTATING_SMOKE_DEFAULT_FIXTURES, SMOKE_CORE_SWITCH_JSON, SMOKE_CORE_SWITCH_TAG,
    SMOKE_DEFAULT_FIXTURES,
};
pub use runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SettingsMutation, SloBaseline,
    StateContract,
};
pub use service::{Authority, Service, ServiceJudgment};
pub use sitl_cal::{needs_calibration_frame, needs_vectored_frame, SitlRc};
pub use source_index::{
    build_source_index, build_source_index_from_walk, catalog_entity_for_citation,
    catalog_module_for_entity, citation_source_path, index_git_diff_pathspecs,
    index_key_to_repo_path, indexable_source_paths, is_confirmed_drift,
    linter_source_paths_from_fields, repo_path_to_index_key, review_urgency,
    work_order_for_changed_paths, CatalogEntity, ReviewUrgency, SourceIndex, SourceIndexEntry,
    WorkOrder, WorkOrderEntry, DOC_SCOPE_NOTE, WORK_ORDER_NOTE,
};
pub use state::StateMachine;
pub use trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};
pub use ui::{
    ui_fixture_skip_reason, ui_plan, ui_suite_plans, ui_typed_skip_reason, wizard_skip_plan,
    UiAction, UiJourneyPlan, UI_CALIBRATION_JOURNEYS, UI_CAMERA_JOURNEYS, UI_EXTENSION_JOURNEYS,
    UI_NO_HARDWARE_JOURNEYS, UI_TYPED_SKIP, UI_VERSION_SETTINGS_JOURNEYS,
};
pub use validate::{validate, ValidationError, COVERAGE_UNKNOWN_THRESHOLD};
pub use version::{
    availability_is_valid, availability_skip, bound_tag, cmp_channels, feature_present_on,
    format_availability_skip_reason, journeys_present_on, parse_release_tag, Availability,
    AvailabilitySkip, BlueOsChannel, VersionBound,
};
pub use wifi_endpoints::run_wifi_endpoints;
