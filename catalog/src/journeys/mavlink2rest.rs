use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const MAVLINK_INSPECTOR_MENUS: &str = "core/frontend/src/menus.ts";
const MAVLINK_INSPECTOR_VIEW: &str = "core/frontend/src/views/MavlinkInspectorView.vue";

pub const JOURNEYS: &[UserJourney] = &[INSPECT_MAVLINK_MESSAGES_IN_BROWSER];

const INSPECT_MAVLINK_MESSAGES_IN_BROWSER: UserJourney =
    UserJourney {
        id: JourneyId::InspectMavlinkMessagesInBrowser,
        summary: Grounded::known(
            "See and inspect MAVLink messages in real time from the browser",
            Provenance::doc(OVERVIEW, 122),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 496)),
        services: MAVLINK2REST_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::InspectLiveMavlinkMessages,
            "MAVLink Inspector filters, lists, and expands live MAVLink messages from the vehicle stream",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Other("Advanced mode enabled to access the MAVLink Inspector page"),
            Provenance::source(MAVLINK_INSPECTOR_MENUS, 73),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the MAVLink Inspector page from the sidebar",
                None,
                Provenance::source(MAVLINK_INSPECTOR_MENUS, 70),
                None,
            ),
            operator_step(
                "Load the MAVLink Inspector interface",
                None,
                Provenance::source(MAVLINK_INSPECTOR_VIEW, 3),
                None,
            ),
            operator_step(
                "Filter for particular MAVLink messages",
                None,
                Provenance::doc(ADV, 504),
                None,
            ),
            operator_step(
                "View past and current MAVLink messages",
                None,
                Provenance::doc(ADV, 505),
                None,
            ),
            operator_step(
                "Click on a message to see its full details",
                None,
                Provenance::doc(ADV, 506),
                None,
            ),
        ]),
        chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const MAVLINK2REST_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Mavlink2rest,
    Provenance::doc(ADV, 499),
)]);

const fn operator_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
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
