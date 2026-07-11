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
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("bag_of_holding".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "bag-of-holding".to_string(),
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 21,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 144,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/bag_of_holding/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 144,
            },
        ),
        tmux_name: Observed::known(
            "bag_of_holding".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 144,
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
                line: 144,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 144,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/bag/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 86,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9101),
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 124,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/bag_of_holding".to_string()),
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/bag/".to_string()),
                    port: PortRef::Literal(9101),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/bag_of_holding/main.py".to_string(),
                    line: 105,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/bag-of-holding".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/bag_of_holding/main.py".to_string(),
                    line: 22,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/bag-of-holding/log".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![Evidenced::new(
            Resource {
                path: PathRef("/root/.config/bag-of-holding".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 61,
            },
        )]),
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
                    ServiceId("nmea_injector".to_string()),
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
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in bag_of_holding source",
        ),
        zenoh_log_topic: Observed::known(
            "services/bag-of-holding/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 121,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("bag_of_holding".to_string()),
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one process owns the shared JSON document store",
        ),
        bounded_context: Asserted::established(
            "generic-json-persistence".to_string(),
            "provisional 2.0 domain: shared key-value JSON store for frontend UI state, wizard progress, and feature tokens",
        ),
        journey_refs: AssertedSet::established(vec![Rationaled::new(
            JourneyId("modify_bag_database".into()),
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
        dangerous_operations: AssertedSet::established(vec![Rationaled::new(
            DangerousOperation::Other("overwrite_entire_datastore".to_string()),
            "POST /overwrite atomically replaces the entire db.json; a bad payload wipes all stored UI/setup/cloud-token state at once",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "whole-store overwrite is destructive bulk replacement of persisted frontend and setup state",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("edit_bag_json_store".to_string()),
                "Bag Editor reads GET /get/* and saves the full edited document via POST /overwrite",
            ),
            Rationaled::new(
                CapabilityId("set_bag_value".to_string()),
                "POST /set/{path} merges a value at a dpath key into the current JSON document",
            ),
            Rationaled::new(
                CapabilityId("get_bag_value".to_string()),
                "GET /get/{path} returns a subtree or GET /get/* returns the full document",
            ),
            Rationaled::new(
                CapabilityId("overwrite_bag_store".to_string()),
                "POST /overwrite replaces db.json with the request body JSON object",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("json_document_store".to_string()),
            "sole owner and writer of the shared generic JSON persistence file consumed by the frontend and feature stores",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; key-value reads and writes are stateless request handlers",
        ),
        edges: AssertedSet::unknown(
            "no outbound coupling to other catalog services; frontend and operators reach bag via nginx REST only",
        ),
        resources: AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/root/.config/bag-of-holding".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            "config directory containing db.json for wizard state, settings, vehicle images, and Major Tom tokens",
        )]),
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
                    ServiceId("nmea_injector".to_string()),
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
                "observed ordered_before lists bag_of_holding before recorder, disk_usage, and customization",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit when main() serve loop returns".to_string(),
                "no explicit shutdown hook; process stops with tmux session teardown",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight writes and db.json migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns HTML title page".to_string(),
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
            "no service-level auth; Bag Editor gated by pirate mode in frontend only".to_string(),
            "REST endpoints accept any caller reaching nginx; advanced UI access is a frontend precondition",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "database_file_missing".to_string(),
                "read_db returns empty object when db.json does not exist yet",
            ),
            Rationaled::new(
                "json_decode_error".to_string(),
                "read_db logs JSONDecodeError and returns empty object when db.json is corrupt",
            ),
            Rationaled::new(
                "invalid_get_path".to_string(),
                "GET /get/{path} returns 400 when dpath.get finds no key at the requested path",
            ),
        ]),
        blast_radius: Asserted::established(
            "frontend wizard, settings, cloud tokens, and vehicle image paths unavailable; core vehicle services unaffected"
                .to_string(),
            "bag_of_holding outage blocks UI persistence but not autopilot, MAVLink, or nginx core paths",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy for unversioned /overwrite vs versioned v1.0 routes not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
