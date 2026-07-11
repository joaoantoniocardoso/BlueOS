use crate::criticality::CriticalityTier;
use crate::edge::{Bus, Edge, FailureImpact, SyncMode};
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{Interface, MavlinkRole};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::MavlinkCameraManager,
        state_contracts: GroundedSet::unknown(
            "mavlink-camera-manager has no service-level state machine (card states Unknown); external Rust binary with no traced lifecycle states",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/v4l", 8.5, 14.9, 20.7, 60),
            runtime_slo(HttpMethod::Get, "/streams", 1.7, 5.9, 6.5, 60),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.59,
                median: 0.89,
                p95: 1.04,
                min: 0.0,
                max: 1.86,
                sd: 0.49,
            },
            Distribution {
                mean: 36.9,
                median: 36.9,
                p95: 36.9,
                min: 36.9,
                max: 36.9,
                sd: 0.0,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "no USB/external cameras attached on capture host; GET /streams returns empty array",
                    "GET /v4l lists onboard bcm2835-isp ISP nodes even without operator-attached cameras",
                    "runtime captured on Navigator only; Rust binary RSS ~36.9 MB flat, CPU ~0.59% mean idle (no active pipelines)",
                    "observed REST API is unversioned at runtime (routes directly under /mavlink-camera-manager/)",
                ],
            },
            runtime_prov("runtime-captures/mavlink_camera_manager__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /streams, POST /v4l, and DELETE /delete_stream are mutating; not exercised (no camera hardware on capture host)",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::MavlinkCameraManager,
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
        runtime_prov("runtime-captures/mavlink_camera_manager__pi4_navigator_master.json#slo_running_baseline"),
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
        runtime_prov(
            "runtime-captures/mavlink_camera_manager__pi4_navigator_master.json#resource_usage",
        ),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::MavlinkCameraManager,
        aliases: ObservedSet::known(&[Evidenced::new(
            "video",
            Evidence {
                file: "core/start-blueos-core",
                line: 120,
            },
        )]),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core",
                line: 120,
            },
        ),
        entrypoint: Observed::known(
            "nice --19 mavlink-camera-manager --default-settings BlueROVUDP --mavlink tcpout:127.0.0.1:5777 --mavlink-system-id $MAV_SYSTEM_ID --mavlink-camera-component-id-range=100-105 --gst-feature-rank omxh264enc=0,v4l2h264enc=250,x264enc=260 --log-path /var/logs/blueos/services/mavlink-camera-manager --stun-server stun://stun.l.google.com:19302 --enable-realtime-threads --recorder=external --zenoh --verbose"
                ,
            Evidence {
                file: "core/start-blueos-core",
                line: 120,
            },
        ),
        tmux_name: Observed::known(
            "video",
            Evidence {
                file: "core/start-blueos-core",
                line: 120,
            },
        ),
        startup_tier: Observed::known(
            StartupTier::Priority,
            Evidence {
                file: "core/start-blueos-core",
                line: 117,
            },
        ),
        resource_limits: Observed::known(
            ResourceLimits {
                memory_mb: Some(0),
                cpu_percent: Some(0),
                io_weight: None,
            },
            Evidence {
                file: "core/start-blueos-core",
                line: 120,
            },
        ),
        nice: Observed::known(
            19,
            Evidence {
                file: "core/start-blueos-core",
                line: 120,
            },
        ),
        run_as: Observed::known(
            "root",
            Evidence {
                file: "core/start-blueos-core",
                line: 120,
            },
        ),
        nginx_prefixes: ObservedSet::known(&[Evidenced::new(
            PathRef("/mavlink-camera-manager/"),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 192,
            },
        )]),
        listen: ObservedSet::known(&[Evidenced::new(
            PortRef::Literal(6020),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 194,
            },
        )]),
        git_path: Observed::unknown(
            "external Rust binary (upstream github.com/bluerobotics/mavlink-camera-manager); no source tree in this repository",
        ),
        interfaces: ObservedSet::known(&[
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/mavlink-camera-manager/"),
                    port: PortRef::Literal(6020),
                    versions: &[],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf",
                    line: 194,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Consumer,
                    connect: "tcpout:127.0.0.1:5777",
                },
                Evidence {
                    file: "core/start-blueos-core",
                    line: 120,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "stun://stun.l.google.com:19302",
                },
                Evidence {
                    file: "core/start-blueos-core",
                    line: 120,
                },
            ),
        ]),
        resources: ObservedSet::unknown(
            "no settings paths or userdata files referenced in start-blueos-core launch args",
        ),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: &["start-blueos-core create_service"],
                ordered_after: &[
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                ],
                ordered_before: &[
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
                    ServiceId::Customization,
                ],
            },
            Evidence {
                file: "core/start-blueos-core",
                line: 318,
            },
        ),
        logs_path: Observed::known(
            PathRef("/var/logs/blueos/services/mavlink-camera-manager"),
            Evidence {
                file: "core/start-blueos-core",
                line: 120,
            },
        ),
        zenoh_log_topic: Observed::unknown(
            "--zenoh flag present in launch args; no citable zenoh topic string in this repository",
        ),
        sentry: Observed::unknown(
            "external Rust binary; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    };

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::MavlinkCameraManager,
        singleton: Asserted::established(
            true,
            "single Priority-tier tmux instance (alias video); one mavlink-camera-manager process owns all camera detection and stream management",
        ),
        bounded_context: Asserted::established(
            "camera-stream-manager",
            "provisional 2.0 domain: detect cameras, manage UDP/RTSP/WebRTC video streams, and advertise them over MAVLink",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::ViewCameraStreams,
                "Video Streams page lists detected cameras and configured streams via GET /v4l and GET /streams",
            ),
            Rationaled::new(
                JourneyId::ConfigureCameraStream,
                "stream creation dialog submits encoding, resolution, and endpoints via POST /streams",
            ),
            Rationaled::new(
                JourneyId::RemoveCameraStream,
                "stream card remove button deletes a stream configuration via DELETE /delete_stream",
            ),
            Rationaled::new(
                JourneyId::ConfigureUvcDeviceControls,
                "Device Controls dialog adjusts UVC sliders and menus via POST /v4l",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "Priority-tier operator-critical video plane; piloting and recording lose all camera feeds when down, but the autopilot remains controllable via GCS and MAVLink router without camera MAVLink advertisements",
        ),
        offline_required: Asserted::established(
            true,
            "core UDP and RTSP streaming and local REST management work without internet; observed OutboundHttp stun://stun.l.google.com:19302 is a best-effort NAT-traversal enhancement for WebRTC, not a hard dependency for local streaming",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Priority-tier launch line; requires /dev/video* device access for camera capture",
        ),
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "rubric requires NotRequired when dangerous_operations is empty; stream removal is reversible configuration, not irreversible data destruction",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::ViewCameraStreams,
                "GET /v4l and GET /streams REST routes list detected cameras and configured streams",
            ),
            Rationaled::new(
                CapabilityId::ConfigureCameraStream,
                "POST /streams creates stream encoding, resolution, framerate, and UDP/RTSP endpoints",
            ),
            Rationaled::new(
                CapabilityId::RemoveCameraStream,
                "DELETE /delete_stream removes a stream configuration from the camera manager",
            ),
            Rationaled::new(
                CapabilityId::ConfigureUvcDeviceControls,
                "POST /v4l adjusts UVC camera control values such as brightness and exposure",
            ),
            Rationaled::new(
                CapabilityId::ProvideWebrtcSignalling,
                "nginx /webrtc/ws/ proxies to 127.0.0.1:6021 adjacent to MCM REST on :6020; MCM is BlueOS's sole WebRTC video provider",
            ),
            Rationaled::new(
                CapabilityId::AdvertiseCamerasOverMavlink,
                "observed Mavlink Consumer tcpout:127.0.0.1:5777 with --mavlink-camera-component-id-range=100-105 advertises camera streams to the MAVLink router",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("camera_stream_manager"),
            "sole catalog service that detects cameras, manages video streams, and provides WebRTC signalling; recorder and frontend video store depend on it exclusively",
        )]),
        states: AssertedSet::unknown(
            "external Rust binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(&[Rationaled::new(
            Edge {
                from: ServiceId::MavlinkCameraManager,
                to: ServiceId::ArdupilotManager,
                via: Bus::Mavlink,
                sync: SyncMode::Async,
                endpoint: "tcpout:127.0.0.1:5777",
                purpose: "advertise camera streams and MAVLink camera protocol to the vehicle MAVLink router",
                required_at_boot: true,
                failure_impact: FailureImpact::Degraded,
            },
            "observed Mavlink Consumer connect tcpout:127.0.0.1:5777 pairs with ardupilot_manager tcpin:127.0.0.1:5777 Endpoint created by the MAVLink router owner",
        )]),
        resources: AssertedSet::unknown(
            "observed artifact has no settings paths or userdata files; external binary with no traced on-disk resources",
        ),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in Priority tier",
            ),
            ordered_after: Asserted::established(
                &[
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                ],
                "observed ordered_after in start-blueos-core Priority block",
            ),
            ordered_before: Asserted::established(
                &[
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
                    ServiceId::Customization,
                ],
                "observed ordered_before lists video before mavlink2rest and remaining Priority and SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "external Rust binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external mavlink-camera-manager binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST /mavlink-camera-manager/ and stream availability serve as health signals"
                ,
            "no dedicated /health route traced; process continuity and HTTP listener serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "core infrastructure video service; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external binary with empty observed REST versions list; upstream API stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no auth middleware traced; REST routes are unauthenticated on the LAN",
            "external binary proxied by nginx without observed permission checks; LAN trust model matches other core REST services",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "mavlink_router_endpoint_unreachable",
                "tcpout:127.0.0.1:5777 has no router listener when ardupilot_manager is down or endpoint not yet created",
            ),
            Rationaled::new(
                "camera_device_unavailable",
                "no /dev/video* devices attached blocks camera detection and stream creation",
            ),
            Rationaled::new(
                "stream_pipeline_failure",
                "GStreamer encoder or pipeline error stops individual streams while the REST API may remain responsive",
            ),
            Rationaled::new(
                "webrtc_stun_unreachable",
                "external STUN server unreachable degrades cross-NAT WebRTC but local UDP/RTSP streaming continues",
            ),
        ]),
        blast_radius: Asserted::established(
            "operator loses all video feeds (piloting view, WebRTC browser streams, MAVLink camera advertisements, and recorder source); autopilot control and GCS MAVLink paths remain available"
                ,
            "sole camera and video-stream manager; outage is significant for ROV operation but does not affect the ardupilot_manager MAVLink router or vehicle control plane",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream mavlink-camera-manager API deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    };
