use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, JourneyStep, Precondition, SoftwareRequirement, StepOutcome, UserJourney,
    Visibility,
};
use crate::journey_presence::PRESENCE_INSPECT_ZENOH_NETWORK;
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ZENOH_MENUS: &str = "core/frontend/src/menus.ts";
const ZENOH_VIEW: &str = "core/frontend/src/views/ZenohInspectorView.vue";
const ZENOH_NETWORK: &str = "core/frontend/src/components/zenoh-inspector/ZenohNetwork.vue";
const ZENOH_INSPECTOR: &str = "core/frontend/src/components/zenoh-inspector/ZenohInspector.vue";
const ZENOH_LIB: &str = "core/frontend/src/libs/zenoh/index.ts";
const NGINX: &str = "core/tools/nginx/nginx.conf";

pub const JOURNEYS: &[UserJourney] = &[INSPECT_ZENOH_NETWORK];

const INSPECT_ZENOH_NETWORK: UserJourney =
    UserJourney {
        id: JourneyId::InspectZenohNetwork,
        summary: Grounded::known(
            "View detailed Zenoh traffic coming from your vehicle",
            Provenance::source(ZENOH_MENUS, 146),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(ZENOH_MENUS, 145)),
        services: ZENOHD_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::InspectZenohNetwork,
            "Zenoh Inspector connects over WebSocket to inspect live pub/sub topics and network topology",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Software(SoftwareRequirement::AdvancedMode),
            Provenance::source(ZENOH_MENUS, 145),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Zenoh Inspector page from the sidebar",
                None,
                Provenance::source(ZENOH_MENUS, 142),
                None,
            ),
            operator_step(
                "Load the Zenoh Inspector with Topics and Network tabs",
                None,
                Provenance::source(ZENOH_VIEW, 53),
                None,
            ),
            operator_step(
                "Open a Zenoh session over WebSocket to /zenoh-api/",
                None,
                Provenance::source(ZENOH_LIB, 35),
                None,
            ),
            operator_step(
                "Switch to the Network tab and view the live topology graph of clients, routers, and peers",
                None,
                Provenance::source(ZENOH_NETWORK, 21),
                Some(live_outcome("live Zenoh network topology depends on connected nodes at runtime")),
            ),
            operator_step(
                "Switch to the Topics tab and browse the searchable live pub/sub topic list",
                None,
                Provenance::source(ZENOH_INSPECTOR, 18),
                Some(live_outcome("live topic list depends on active Zenoh publishers at runtime")),
            ),
            operator_step(
                "Select a topic to inspect its latest message payload",
                None,
                Provenance::source(ZENOH_INSPECTOR, 114),
                Some(live_outcome("topic message payload depends on live Zenoh traffic at runtime")),
            ),
        ]),
        availability: PRESENCE_INSPECT_ZENOH_NETWORK,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted(
                "Zenoh Inspector only connects over WebSocket to read live pub/sub topics and network topology",
            ),
        ),
        chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const ZENOHD_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Zenohd,
    Provenance::source(NGINX, 261),
)]);

const fn live_outcome(reason: &'static str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
}

const fn operator_step(
    description: &'static str,
    route: Option<Grounded<crate::journey::RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route,
            outcome,
        },
        provenance,
    )
}
