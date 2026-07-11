use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("nmea_injector".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "nmea-injector".to_string(),
            Evidence {
                file: "core/services/nmea_injector/main.py".to_string(),
                line: 17,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        entrypoint: Observed::known(
            "nice -19 $SERVICES_PATH/nmea_injector/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        tmux_name: Observed::known(
            "nmea_injector".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        startup_tier: Observed::known(
            StartupTier::Normal,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 124,
            },
        ),
        resource_limits: Observed::known(
            ResourceLimits {
                memory_mb: Some(250),
                cpu_percent: Some(0),
                io_weight: None,
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        nice: Observed::known(
            19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/nmea-injector/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 158,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(2748),
            Evidence {
                file: "core/services/nmea_injector/main.py".to_string(),
                line: 88,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/nmea_injector".to_string()),
            Evidence {
                file: "core/services/nmea_injector/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/nmea-injector/".to_string()),
                    port: PortRef::Literal(2748),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/nmea_injector/main.py".to_string(),
                    line: 69,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "localhost:6040".to_string(),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/mavlink_comm/MavlinkComm.py"
                        .to_string(),
                    line: 24,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/nmea-injector/settings-1.json".to_string()),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 69,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/nmea-injector".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/nmea-injector/log".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector/settings-1.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 69,
                },
            ),
        ]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![
                    ServiceId("autopilot".to_string()),
                    ServiceId("cable_guy".to_string()),
                    ServiceId("video".to_string()),
                    ServiceId("mavlink2rest".to_string()),
                    ServiceId("kraken".to_string()),
                    ServiceId("wifi".to_string()),
                    ServiceId("zenohd".to_string()),
                    ServiceId("beacon".to_string()),
                    ServiceId("bridget".to_string()),
                    ServiceId("commander".to_string()),
                ],
                ordered_before: vec![
                    ServiceId("helper".to_string()),
                    ServiceId("iperf3".to_string()),
                    ServiceId("linux2rest".to_string()),
                    ServiceId("filebrowser".to_string()),
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
                    ServiceId("ping".to_string()),
                    ServiceId("user_terminal".to_string()),
                    ServiceId("ttyd".to_string()),
                    ServiceId("nginx".to_string()),
                    ServiceId("bag_of_holding".to_string()),
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in nmea_injector source",
        ),
        zenoh_log_topic: Observed::known(
            "services/nmea-injector/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/nmea_injector/main.py".to_string(),
                line: 85,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("nmea_injector".to_string()),
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one nmea_injector process owns all NMEA listen sockets",
        ),
        bounded_context: Asserted::established(
            "external-gps-nmea-injection".to_string(),
            "provisional 2.0 domain: opt-in external NMEA ingest parsed into MAVLink GPS_INPUT for the autopilot",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("view_configured_nmea_sockets".into()),
                "NMEA Injector page lists configured sockets via GET /socks",
            ),
            Rationaled::new(
                JourneyId("add_external_nmea_gps_socket".into()),
                "creation dialog submits socket kind, port, and component ID via POST /socks",
            ),
            Rationaled::new(
                JourneyId("remove_configured_nmea_socket".into()),
                "socket card remove button deletes the matching socket via DELETE /socks",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "opt-in external GPS injection; core vehicle operation uses the autopilot's own GPS and does not require this service",
        ),
        offline_required: Asserted::established(
            true,
            "NMEA listen sockets and GPS_INPUT POST to localhost mavlink2rest operate without internet",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; binds UDP/TCP listen sockets on 0.0.0.0 and writes /root/.config/nmea-injector settings",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "socket add/remove is reversible reconfiguration; no irreversible, untrusted-code, or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("list_nmea_sockets".to_string()),
                "GET /socks returns configured NMEA sockets with kind, port, and MAVLink component ID",
            ),
            Rationaled::new(
                CapabilityId("create_nmea_socket".to_string()),
                "POST /socks opens a UDP or TCP listen socket and persists the spec in SettingsV1",
            ),
            Rationaled::new(
                CapabilityId("remove_nmea_socket".to_string()),
                "DELETE /socks closes the matching listen socket and removes it from SettingsV1",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("nmea_gps_injector".to_string()),
            "sole catalog service that ingests external NMEA and produces MAVLink GPS_INPUT messages via mavlink2rest",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; socket listeners and settings reload are managed inside TrafficController",
        ),
        edges: AssertedSet::unknown(
            "observed OutboundHttp to localhost:6040 (mavlink2rest) for GPS_INPUT; target not cataloged yet so no validated ServiceId edge",
        ),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV1 pykson manager directory for persisted NMEA socket specifications",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector/settings-1.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted socket kind, port, and MAVLink component ID list consumed only by nmea_injector",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in SERVICES tier",
            ),
            ordered_after: Asserted::established(
                vec![
                    ServiceId("autopilot".to_string()),
                    ServiceId("cable_guy".to_string()),
                    ServiceId("video".to_string()),
                    ServiceId("mavlink2rest".to_string()),
                    ServiceId("kraken".to_string()),
                    ServiceId("wifi".to_string()),
                    ServiceId("zenohd".to_string()),
                    ServiceId("beacon".to_string()),
                    ServiceId("bridget".to_string()),
                    ServiceId("commander".to_string()),
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("helper".to_string()),
                    ServiceId("iperf3".to_string()),
                    ServiceId("linux2rest".to_string()),
                    ServiceId("filebrowser".to_string()),
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
                    ServiceId("ping".to_string()),
                    ServiceId("user_terminal".to_string()),
                    ServiceId("ttyd".to_string()),
                    ServiceId("nginx".to_string()),
                    ServiceId("bag_of_holding".to_string()),
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
                "observed ordered_before lists nmea_injector before remaining SERVICES-tier peers including customization",
            ),
            shutdown: Asserted::unknown(
                "main.py has no explicit shutdown handler; socket cleanup relies on TrafficController __del__",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight NMEA sockets and settings migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name".to_string(),
            "no dedicated /health route; uvicorn availability serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "external GPS injection utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /nmea-injector/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated".to_string(),
            "nmea_injector routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "injecting_wrong_position_affects_navigation".to_string(),
                "malformed or spoofed NMEA forwarded as GPS_INPUT can skew autopilot position estimates while sockets are active",
            ),
            Rationaled::new(
                "mavlink2rest_unreachable".to_string(),
                "MavlinkMessenger POST to localhost:6040 fails when mavlink2rest is down; NMEA data is dropped",
            ),
            Rationaled::new(
                "invalid_nmea_parse_failure".to_string(),
                "pynmea2.parse errors on non-NMEA datagrams prevent GPS_INPUT forwarding for that message",
            ),
            Rationaled::new(
                "port_conflict_on_socket_create".to_string(),
                "add_sock fails when the requested UDP/TCP port is already bound on the host",
            ),
            Rationaled::new(
                "remove_nonexistent_socket".to_string(),
                "DELETE /socks returns error when the specified kind, port, and component ID is not configured",
            ),
        ]),
        blast_radius: Asserted::established(
            "external GPS injection unavailable; autopilot falls back to onboard GPS; misconfigured active sockets can corrupt position data"
                .to_string(),
            "outage stops optional external GPS path; live misconfiguration affects navigation estimates but not arm/disarm or motion commands directly",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV1 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
