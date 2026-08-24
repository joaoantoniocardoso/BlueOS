use schemars::JsonSchema;
use serde::Serialize;

use catalog_kernel::{
    id::{
        capability::CapabilityId, journey::JourneyId, page::PageId, refs::PathRef,
        service::ServiceId,
    },
    provenance::{Grounded, GroundedSet, Provenance},
    version::Availability,
};

pub const BLAST_RADIUS_UNKNOWN: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Unknown {
        reason: "not yet annotated",
    },
    Provenance::asserted("not yet annotated"),
);

// Every journey must set `availability` from `journey_presence::PRESENCE_*`
// (full git tag membership). `Availability::unknown()` fails `validate()`.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct UseCase {
    pub id: JourneyId,
    pub summary: Grounded<&'static str>,
    pub visibility: Grounded<Visibility>,
    pub services: GroundedSet<ServiceId>,
    pub capability_refs: GroundedSet<CapabilityId>,
    pub preconditions: GroundedSet<Precondition>,
    pub steps: GroundedSet<JourneyStep>,
    pub availability: Availability,
    pub blast_radius: Grounded<BlastRadius>,
    pub chains_from: Option<JourneyId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlastRadius {
    Safe,
    Reversible,
    Disruptive,
    Destructive,
    Unknown { reason: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BodyKind {
    Empty,
    ErrorEnvelope,
    Payload,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OracleClass {
    HttpPassthrough,
    ClientComposed,
    ClientOrchestrated,
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
    pub body_kind: BodyKind,
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
    /// A step driven by client-side (browser) logic on a given page -- the 1.x
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
    Hardware(HardwareAssumption),
    Software(SoftwareAssumption),
    NetworkResource(NetworkResource),
    Data(DataAssumption),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HardwareAssumption {
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
pub enum SoftwareAssumption {
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
pub enum DataAssumption {
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
pub enum VerificationMethod {
    Inspect,
    Analyze,
    Demo,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct StateTransition {
    pub machine: &'static str,
    pub from: &'static str,
    pub to: &'static str,
}

impl BoardKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "Any",
            Self::Navigator => "Navigator",
            Self::Pixhawk => "Pixhawk",
            Self::Sitl => "Sitl",
        }
    }
}

impl HardwareAssumption {
    pub fn as_str(self) -> String {
        match self {
            Self::FlightController(board) => format!("FlightController({})", board.as_str()),
            Self::UsbCamera => "UsbCamera".to_string(),
            Self::Ping1d => "Ping1d".to_string(),
            Self::Ping360 => "Ping360".to_string(),
            Self::ExternalNmeaGps => "ExternalNmeaGps".to_string(),
            Self::UsbSerialDevice => "UsbSerialDevice".to_string(),
        }
    }
}

impl SoftwareAssumption {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PirateMode => "PirateMode",
            Self::AdvancedMode => "AdvancedMode",
            Self::DevMode => "DevMode",
            Self::ConfirmDangerousOp => "ConfirmDangerousOp",
        }
    }
}

impl NetworkResource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WifiRadioPresent => "WifiRadioPresent",
            Self::KnownWifiNetwork => "KnownWifiNetwork",
            Self::HotspotCapable => "HotspotCapable",
            Self::WiredEthernetPresent => "WiredEthernetPresent",
            Self::UsbOtgPresent => "UsbOtgPresent",
        }
    }
}

impl DataAssumption {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExtensionInstalled => "ExtensionInstalled",
            Self::LocalBlueosVersionAvailable => "LocalBlueosVersionAvailable",
            Self::SerialBridgeConfigured => "SerialBridgeConfigured",
            Self::NmeaSocketConfigured => "NmeaSocketConfigured",
            Self::RecordingListed => "RecordingListed",
            Self::WifiNetworkSaved => "WifiNetworkSaved",
            Self::WifiCurrentlyConnected => "WifiCurrentlyConnected",
            Self::OnboardDhcpServerActive => "OnboardDhcpServerActive",
        }
    }
}

impl NetworkState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Online => "Online",
            Self::Offline => "Offline",
        }
    }
}

pub fn precondition_is_typed(p: &Precondition) -> bool {
    !matches!(p, Precondition::Other(_) | Precondition::HardwarePresent(_))
}

pub fn journey_requirements(journey: &UseCase) -> Vec<&Precondition> {
    match &journey.preconditions {
        GroundedSet::Known { items } => items.iter().map(|item| &item.value).collect(),
        GroundedSet::Unknown { .. } => Vec::new(),
    }
}

/// Tier-0 mechanical classification; not stored on [`UseCase`] until all journeys carry an
/// explicit field (const structs have no `..` default).
pub fn derive_automatable(journey: &UseCase) -> VerificationMethod {
    if let GroundedSet::Known { items: steps } = &journey.steps {
        if steps
            .iter()
            .any(|step| matches!(step.value.actor, Actor::Frontend(_)))
        {
            return VerificationMethod::Test;
        }
    }

    if journey_requirements(journey).iter().any(|p| {
        matches!(
            p,
            Precondition::Hardware(_) | Precondition::HardwarePresent(_)
        )
    }) {
        return VerificationMethod::Demo;
    }

    if let GroundedSet::Known { items: steps } = &journey.steps {
        if steps.is_empty() {
            return VerificationMethod::Inspect;
        }

        let mut has_known_route = false;
        for step in steps.iter() {
            match &step.value.route {
                None => {}
                Some(Grounded::Known { .. }) => has_known_route = true,
                Some(Grounded::Unknown { .. }) => return VerificationMethod::Inspect,
            }
        }

        if has_known_route {
            return VerificationMethod::Test;
        }

        return VerificationMethod::Inspect;
    }

    VerificationMethod::Inspect
}

pub fn http_automatable(journey: &UseCase) -> bool {
    derive_automatable(journey) == VerificationMethod::Test && !journey_has_frontend_step(journey)
}

fn journey_has_frontend_step(journey: &UseCase) -> bool {
    matches!(&journey.steps, GroundedSet::Known { items: steps } if steps
        .iter()
        .any(|step| matches!(step.value.actor, Actor::Frontend(_))))
}

pub fn blast_radius_is_unknown(journey: &UseCase) -> bool {
    matches!(
        journey.blast_radius,
        Grounded::Known {
            value: BlastRadius::Unknown { .. },
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use catalog_kernel::provenance::{GroundedItem, Provenance};

    use super::*;

    const DOC: Provenance = Provenance::doc("test.md", 1, "");

    const TEST_PRESENCE: Availability = Availability {
        intro_commit: "0000000000000000000000000000000000000001",
        present_in_tags: &["1.0.0"],
        present_on_master: true,
        present_on_1_4_dev: true,
    };

    const fn empty_journey(
        preconditions: GroundedSet<Precondition>,
        steps: GroundedSet<JourneyStep>,
    ) -> UseCase {
        UseCase {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions,
            steps,
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
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
            HardwareAssumption::UsbCamera
        )));
        assert!(!precondition_is_typed(&Precondition::Other("legacy")));
        assert!(!precondition_is_typed(&Precondition::HardwarePresent(
            "camera"
        )));
    }

    #[test]
    fn journey_requirements_flattens_known_preconditions() {
        static PRECONDITIONS: &[GroundedItem<Precondition>] = &[GroundedItem::new(
            Precondition::Software(SoftwareAssumption::PirateMode),
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
            Precondition::Software(SoftwareAssumption::PirateMode)
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
        assert_eq!(derive_automatable(&journey), VerificationMethod::Test);
    }

    #[test]
    fn derive_automatable_hardware_from_typed_or_legacy_precondition() {
        static LEGACY: &[GroundedItem<Precondition>] = &[GroundedItem::new(
            Precondition::HardwarePresent("camera"),
            DOC,
        )];
        let legacy = empty_journey(GroundedSet::known(LEGACY), GroundedSet::known(&[]));
        assert_eq!(derive_automatable(&legacy), VerificationMethod::Demo);

        static TYPED: &[GroundedItem<Precondition>] = &[GroundedItem::new(
            Precondition::Hardware(HardwareAssumption::Ping1d),
            DOC,
        )];
        let typed = empty_journey(GroundedSet::known(TYPED), GroundedSet::known(&[]));
        assert_eq!(derive_automatable(&typed), VerificationMethod::Demo);
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
        assert_eq!(derive_automatable(&journey), VerificationMethod::Test);
        assert!(http_automatable(&journey));
    }

    #[test]
    fn derive_automatable_manual_for_unknown_route_or_operator_only() {
        static UNKNOWN_ROUTE: &[GroundedItem<JourneyStep>] = &[GroundedItem::new(
            operator_step("scan", Some(Grounded::unknown("route not grounded yet"))),
            DOC,
        )];
        let unknown = empty_journey(GroundedSet::known(&[]), GroundedSet::known(UNKNOWN_ROUTE));
        assert_eq!(derive_automatable(&unknown), VerificationMethod::Inspect);

        static OPERATOR_ONLY: &[GroundedItem<JourneyStep>] =
            &[GroundedItem::new(operator_step("click connect", None), DOC)];
        let manual = empty_journey(GroundedSet::known(&[]), GroundedSet::known(OPERATOR_ONLY));
        assert_eq!(derive_automatable(&manual), VerificationMethod::Inspect);
    }

    #[test]
    fn blast_radius_unknown_counts_as_unannotated() {
        let journey = empty_journey(GroundedSet::known(&[]), GroundedSet::known(&[]));
        assert!(blast_radius_is_unknown(&journey));
    }
}
