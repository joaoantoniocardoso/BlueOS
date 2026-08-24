use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, PortKind};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, Service, ServiceJudgment};
use crate::trust::{PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Customization,
        state_contracts: GroundedSet::unknown(
            "customization has no service-level state machine (card states Unknown); stateless file-backed asset handlers",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/theme", 5.6, 9.4, 9.4, 40),
            runtime_slo(HttpMethod::Get, "/models", 5.3, 7.0, 9.3, 40),
            runtime_slo(HttpMethod::Get, "/branding/logo", 5.5, 8.6, 10.5, 40),
            runtime_slo(HttpMethod::Get, "/branding/vehicle-image", 5.7, 7.5, 10.1, 40),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.26,
                median: 0.00,
                p95: 1.01,
                min: 0.00,
                max: 1.06,
                sd: 0.43,
            },
            Distribution {
                mean: 35.0,
                median: 35.0,
                p95: 35.0,
                min: 35.0,
                max: 35.0,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "customization serves branding/theme assets independent of the flight controller; platform-independent",
                    "runtime captured on Navigator only; RSS ~35.0 MB flat, CPU ~0.26% mean; all GETs ~5-6 ms",
                ],
            },
            runtime_prov("runtime-captures/customization__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "PUT /theme writes theme_config.json + theme_style.css; branding/model uploads write /usr/blueos/userdata/{branding,modeloverrides}; not exercised",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Customization,
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
            "runtime-captures/customization__pi4_navigator_master.json#slo_running_baseline",
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
        runtime_prov("runtime-captures/customization__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::Customization,
    aliases: ObservedSet::known(&[Evidenced::new(
        "customization",
        Evidence {
            file: "core/services/customization/main.py",
            line: 34,
            anchor: "SERVICE_NAME = \"customization\"",
        },
    )]),
    kind: Observed::known(
        ServiceKind::PythonService,
        Evidence {
            file: "core/start-blueos-core",
            line: 148,
            anchor: "'customization',250,0,0,0,\"$SERVICES_PATH/customization/main",
        },
    ),
    entrypoint: Observed::known(
        "$SERVICES_PATH/customization/main.py",
        Evidence {
            file: "core/start-blueos-core",
            line: 148,
            anchor: "'customization',250,0,0,0,\"$SERVICES_PATH/customization/main",
        },
    ),
    tmux_name: Observed::known(
        "customization",
        Evidence {
            file: "core/start-blueos-core",
            line: 148,
            anchor: "'customization',250,0,0,0,\"$SERVICES_PATH/customization/main",
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
            line: 148,
            anchor: "'customization',250,0,0,0,\"$SERVICES_PATH/customization/main",
        },
    ),
    nice: Observed::unknown("no nice prefix in start tuple"),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 148,
            anchor: "'customization',250,0,0,0,\"$SERVICES_PATH/customization/main",
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/customization/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 145,
            anchor: "location /customization/ {",
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(9152),
        Evidence {
            file: "core/services/customization/main.py",
            line: 35,
            anchor: "PORT = 9152",
        },
    )]),
    git_path: Observed::known(
        PathRef("core/services/customization"),
        Evidence {
            file: "core/services/customization/main.py",
            line: 1,
            anchor: "#! /usr/bin/env python3",
        },
    ),
    interfaces: ObservedSet::known(&[
        Evidenced::new(
            PortKind::Rest {
                path_prefix: PathRef("/customization/"),
                port: PortRef::Literal(9152),
                versions: &["v1.0"],
            },
            Evidence {
                file: "core/services/customization/main.py",
                line: 328,
                anchor: "prefix_format=\"/v{major}.{minor}\",",
            },
        ),
        Evidenced::new(
            PortKind::Settings {
                path: PathRef("/usr/blueos/userdata/styles/theme_config.json"),
            },
            Evidence {
                file: "core/services/customization/storage.py",
                line: 10,
                anchor: "THEME_CONFIG_FILE = STYLES_DIR / \"theme_config.json\"",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/usr/blueos/userdata/styles/theme_style.css"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/customization/storage.py",
                line: 9,
                anchor: "THEME_FILE = STYLES_DIR / \"theme_style.css\"",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/usr/blueos/userdata/modeloverrides"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/customization/storage.py",
                line: 6,
                anchor: "MODELS_DIR = USERDATA / \"modeloverrides\"",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/usr/blueos/userdata/branding"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/customization/storage.py",
                line: 7,
                anchor: "BRANDING_DIR = USERDATA / \"branding\"",
            },
        ),
        Evidenced::new(
            PortKind::Zenoh {
                topics_produced: &["services/customization/log"],
                topics_consumed: &[],
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
                anchor: "topic = f\"services/{service_name}/log\"",
            },
        ),
    ]),
    resources: ObservedSet::known(&[
        Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/styles/theme_config.json"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/customization/main.py",
                line: 97,
                anchor: "THEME_CONFIG_FILE.write_text(json.dumps(config.dict(), inden",
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/styles/theme_style.css"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/customization/main.py",
                line: 102,
                anchor: "THEME_FILE.write_text(render_css(primary), encoding=\"utf-8\")",
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/modeloverrides"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/customization/storage.py",
                line: 20,
                anchor: "for directory in (STYLES_DIR, MODELS_DIR, BRANDING_DIR):",
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/branding"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/customization/storage.py",
                line: 20,
                anchor: "for directory in (STYLES_DIR, MODELS_DIR, BRANDING_DIR):",
            },
        ),
    ]),
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
                ServiceId::BagOfHolding,
                ServiceId::Recorder,
                ServiceId::RecorderExtractor,
                ServiceId::DiskUsage,
            ],
            ordered_before: &[],
        },
        Evidence {
            file: "core/start-blueos-core",
            line: 326,
            anchor: "for TUPLE in \"${SERVICES[@]}\"; do",
        },
    ),
    logs_path: Observed::unknown(
        "init_logger publishes to zenoh only; no on-disk log path set in customization source",
    ),
    zenoh_log_topic: Observed::known(
        "services/customization/log",
        Evidence {
            file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
            line: 78,
            anchor: "topic = f\"services/{service_name}/log\"",
        },
    ),
    sentry: Observed::known(
        true,
        Evidence {
            file: "core/services/customization/main.py",
            line: 340,
            anchor: "await init_sentry_async(SERVICE_NAME)",
        },
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::Customization,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one customization process owns userdata branding assets",
        ),
        bounded_context: Asserted::established(
            "ui-branding-and-theming",
            "provisional 2.0 domain: white-label theme color, logo/vehicle-image branding, and 3D model overrides",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::ChangeUiThemeColor,
                "Settings Appearance panel saves a primary color and regenerates theme CSS",
            ),
            Rationaled::new(
                JourneyId::ResetUiThemeColor,
                "Settings Appearance panel resets theme config and restores default BlueOS colors",
            ),
            Rationaled::new(
                JourneyId::UploadCustomLogo,
                "Settings Customization panel uploads a square company logo image",
            ),
            Rationaled::new(
                JourneyId::RemoveCustomLogo,
                "Settings Customization panel removes the uploaded logo and reverts to default branding",
            ),
            Rationaled::new(
                JourneyId::UploadCustomVehicleImage,
                "Settings Customization panel uploads a square vehicle image for the interface",
            ),
            Rationaled::new(
                JourneyId::RemoveCustomVehicleImage,
                "Settings Customization panel removes the uploaded vehicle image asset",
            ),
            Rationaled::new(
                JourneyId::Upload3dModelOverride,
                "Settings Customization panel uploads a .glb model served under userdata/modeloverrides/",
            ),
            Rationaled::new(
                JourneyId::Delete3dModelOverride,
                "Settings Customization panel deletes a model from the override list",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "SERVICES startup tier; vehicle control and core functions work without custom theme, logo, or model assets",
        ),
        offline_required: Asserted::established(
            true,
            "theme, branding, and model overrides persist under local /usr/blueos/userdata without network access",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; writes theme CSS, branding images, and model files under userdata",
        ),
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "dangerous_operations empty per rubric v1.0; theme, branding, and model changes are reversible reconfiguration, not irreversible destructive ops",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::SetThemeColor,
                "PUT /theme saves primary color to theme_config.json and regenerates theme_style.css",
            ),
            Rationaled::new(
                CapabilityId::ResetThemeColor,
                "DELETE /theme removes theme config and restores default primary color CSS",
            ),
            Rationaled::new(
                CapabilityId::GetThemeConfiguration,
                "GET /theme returns current primary color, palette, and css_url for the web UI",
            ),
            Rationaled::new(
                CapabilityId::UploadBrandingLogo,
                "POST /branding/logo stores a custom company logo image under userdata/branding/",
            ),
            Rationaled::new(
                CapabilityId::RemoveBrandingLogo,
                "DELETE /branding/logo removes the custom logo file and reverts to default branding",
            ),
            Rationaled::new(
                CapabilityId::GetBrandingLogo,
                "GET /branding/logo returns the current custom logo URL and size, if any",
            ),
            Rationaled::new(
                CapabilityId::UploadBrandingVehicleImage,
                "POST /branding/vehicle-image stores a custom vehicle image under userdata/branding/",
            ),
            Rationaled::new(
                CapabilityId::RemoveBrandingVehicleImage,
                "DELETE /branding/vehicle-image removes the custom vehicle image asset",
            ),
            Rationaled::new(
                CapabilityId::GetBrandingVehicleImage,
                "GET /branding/vehicle-image returns the current custom vehicle image URL and size, if any",
            ),
            Rationaled::new(
                CapabilityId::UploadModelOverride,
                "POST /models uploads a .glb file into userdata/modeloverrides/",
            ),
            Rationaled::new(
                CapabilityId::DeleteModelOverride,
                "DELETE /models/{name} removes an uploaded model override file",
            ),
            Rationaled::new(
                CapabilityId::ListModelOverrides,
                "GET /models lists uploaded model override entries with URLs and sizes",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("ui_branding_manager"),
            "sole writer of /usr/blueos/userdata styles, branding, and modeloverrides assets served by the customization API; bag_of_holding separately stores sidebar vehicle images in db.json",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; theme, branding, and model handlers are stateless request handlers",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[
            Rationaled::new(
                Resource {
                    path: PathRef("/usr/blueos/userdata/styles/theme_config.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted primary color JSON consumed by GET /theme and regenerated CSS",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/usr/blueos/userdata/styles/theme_style.css"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "generated theme CSS linked by the web UI at /userdata/styles/theme_style.css",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/usr/blueos/userdata/branding"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "directory for custom logo and Settings-panel vehicle image assets",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/usr/blueos/userdata/modeloverrides"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "directory for uploaded .glb 3D model overrides served to Vehicle Setup",
            ),
        ]),
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
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[],
                "observed ordered_before is empty; customization is last in the SERVICES startup list",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit logs Customization service stopped",
                "main.py finally block after server.serve returns",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for persisted userdata branding assets and in-flight uploads not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name",
            "no dedicated /health route; uvicorn availability serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "white-label branding utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /customization/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated",
            "customization routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "invalid_theme_color",
                "PUT /theme returns 400 when parse_hex rejects the primary color value",
            ),
            Rationaled::new(
                "upload_size_exceeded",
                "save_upload returns 413 when an uploaded file exceeds the configured size limit",
            ),
            Rationaled::new(
                "invalid_file_extension",
                "branding and model uploads return 400 for disallowed image or model suffixes",
            ),
            Rationaled::new(
                "model_not_found",
                "DELETE /models/{name} returns 404 when the requested override file does not exist",
            ),
            Rationaled::new(
                "path_traversal_blocked",
                "safe_join rejects model paths that escape userdata/modeloverrides via traversal",
            ),
        ]),
        blast_radius: Asserted::established(
            "custom theme, branding, and 3D model overrides unavailable; core vehicle services and MAVLink unaffected"
                ,
            "customization outage reverts the UI to default theme and assets but does not block autopilot or nginx core paths",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and allowed asset format evolution not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Customization,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
