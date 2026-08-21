use crate::catalog::Catalog;
use crate::id::ServiceId;
use crate::journey::{JourneyStep, RouteRef};
use crate::provenance::{Grounded, GroundedSet, Provenance};
use crate::runner::resolve_http_path;
use crate::validate::ValidationError;
use serde::Serialize;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FrontendApiBase {
    pub store_file: &'static str,
    pub line: u32,
    pub api_url: &'static str,
    pub service: ServiceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FrontendEndpoint {
    pub base: &'static FrontendApiBase,
    pub relative_path: &'static str,
    pub call_file: &'static str,
    pub call_line: u32,
}

const RECORDS_BASE: FrontendApiBase = FrontendApiBase {
    store_file: "core/frontend/src/store/records.ts",
    line: 11,
    api_url: "/recorder-extractor/v1.0/recorder",
    service: ServiceId::RecorderExtractor,
};

const HELPER_BASE: FrontendApiBase = FrontendApiBase {
    store_file: "core/frontend/src/store/helper.ts",
    line: 36,
    api_url: "/helper/latest",
    service: ServiceId::Helper,
};

const KRAKEN_BASE: FrontendApiBase = FrontendApiBase {
    store_file: "core/frontend/src/components/kraken/KrakenManager.ts",
    line: 15,
    api_url: "/kraken/v2.0",
    service: ServiceId::Kraken,
};

const WIFI_BASE: FrontendApiBase = FrontendApiBase {
    store_file: "core/frontend/src/store/wifi.ts",
    line: 18,
    api_url: "/wifi-manager/v1.0",
    service: ServiceId::Wifi,
};

const ETHERNET_BASE: FrontendApiBase = FrontendApiBase {
    store_file: "core/frontend/src/store/ethernet.ts",
    line: 21,
    api_url: "/cable-guy/v1.0",
    service: ServiceId::CableGuy,
};

const VERSION_CHOOSER_BASE: FrontendApiBase = FrontendApiBase {
    store_file: "core/frontend/src/utils/version_chooser.ts",
    line: 11,
    api_url: "/version-chooser/v1.0",
    service: ServiceId::Versionchooser,
};

const ARDUPILOT_BASE: FrontendApiBase = FrontendApiBase {
    store_file: "core/frontend/src/store/autopilot_manager.ts",
    line: 25,
    api_url: "/ardupilot-manager/v1.0",
    service: ServiceId::ArdupilotManager,
};

pub const FRONTEND_API_ENDPOINTS: &[FrontendEndpoint] = &[
    FrontendEndpoint {
        base: &RECORDS_BASE,
        relative_path: "/files",
        call_file: "core/frontend/src/store/records.ts",
        call_line: 47,
    },
    FrontendEndpoint {
        base: &RECORDS_BASE,
        relative_path: "/status",
        call_file: "core/frontend/src/store/records.ts",
        call_line: 84,
    },
    FrontendEndpoint {
        base: &HELPER_BASE,
        relative_path: "/check_internet_access",
        call_file: "core/frontend/src/store/helper.ts",
        call_line: 71,
    },
    FrontendEndpoint {
        base: &HELPER_BASE,
        relative_path: "/web_services",
        call_file: "core/frontend/src/store/helper.ts",
        call_line: 106,
    },
    FrontendEndpoint {
        base: &HELPER_BASE,
        relative_path: "/ping",
        call_file: "core/frontend/src/store/helper.ts",
        call_line: 135,
    },
    FrontendEndpoint {
        base: &KRAKEN_BASE,
        relative_path: "/manifest/consolidated",
        call_file: "core/frontend/src/components/kraken/KrakenManager.ts",
        call_line: 73,
    },
    FrontendEndpoint {
        base: &KRAKEN_BASE,
        relative_path: "/extension/",
        call_file: "core/frontend/src/components/kraken/KrakenManager.ts",
        call_line: 24,
    },
    FrontendEndpoint {
        base: &WIFI_BASE,
        relative_path: "/scan",
        call_file: "core/frontend/src/components/wifi/WifiUpdater.vue",
        call_line: 112,
    },
    FrontendEndpoint {
        base: &WIFI_BASE,
        relative_path: "/connect",
        call_file: "core/frontend/src/components/wifi/ConnectionDialog.vue",
        call_line: 208,
    },
    FrontendEndpoint {
        base: &WIFI_BASE,
        relative_path: "/disconnect",
        call_file: "core/frontend/src/components/wifi/DisconnectionDialog.vue",
        call_line: 140,
    },
    FrontendEndpoint {
        base: &ETHERNET_BASE,
        relative_path: "/address",
        call_file: "core/frontend/src/store/ethernet.ts",
        call_line: 44,
    },
    FrontendEndpoint {
        base: &ETHERNET_BASE,
        relative_path: "/host_dns",
        call_file: "core/frontend/src/store/ethernet.ts",
        call_line: 128,
    },
    FrontendEndpoint {
        base: &VERSION_CHOOSER_BASE,
        relative_path: "/version/current/",
        call_file: "core/frontend/src/utils/version_chooser.ts",
        call_line: 157,
    },
    FrontendEndpoint {
        base: &VERSION_CHOOSER_BASE,
        relative_path: "/version/available/local",
        call_file: "core/frontend/src/utils/version_chooser.ts",
        call_line: 135,
    },
    FrontendEndpoint {
        base: &ARDUPILOT_BASE,
        relative_path: "/firmware_info",
        call_file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
        call_line: 77,
    },
    FrontendEndpoint {
        base: &ARDUPILOT_BASE,
        relative_path: "/install_firmware_from_url",
        call_file: "core/frontend/src/components/autopilot/FirmwareManager.vue",
        call_line: 441,
    },
];

pub fn compose_api_path(api_url: &str, relative_path: &str) -> String {
    let base = api_url.trim_end_matches('/');
    let suffix = if relative_path.starts_with('/') {
        relative_path.to_string()
    } else {
        format!("/{relative_path}")
    };
    format!("{base}{suffix}")
}

pub fn expected_resolved_path(endpoint: &FrontendEndpoint) -> String {
    compose_api_path(endpoint.base.api_url, endpoint.relative_path)
}

fn path_without_query(path: &str) -> &str {
    path.split('?').next().unwrap_or(path)
}

fn canonicalize_path(path: &str) -> String {
    let path = path_without_query(path).trim_end_matches('/');
    if path.is_empty() {
        "/".to_string()
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

pub fn paths_equivalent(expected: &str, resolved: &str) -> bool {
    let expected = canonicalize_path(expected);
    let resolved = canonicalize_path(resolved);
    if expected == resolved {
        return true;
    }
    let expected_v1 = expected.replace("/helper/latest/", "/helper/v1.0/");
    canonicalize_path(&expected_v1) == resolved
}

fn provenance_cites(provenance: &Provenance, call_file: &str, call_line: u32) -> bool {
    let (file, line) = match provenance {
        Provenance::Source(evidence) => (evidence.file, evidence.line),
        Provenance::Doc { file, line, .. } => (*file, *line),
        Provenance::Runtime { .. } | Provenance::Asserted { .. } => return false,
    };
    file.ends_with(call_file) && line == call_line
}

fn route_relative_matches(route: &RouteRef, endpoint: &FrontendEndpoint) -> bool {
    if route.service != endpoint.base.service {
        return false;
    }
    let route_path = canonicalize_path(route.path);
    let relative = canonicalize_path(endpoint.relative_path);
    route_path == relative || route_path.ends_with(&relative)
}

fn step_matches_endpoint(
    step: &JourneyStep,
    step_provenance: &Provenance,
    endpoint: &FrontendEndpoint,
    route: &RouteRef,
) -> bool {
    if provenance_cites(step_provenance, endpoint.call_file, endpoint.call_line) {
        return true;
    }
    if let Some(Grounded::Known { provenance, .. }) = &step.route {
        if provenance_cites(provenance, endpoint.call_file, endpoint.call_line) {
            return true;
        }
    }
    route_relative_matches(route, endpoint)
}

pub fn check_frontend_route_refs(catalog: &Catalog) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for journey in catalog.journeys() {
        let GroundedSet::Known { items: steps } = &journey.steps else {
            continue;
        };
        let journey_id = journey.id.to_string();

        for (step_index, step) in steps.iter().enumerate() {
            let Some(Grounded::Known {
                value: route_ref, ..
            }) = &step.value.route
            else {
                continue;
            };
            if route_ref.path.contains('{') {
                continue;
            }
            let Some(resolved) = resolve_http_path(catalog, route_ref) else {
                continue;
            };

            for endpoint in FRONTEND_API_ENDPOINTS {
                if !step_matches_endpoint(&step.value, &step.provenance, endpoint, route_ref) {
                    continue;
                }
                let expected = expected_resolved_path(endpoint);
                if paths_equivalent(&expected, &resolved) {
                    continue;
                }
                errors.push(ValidationError::FrontendRoutePrefixDropped {
                    journey: journey_id.clone(),
                    step: step_index,
                    resolved: resolved.clone(),
                    expected,
                    store_file: endpoint.base.store_file,
                });
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRESENCE: crate::version::FeatureAvailability =
        crate::version::FeatureAvailability {
            intro_commit: "0000000000000000000000000000000000000001",
            present_in_tags: &["1.0.0"],
            present_on_master: true,
            present_on_1_4_dev: true,
        };

    use crate::catalog::Catalog;
    use crate::id::{CapabilityId, JourneyId};
    use crate::journey::BLAST_RADIUS_UNKNOWN;
    use crate::journey::{Actor, HttpMethod, JourneyStep, RouteRef, UserJourney, Visibility};
    use crate::provenance::{Grounded, GroundedItem, GroundedSet};
    use crate::runtime::RuntimeFacts;
    use crate::service::Service;
    use crate::services::recorder_extractor::{OBSERVED_FACTS, SERVICE_DEFINITION};

    #[test]
    fn frontend_api_endpoints_table_is_populated() {
        assert!(FRONTEND_API_ENDPOINTS.len() >= 12);
    }

    #[test]
    fn compose_records_files_path() {
        let endpoint = &FRONTEND_API_ENDPOINTS[0];
        assert_eq!(
            expected_resolved_path(endpoint),
            "/recorder-extractor/v1.0/recorder/files"
        );
    }

    #[test]
    fn helper_latest_alias_equivalent_to_v1() {
        assert!(paths_equivalent("/helper/latest/ping", "/helper/v1.0/ping"));
    }

    #[test]
    fn bootstrap_catalog_passes_frontend_route_refs() {
        let errors = check_frontend_route_refs(&Catalog::bootstrap());
        assert!(
            errors.is_empty(),
            "unexpected frontend route errors: {errors:?}"
        );
    }

    #[test]
    fn dropped_recorder_prefix_is_detected() {
        const SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
            ServiceId::RecorderExtractor,
            Provenance::doc("test.md", 1, ""),
        )]);
        const CAPABILITIES: GroundedSet<CapabilityId> = GroundedSet::known(&[GroundedItem::new(
            CapabilityId::BrowseVideoRecordings,
            Provenance::asserted("test"),
        )]);
        const STEPS: &[GroundedItem<JourneyStep>] = &[GroundedItem::new(
            JourneyStep {
                actor: Actor::Operator,
                description: "load recordings",
                route: Some(Grounded::known(
                    RouteRef {
                        service: ServiceId::RecorderExtractor,
                        method: HttpMethod::Get,
                        path: "/files",
                        version: Some("v1.0"),
                    },
                    Provenance::source(
                        "core/frontend/src/store/records.ts",
                        47,
                        "url: `${this.API_URL}/files`,",
                    ),
                )),
                outcome: None,
            },
            Provenance::source(
                "core/frontend/src/store/records.ts",
                47,
                "url: `${this.API_URL}/files`,",
            ),
        )];

        let journey = UserJourney {
            id: JourneyId::BrowseVideoRecordings,
            summary: Grounded::known("test", Provenance::doc("test.md", 1, "")),
            visibility: Grounded::known(Visibility::Default, Provenance::doc("test.md", 1, "")),
            services: SERVICES,
            capability_refs: CAPABILITIES,
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
            chains_from: None,
        };

        let catalog = Catalog::with_parts(
            vec![Service {
                id: ServiceId::RecorderExtractor,
                observed: OBSERVED_FACTS,
                definition: SERVICE_DEFINITION,
                runtime: RuntimeFacts {
                    service: ServiceId::RecorderExtractor,
                    state_contracts: GroundedSet::unknown("test"),
                    slo_baselines: GroundedSet::unknown("test"),
                    resource_usage: GroundedSet::unknown("test"),
                    platform_matrix: GroundedSet::unknown("test"),
                    settings_mutations: GroundedSet::unknown("test"),
                },
            }],
            vec![journey],
            vec![],
        );

        let errors = check_frontend_route_refs(&catalog);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            &errors[0],
            ValidationError::FrontendRoutePrefixDropped {
                journey,
                resolved,
                expected,
                store_file,
                ..
            } if journey == "browse_video_recordings"
                && resolved == "/recorder-extractor/v1.0/files"
                && expected == "/recorder-extractor/v1.0/recorder/files"
                && store_file == &"core/frontend/src/store/records.ts"
        ));
    }
}
