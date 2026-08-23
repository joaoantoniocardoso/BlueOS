use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::{capability_def, frontend_capability_def};
use crate::catalog::Catalog;
use crate::id::{CapabilityId, JourneyId};
use crate::journey::{RouteRef, UserJourney};
use crate::provenance::{Grounded, GroundedSet};
use crate::runner::{http_method_label, resolve_http_path};

pub const FUNCTION_COUNT: usize = 108;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct FunctionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FunctionIo {
    Unknown { reason: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Function {
    pub id: FunctionId,
    pub capability: CapabilityId,
    pub verifying_journeys: Vec<JourneyId>,
    pub input: FunctionIo,
    pub output: FunctionIo,
    pub children: Vec<FunctionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FunctionCatalog {
    functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RouteSignature {
    service: String,
    method: String,
    path: String,
    version: String,
}

impl FunctionId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FunctionCatalog {
    pub fn bootstrap() -> Self {
        Self::from_catalog(&Catalog::bootstrap())
    }

    pub fn from_catalog(catalog: &Catalog) -> Self {
        let mut capability_journeys: BTreeMap<CapabilityId, Vec<JourneyId>> = BTreeMap::new();

        for journey in catalog.journeys() {
            let GroundedSet::Known { items } = &journey.capability_refs else {
                continue;
            };
            for item in items.iter() {
                capability_journeys
                    .entry(item.value)
                    .or_default()
                    .push(journey.id);
            }
        }

        let journey_id_strings: BTreeSet<&str> = catalog
            .journeys()
            .iter()
            .map(|journey| journey.id.as_str())
            .collect();

        let mut functions = Vec::new();

        for (capability, journey_ids) in capability_journeys {
            let mut clusters: BTreeMap<BTreeSet<RouteSignature>, Vec<JourneyId>> = BTreeMap::new();
            for journey_id in journey_ids {
                let journey = catalog
                    .journey_by_id(&journey_id)
                    .expect("journey id from capability index");
                let signatures = journey_route_signatures(catalog, journey);
                clusters.entry(signatures).or_default().push(journey_id);
            }

            let multi_cluster = clusters.len() > 1;
            let base_id = capability.as_str();
            let base_collides = journey_id_strings.contains(base_id);
            for (signatures, mut verifying_journeys) in clusters {
                verifying_journeys.sort_by_key(|id| id.as_str());
                let id = if multi_cluster || base_collides {
                    FunctionId(format!("{}/{}", base_id, cluster_key(&signatures)))
                } else {
                    FunctionId(base_id.to_string())
                };
                let (input, output) = io_for_signatures(&signatures);
                functions.push(Function {
                    id,
                    capability,
                    verifying_journeys,
                    input,
                    output,
                    children: Vec::new(),
                });
            }
        }

        functions.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        Self { functions }
    }

    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
}

impl RouteSignature {
    fn from_route(route: &RouteRef) -> Self {
        Self {
            service: route.service.as_str().to_string(),
            method: http_method_label(&route.method).to_string(),
            path: route.path.to_string(),
            version: route.version.unwrap_or("").to_string(),
        }
    }

    fn canonical(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.service, self.method, self.path, self.version
        )
    }
}

fn journey_route_signatures(catalog: &Catalog, journey: &UserJourney) -> BTreeSet<RouteSignature> {
    let GroundedSet::Known { items } = &journey.steps else {
        return BTreeSet::new();
    };
    let mut signatures = BTreeSet::new();
    for step in items.iter() {
        let Some(Grounded::Known { value: route, .. }) = &step.value.route else {
            continue;
        };
        if resolve_http_path(catalog, route).is_some() {
            signatures.insert(RouteSignature::from_route(route));
        }
    }
    signatures
}

fn io_for_signatures(signatures: &BTreeSet<RouteSignature>) -> (FunctionIo, FunctionIo) {
    let reason = if signatures.is_empty() {
        "no resolved route signature"
    } else {
        "fastapi extract is method+path only"
    };
    (
        FunctionIo::Unknown { reason },
        FunctionIo::Unknown { reason },
    )
}

fn cluster_key(signatures: &BTreeSet<RouteSignature>) -> String {
    let canonical = signatures
        .iter()
        .map(RouteSignature::canonical)
        .collect::<Vec<_>>()
        .join("|");
    format!("{:08x}", fnv1a64(canonical.as_bytes()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) fn capability_exists(capability: CapabilityId) -> bool {
    capability_def(capability).is_some() || frontend_capability_def(capability).is_some()
}

pub(crate) fn function_id_looks_like_http_path(id: &str) -> bool {
    id.starts_with('/') || id.contains("/v1") || id.contains("//")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ServiceId;
    use crate::journey::{
        Actor, BodyKind, HttpMethod, JourneyStep, StepOutcome, Visibility, BLAST_RADIUS_UNKNOWN,
    };
    use crate::provenance::{GroundedItem, Provenance};
    use crate::service::Service;
    use crate::version::FeatureAvailability;

    const TEST_PRESENCE: FeatureAvailability = FeatureAvailability {
        intro_commit: "0000000000000000000000000000000000000001",
        present_in_tags: &["1.0.0"],
        present_on_master: true,
        present_on_1_4_dev: true,
    };

    const TEST_SERVICES: &[GroundedItem<ServiceId>] = &[GroundedItem::new(
        ServiceId::Helper,
        Provenance::doc("test.md", 1, "helper"),
    )];
    const TEST_CAPABILITY_REFS: &[GroundedItem<CapabilityId>] = &[GroundedItem::new(
        CapabilityId::CheckInternetConnectivity,
        Provenance::doc("test.md", 1, "cap"),
    )];

    const ROUTE_CHECK: RouteRef = RouteRef {
        service: ServiceId::Helper,
        method: HttpMethod::Get,
        path: "/check_internet_access",
        version: Some("v1.0"),
    };
    const ROUTE_PING: RouteRef = RouteRef {
        service: ServiceId::Helper,
        method: HttpMethod::Get,
        path: "/ping",
        version: Some("v1.0"),
    };

    const STEPS_CHECK: &[GroundedItem<JourneyStep>] = &[GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: "call check",
            route: Some(Grounded::known(
                ROUTE_CHECK,
                Provenance::source("core/services/helper/main.py", 540, "def check_internet_a"),
            )),
            outcome: Some(Grounded::known(
                StepOutcome {
                    expected_status: Some(200),
                    body_predicate: None,
                    body_kind: BodyKind::Unknown,
                    transition: None,
                },
                Provenance::runtime("runtime-captures/helper.json#k", "test"),
            )),
        },
        Provenance::source("core/services/helper/main.py", 540, "def check_internet_a"),
    )];

    const STEPS_PING: &[GroundedItem<JourneyStep>] = &[GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: "call ping",
            route: Some(Grounded::known(
                ROUTE_PING,
                Provenance::source("core/services/helper/main.py", 583, "async def ping(host:"),
            )),
            outcome: Some(Grounded::known(
                StepOutcome {
                    expected_status: Some(200),
                    body_predicate: None,
                    body_kind: BodyKind::Unknown,
                    transition: None,
                },
                Provenance::runtime("runtime-captures/helper.json#k", "test"),
            )),
        },
        Provenance::source("core/services/helper/main.py", 583, "async def ping(host:"),
    )];

    const fn test_journey(
        id: JourneyId,
        steps: &'static [GroundedItem<JourneyStep>],
    ) -> UserJourney {
        UserJourney {
            id,
            summary: Grounded::known(
                "test journey",
                Provenance::doc("test.md", 1, "test journey"),
            ),
            visibility: Grounded::known(Visibility::Default, Provenance::doc("test.md", 1, "vis")),
            services: GroundedSet::known(TEST_SERVICES),
            capability_refs: GroundedSet::known(TEST_CAPABILITY_REFS),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(steps),
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
            chains_from: None,
        }
    }

    fn helper_from_bootstrap() -> Service {
        Catalog::bootstrap()
            .service_by_id(&ServiceId::Helper)
            .expect("helper service")
            .clone()
    }

    #[test]
    fn bootstrap_function_count_is_pinned() {
        let functions = FunctionCatalog::bootstrap();
        assert_eq!(functions.functions().len(), FUNCTION_COUNT);
    }

    #[test]
    fn bootstrap_functions_are_not_one_per_journey() {
        let catalog = Catalog::bootstrap();
        let functions = FunctionCatalog::from_catalog(&catalog);
        assert_ne!(functions.functions().len(), catalog.journeys().len());
    }

    #[test]
    fn bootstrap_merges_same_route_cluster_journeys() {
        let functions = FunctionCatalog::bootstrap();
        let function = functions
            .functions()
            .iter()
            .find(|function| function.capability == CapabilityId::CheckInternetConnectivity)
            .expect("check_internet_connectivity function");
        assert_eq!(function.verifying_journeys.len(), 2);
        assert!(function
            .verifying_journeys
            .contains(&JourneyId::MonitorInternetConnectivity));
        assert!(function
            .verifying_journeys
            .contains(&JourneyId::VerifyInternetConnectivity));
        assert_eq!(function.id.as_str(), "check_internet_connectivity");
    }

    #[test]
    fn same_capability_different_route_signatures_yield_suffixed_ids() {
        let catalog = Catalog::with_parts(
            vec![helper_from_bootstrap()],
            vec![
                test_journey(JourneyId::MonitorInternetConnectivity, STEPS_CHECK),
                test_journey(JourneyId::VerifyInternetConnectivity, STEPS_PING),
            ],
            vec![],
        );
        let functions = FunctionCatalog::from_catalog(&catalog);
        let matching: Vec<_> = functions
            .functions()
            .iter()
            .filter(|function| function.capability == CapabilityId::CheckInternetConnectivity)
            .collect();
        assert_eq!(matching.len(), 2);
        assert!(matching.iter().all(|function| function
            .id
            .as_str()
            .starts_with("check_internet_connectivity/")));
        assert_ne!(matching[0].id, matching[1].id);
        assert!(matching
            .iter()
            .all(|function| !function_id_looks_like_http_path(function.id.as_str())));
    }

    #[test]
    fn function_id_is_stable_when_journey_id_changes() {
        let original = Catalog::with_parts(
            vec![helper_from_bootstrap()],
            vec![test_journey(
                JourneyId::MonitorInternetConnectivity,
                STEPS_CHECK,
            )],
            vec![],
        );
        let renamed = Catalog::with_parts(
            vec![helper_from_bootstrap()],
            vec![test_journey(
                JourneyId::VerifyInternetConnectivity,
                STEPS_CHECK,
            )],
            vec![],
        );
        let id_original = FunctionCatalog::from_catalog(&original).functions()[0]
            .id
            .clone();
        let id_renamed = FunctionCatalog::from_catalog(&renamed).functions()[0]
            .id
            .clone();
        assert_eq!(id_original, id_renamed);
        assert_eq!(id_original.as_str(), "check_internet_connectivity");
    }

    #[test]
    fn function_id_http_path_probe_catches_routes_not_capability_suffix() {
        assert!(function_id_looks_like_http_path(
            "/helper/v1.0/check_internet_access"
        ));
        assert!(!function_id_looks_like_http_path(
            "check_internet_connectivity"
        ));
        assert!(!function_id_looks_like_http_path(
            "check_internet_connectivity/deadbeef"
        ));
    }
}
