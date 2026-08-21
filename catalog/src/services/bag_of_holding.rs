use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, Service, ServiceDefinition};
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::BagOfHolding,
        state_contracts: GroundedSet::unknown(
            "bag_of_holding has no service-level state machine (card states Unknown); stateless request handlers over a JSON file",
        ),
        slo_baselines: GroundedSet::known(&[runtime_slo(
            HttpMethod::Get,
            "/get/*",
            7.0,
            11.9,
            12.2,
            40,
        )]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.49,
                median: 0.00,
                p95: 1.04,
                min: 0.00,
                max: 4.15,
                sd: 0.90,
            },
            Distribution {
                mean: 35.8,
                median: 35.8,
                p95: 35.8,
                min: 35.8,
                max: 35.8,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "bag_of_holding is a generic JSON store independent of the flight controller; platform-independent",
                    "runtime captured on Navigator only; RSS ~35.8 MB flat (lightest Python service captured), CPU ~0.49% mean",
                ],
            },
            runtime_prov("runtime-captures/bag_of_holding__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /set/{path} merges a value and POST /overwrite replaces the entire db.json; not exercised (shared-store mutation)",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::BagOfHolding,
        method,
        path,
        version: None,
    }
}

const fn runtime_slo(
    method: HttpMethod,
    path: &'static str,
    p50: f64,
    p95: f64,
    p99: f64,
    sample_size: u32,
) -> GroundedItem<SloBaseline> {
    GroundedItem::new(
        SloBaseline {
            route: runtime_route(method, path),
            latency_p50_ms: p50,
            latency_p95_ms: p95,
            latency_p99_ms: p99,
            sample_size,
        },
        runtime_prov(
            "runtime-captures/bag_of_holding__pi4_navigator_master.json#slo_running_baseline",
        ),
    )
}

const fn runtime_resource(
    condition: &'static str,
    cpu_pct: Distribution,
    rss_mb: Distribution,
    samples: u32,
) -> GroundedItem<ResourceUsage> {
    GroundedItem::new(
        ResourceUsage {
            condition,
            cpu_pct,
            rss_mb,
            samples,
        },
        runtime_prov("runtime-captures/bag_of_holding__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::BagOfHolding,
    aliases: ObservedSet::known(&[Evidenced::new(
        "bag-of-holding",
        Evidence {
            file: "core/services/bag_of_holding/main.py",
            line: 21,
            anchor: "SERVICE_NAME = \"bag-of-holding\"",
        },
    )]),
    kind: Observed::known(
        ServiceKind::PythonService,
        Evidence {
            file: "core/start-blueos-core",
            line: 144,
            anchor: "'bag_of_holding',250,0,0,0,\"$SERVICES_PATH/bag_of_holding/ma",
        },
    ),
    entrypoint: Observed::known(
        "$SERVICES_PATH/bag_of_holding/main.py",
        Evidence {
            file: "core/start-blueos-core",
            line: 144,
            anchor: "'bag_of_holding',250,0,0,0,\"$SERVICES_PATH/bag_of_holding/ma",
        },
    ),
    tmux_name: Observed::known(
        "bag_of_holding",
        Evidence {
            file: "core/start-blueos-core",
            line: 144,
            anchor: "'bag_of_holding',250,0,0,0,\"$SERVICES_PATH/bag_of_holding/ma",
        },
    ),
    startup_tier: Observed::known(
        StartupTier::Normal,
        Evidence {
            file: "core/start-blueos-core",
            line: 124,
            anchor: "SERVICES=(",
        },
    ),
    resource_limits: Observed::known(
        ResourceLimits {
            memory_mb: Some(250),
            cpu_percent: Some(0),
            io_weight: None,
        },
        Evidence {
            file: "core/start-blueos-core",
            line: 144,
            anchor: "'bag_of_holding',250,0,0,0,\"$SERVICES_PATH/bag_of_holding/ma",
        },
    ),
    nice: Observed::unknown("no nice prefix in start tuple"),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 144,
            anchor: "'bag_of_holding',250,0,0,0,\"$SERVICES_PATH/bag_of_holding/ma",
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/bag/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 86,
            anchor: "location /bag/ {",
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(9101),
        Evidence {
            file: "core/services/bag_of_holding/main.py",
            line: 124,
            anchor: "config = Config(app=app, host=\"0.0.0.0\", port=9101, log_conf",
        },
    )]),
    git_path: Observed::known(
        PathRef("core/services/bag_of_holding"),
        Evidence {
            file: "core/services/bag_of_holding/main.py",
            line: 1,
            anchor: "#! /usr/bin/env python3",
        },
    ),
    interfaces: ObservedSet::known(&[
        Evidenced::new(
            Interface::Rest {
                path_prefix: PathRef("/bag/"),
                port: PortRef::Literal(9101),
                versions: &["v1.0"],
            },
            Evidence {
                file: "core/services/bag_of_holding/main.py",
                line: 105,
                anchor: "app = VersionedFastAPI(app, version=\"1.0.0\", prefix_format=\"",
            },
        ),
        Evidenced::new(
            Interface::File {
                path: PathRef("/root/.config/bag-of-holding"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/bag_of_holding/main.py",
                line: 22,
                anchor: "FILE_PATH = Path(appdirs.user_config_dir(SERVICE_NAME, \"db.j",
            },
        ),
        Evidenced::new(
            Interface::Zenoh {
                topics_produced: &["services/bag-of-holding/log"],
                topics_consumed: &[],
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
                anchor: "topic = f\"services/{service_name}/log\"",
            },
        ),
    ]),
    resources: ObservedSet::known(&[Evidenced::new(
        Resource {
            path: PathRef("/root/.config/bag-of-holding"),
            ownership: ResourceOwnership::SharedWrite,
        },
        Evidence {
            file: "core/services/bag_of_holding/main.py",
            line: 61,
            anchor: "with open(FILE_PATH, \"w\", encoding=\"utf-8\") as f:",
        },
    )]),
    lifecycle: Observed::known(
        ObservedLifecycle {
            triggers: &["start-blueos-core create_service"],
            ordered_after: &[
                ServiceId::ArdupilotManager,
                ServiceId::CableGuy,
                ServiceId::MavlinkCameraManager,
                ServiceId::Mavlink2rest,
                ServiceId::Kraken,
                ServiceId::Wifi,
                ServiceId::Zenohd,
                ServiceId::Beacon,
                ServiceId::Bridget,
                ServiceId::Commander,
                ServiceId::NmeaInjector,
                ServiceId::Helper,
                ServiceId::Iperf3,
                ServiceId::Linux2rest,
                ServiceId::Filebrowser,
                ServiceId::Versionchooser,
                ServiceId::Pardal,
                ServiceId::Ping,
                ServiceId::UserTerminal,
                ServiceId::Ttyd,
                ServiceId::Nginx,
            ],
            ordered_before: &[
                ServiceId::Recorder,
                ServiceId::RecorderExtractor,
                ServiceId::DiskUsage,
                ServiceId::Customization,
            ],
        },
        Evidence {
            file: "core/start-blueos-core",
            line: 326,
            anchor: "for TUPLE in \"${SERVICES[@]}\"; do",
        },
    ),
    logs_path: Observed::unknown(
        "init_logger publishes to zenoh only; no on-disk log path set in bag_of_holding source",
    ),
    zenoh_log_topic: Observed::known(
        "services/bag-of-holding/log",
        Evidence {
            file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
            line: 78,
            anchor: "topic = f\"services/{service_name}/log\"",
        },
    ),
    sentry: Observed::known(
        true,
        Evidence {
            file: "core/services/bag_of_holding/main.py",
            line: 121,
            anchor: "await init_sentry_async(SERVICE_NAME)",
        },
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::BagOfHolding,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one process owns the shared JSON document store",
        ),
        bounded_context: Asserted::established(
            "generic-json-persistence",
            "provisional 2.0 domain: shared key-value JSON store for frontend UI state, wizard progress, and feature tokens",
        ),
        journey_refs: AssertedSet::established(&[Rationaled::new(
            JourneyId::ModifyBagDatabase,
            "Bag Editor loads the full document via GET /get/* and persists edits via POST /overwrite",
        )]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "wizard, settings, cloud tokens, and vehicle image paths persist here; vehicle MAVLink control does not depend on it",
        ),
        offline_required: Asserted::established(
            true,
            "db.json is a local file under /root/.config; set/get/overwrite need no network access",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; sole writer of /root/.config/bag-of-holding/db.json",
        ),
        dangerous_operations: AssertedSet::established(&[Rationaled::new(
            DangerousOperation::Other("overwrite_entire_datastore"),
            "POST /overwrite atomically replaces the entire db.json; a bad payload wipes all stored UI/setup/cloud-token state at once",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "whole-store overwrite is destructive bulk replacement of persisted frontend and setup state",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::EditBagJsonStore,
                "Bag Editor reads GET /get/* and saves the full edited document via POST /overwrite",
            ),
            Rationaled::new(
                CapabilityId::SetBagValue,
                "POST /set/{path} merges a value at a dpath key into the current JSON document",
            ),
            Rationaled::new(
                CapabilityId::GetBagValue,
                "GET /get/{path} returns a subtree or GET /get/* returns the full document",
            ),
            Rationaled::new(
                CapabilityId::OverwriteBagStore,
                "POST /overwrite replaces db.json with the request body JSON object",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("json_document_store"),
            "sole owner and writer of the shared generic JSON persistence file consumed by the frontend and feature stores",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; key-value reads and writes are stateless request handlers",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[Rationaled::new(
            Resource {
                path: PathRef("/root/.config/bag-of-holding"),
                ownership: ResourceOwnership::SharedWrite,
            },
            "config directory containing db.json for wizard state, settings, vehicle images, and Major Tom tokens",
        )]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in SERVICES tier",
            ),
            ordered_after: Asserted::established(
                &[
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                    ServiceId::Kraken,
                    ServiceId::Wifi,
                    ServiceId::Zenohd,
                    ServiceId::Beacon,
                    ServiceId::Bridget,
                    ServiceId::Commander,
                    ServiceId::NmeaInjector,
                    ServiceId::Helper,
                    ServiceId::Iperf3,
                    ServiceId::Linux2rest,
                    ServiceId::Filebrowser,
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                    ServiceId::Ttyd,
                    ServiceId::Nginx,
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[
                    ServiceId::Recorder,
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
                "observed ordered_before lists bag_of_holding before recorder, disk_usage, and customization",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit when main() serve loop returns",
                "no explicit shutdown hook; process stops with tmux session teardown",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight writes and db.json migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns HTML title page",
            "no dedicated /health route; uvicorn availability serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "internal persistence backend for BlueOS UI state; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 set/get routes are stable under /bag/v1.0; legacy unversioned POST /overwrite retained for Bag Editor",
        ),
        permissions_model: Asserted::established(
            "no service-level auth; Bag Editor gated by pirate mode in frontend only",
            "REST endpoints accept any caller reaching nginx; advanced UI access is a frontend precondition",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "database_file_missing",
                "read_db returns empty object when db.json does not exist yet",
            ),
            Rationaled::new(
                "json_decode_error",
                "read_db logs JSONDecodeError and returns empty object when db.json is corrupt",
            ),
            Rationaled::new(
                "invalid_get_path",
                "GET /get/{path} returns 400 when dpath.get finds no key at the requested path",
            ),
        ]),
        blast_radius: Asserted::established(
            "frontend wizard, settings, cloud tokens, and vehicle image paths unavailable; core vehicle services unaffected"
                ,
            "bag_of_holding outage blocks UI persistence but not autopilot, MAVLink, or nginx core paths",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy for unversioned /overwrite vs versioned v1.0 routes not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::BagOfHolding,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
