use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{Actor, JourneyStep, Precondition, StepOutcome, UserJourney, Visibility};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ZENOH_MENUS: &str = "core/frontend/src/menus.ts";
const ZENOH_VIEW: &str = "core/frontend/src/views/ZenohInspectorView.vue";
const ZENOH_NETWORK: &str = "core/frontend/src/components/zenoh-inspector/ZenohNetwork.vue";
const ZENOH_INSPECTOR: &str = "core/frontend/src/components/zenoh-inspector/ZenohInspector.vue";
const ZENOH_LIB: &str = "core/frontend/src/libs/zenoh/index.ts";
const NGINX: &str = "core/tools/nginx/nginx.conf";

pub fn journeys() -> Vec<UserJourney> {
    vec![inspect_zenoh_network()]
}

fn inspect_zenoh_network() -> UserJourney {
    UserJourney {
        id: JourneyId::InspectZenohNetwork,
        summary: Grounded::known(
            "View detailed Zenoh traffic coming from your vehicle".into(),
            Provenance::source(ZENOH_MENUS, 146),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(ZENOH_MENUS, 145)),
        services: zenohd_services(),
        capability_refs: GroundedSet::known(vec![cap(CapabilityId::InspectZenohNetwork,
            "Zenoh Inspector connects over WebSocket to inspect live pub/sub topics and network topology",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Advanced mode enabled to access the Zenoh Inspector page".into()),
            Provenance::source(ZENOH_MENUS, 145),
        )]),
        steps: GroundedSet::known(vec![
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
        chains_from: None,
    }
}

fn cap(id: CapabilityId, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

fn zenohd_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Zenohd,
        Provenance::source(NGINX, 261),
    )])
}

fn live_outcome(reason: &str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
}

fn operator_step(
    description: &str,
    route: Option<Grounded<crate::journey::RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}
