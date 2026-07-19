use schemars::JsonSchema;
use serde::Serialize;

use crate::id::{CapabilityId, JourneyId, PathRef, ServiceId};
use crate::page::PageId;
use crate::provenance::{Grounded, GroundedSet};
use crate::version::FeatureAvailability;

// Journey `availability` bounds are seeded incrementally as release history is grounded;
// unknown is the honest default until a feature's intro/removal tag is confirmed.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct UserJourney {
    pub id: JourneyId,
    pub summary: Grounded<&'static str>,
    pub visibility: Grounded<Visibility>,
    pub services: GroundedSet<ServiceId>,
    pub capability_refs: GroundedSet<CapabilityId>,
    pub preconditions: GroundedSet<Precondition>,
    pub steps: GroundedSet<JourneyStep>,
    pub availability: FeatureAvailability,
    pub chains_from: Option<JourneyId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct JourneyStep {
    pub actor: Actor,
    pub description: &'static str,
    pub route: Option<Grounded<RouteRef>>,
    pub outcome: Option<Grounded<StepOutcome>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct RouteRef {
    pub service: ServiceId,
    pub method: HttpMethod,
    pub path: &'static str,
    pub version: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct StepOutcome {
    pub expected_status: Option<u16>,
    pub body_predicate: Option<&'static str>,
    pub transition: Option<StateTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    Operator,
    Service(ServiceId),
    Subprocess(&'static str),
    /// A step driven by client-side (browser) logic on a given page — the 1.x
    /// pattern where a wizard/sequence runs in the frontend rather than a service.
    Frontend(PageId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Default,
    Advanced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Precondition {
    ServiceState {
        service: ServiceId,
        state: &'static str,
    },
    Network(NetworkState),
    HardwarePresent(&'static str),
    ConfigClean(PathRef),
    Other(&'static str),
    Hardware(HardwareRequirement),
    Software(SoftwareRequirement),
    NetworkResource(NetworkResource),
    Data(DataRequirement),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HardwareRequirement {
    FlightController(BoardKind),
    UsbCamera,
    Ping1d,
    Ping360,
    ExternalNmeaGps,
    UsbSerialDevice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BoardKind {
    Any,
    Navigator,
    Pixhawk,
    Sitl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SoftwareRequirement {
    PirateMode,
    AdvancedMode,
    DevMode,
    ConfirmDangerousOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NetworkResource {
    WifiRadioPresent,
    KnownWifiNetwork,
    HotspotCapable,
    WiredEthernetPresent,
    UsbOtgPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataRequirement {
    ExtensionInstalled,
    LocalBlueosVersionAvailable,
    SerialBridgeConfigured,
    NmeaSocketConfigured,
    RecordingListed,
    WifiNetworkSaved,
    WifiCurrentlyConnected,
    OnboardDhcpServerActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NetworkState {
    Online,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Automatable {
    Http,
    Frontend,
    Hardware,
    ExternalGcs,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct StateTransition {
    pub machine: &'static str,
    pub from: &'static str,
    pub to: &'static str,
}

pub fn precondition_is_typed(p: &Precondition) -> bool {
    !matches!(p, Precondition::Other(_) | Precondition::HardwarePresent(_))
}

pub fn journey_requirements(journey: &UserJourney) -> Vec<&Precondition> {
    match &journey.preconditions {
        GroundedSet::Known { items } => items.iter().map(|item| &item.value).collect(),
        GroundedSet::Unknown { .. } => Vec::new(),
    }
}

/// Tier-0 mechanical classification; not stored on [`UserJourney`] until all journeys carry an
/// explicit field (const structs have no `..` default).
pub fn derive_automatable(journey: &UserJourney) -> Automatable {
    if let GroundedSet::Known { items: steps } = &journey.steps {
        if steps
            .iter()
            .any(|step| matches!(step.value.actor, Actor::Frontend(_)))
        {
            return Automatable::Frontend;
        }
    }

    if journey_requirements(journey).iter().any(|p| {
        matches!(
            p,
            Precondition::Hardware(_) | Precondition::HardwarePresent(_)
        )
    }) {
        return Automatable::Hardware;
    }

    if let GroundedSet::Known { items: steps } = &journey.steps {
        if steps.is_empty() {
            return Automatable::Manual;
        }

        let mut has_known_route = false;
        for step in steps.iter() {
            match &step.value.route {
                None => {}
                Some(Grounded::Known { .. }) => has_known_route = true,
                Some(Grounded::Unknown { .. }) => return Automatable::Manual,
            }
        }

        if has_known_route {
            return Automatable::Http;
        }

        return Automatable::Manual;
    }

    Automatable::Manual
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::JourneyId;
    use crate::page::PageId;
    use crate::provenance::{GroundedItem, Provenance};

    const DOC: Provenance = Provenance::doc("test.md", 1);

    const fn empty_journey(
        preconditions: GroundedSet<Precondition>,
        steps: GroundedSet<JourneyStep>,
    ) -> UserJourney {
        UserJourney {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions,
            steps,
            availability: FeatureAvailability::unknown(),
            chains_from: None,
        }
    }

    const fn operator_step(
        description: &'static str,
        route: Option<Grounded<RouteRef>>,
    ) -> JourneyStep {
        JourneyStep {
            actor: Actor::Operator,
            description,
            route,
            outcome: None,
        }
    }

    const WIFI_ROUTE: RouteRef = RouteRef {
        service: ServiceId::Wifi,
        method: HttpMethod::Get,
        path: "/scan",
        version: Some("v1.0"),
    };

    #[test]
    fn precondition_is_typed_excludes_legacy_variants() {
        assert!(precondition_is_typed(&Precondition::Network(
            NetworkState::Online
        )));
        assert!(precondition_is_typed(&Precondition::Hardware(
            HardwareRequirement::UsbCamera
        )));
        assert!(!precondition_is_typed(&Precondition::Other("legacy")));
        assert!(!precondition_is_typed(&Precondition::HardwarePresent(
            "camera"
        )));
    }

    #[test]
    fn journey_requirements_flattens_known_preconditions() {
        static PRECONDITIONS: &[GroundedItem<Precondition>] = &[GroundedItem::new(
            Precondition::Software(SoftwareRequirement::PirateMode),
            DOC,
        )];
        let journey = empty_journey(
            GroundedSet::known(PRECONDITIONS),
            GroundedSet::unknown("test"),
        );
        let reqs = journey_requirements(&journey);
        assert_eq!(reqs.len(), 1);
        assert!(matches!(
            reqs[0],
            Precondition::Software(SoftwareRequirement::PirateMode)
        ));
    }

    #[test]
    fn derive_automatable_frontend_when_any_step_is_frontend() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[GroundedItem::new(
            JourneyStep {
                actor: Actor::Frontend(PageId::VehicleSetup),
                description: "run wizard",
                route: None,
                outcome: None,
            },
            DOC,
        )];
        let journey = empty_journey(GroundedSet::known(&[]), GroundedSet::known(STEPS));
        assert_eq!(derive_automatable(&journey), Automatable::Frontend);
    }

    #[test]
    fn derive_automatable_hardware_from_typed_or_legacy_precondition() {
        static LEGACY: &[GroundedItem<Precondition>] = &[GroundedItem::new(
            Precondition::HardwarePresent("camera"),
            DOC,
        )];
        let legacy = empty_journey(GroundedSet::known(LEGACY), GroundedSet::known(&[]));
        assert_eq!(derive_automatable(&legacy), Automatable::Hardware);

        static TYPED: &[GroundedItem<Precondition>] = &[GroundedItem::new(
            Precondition::Hardware(HardwareRequirement::Ping1d),
            DOC,
        )];
        let typed = empty_journey(GroundedSet::known(TYPED), GroundedSet::known(&[]));
        assert_eq!(derive_automatable(&typed), Automatable::Hardware);
    }

    #[test]
    fn derive_automatable_http_when_all_routed_steps_are_known() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[
            GroundedItem::new(operator_step("open tray", None), DOC),
            GroundedItem::new(
                operator_step("scan", Some(Grounded::known(WIFI_ROUTE, DOC))),
                DOC,
            ),
        ];
        let journey = empty_journey(GroundedSet::known(&[]), GroundedSet::known(STEPS));
        assert_eq!(derive_automatable(&journey), Automatable::Http);
    }

    #[test]
    fn derive_automatable_manual_for_unknown_route_or_operator_only() {
        static UNKNOWN_ROUTE: &[GroundedItem<JourneyStep>] = &[GroundedItem::new(
            operator_step("scan", Some(Grounded::unknown("route not grounded yet"))),
            DOC,
        )];
        let unknown = empty_journey(GroundedSet::known(&[]), GroundedSet::known(UNKNOWN_ROUTE));
        assert_eq!(derive_automatable(&unknown), Automatable::Manual);

        static OPERATOR_ONLY: &[GroundedItem<JourneyStep>] =
            &[GroundedItem::new(operator_step("click connect", None), DOC)];
        let manual = empty_journey(GroundedSet::known(&[]), GroundedSet::known(OPERATOR_ONLY));
        assert_eq!(derive_automatable(&manual), Automatable::Manual);
    }
}
