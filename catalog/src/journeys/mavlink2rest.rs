use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const MAVLINK_INSPECTOR_MENUS: &str = "core/frontend/src/menus.ts";
const MAVLINK_INSPECTOR_VIEW: &str = "core/frontend/src/views/MavlinkInspectorView.vue";

pub fn journeys() -> Vec<UserJourney> {
    vec![inspect_mavlink_messages_in_browser()]
}

fn inspect_mavlink_messages_in_browser() -> UserJourney {
    UserJourney {
        id: JourneyId("inspect_mavlink_messages_in_browser".into()),
        summary: Grounded::known(
            "See and inspect MAVLink messages in real time from the browser".into(),
            Provenance::doc(OVERVIEW, 122),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 496)),
        services: mavlink2rest_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "inspect_live_mavlink_messages",
            "MAVLink Inspector filters, lists, and expands live MAVLink messages from the vehicle stream",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Advanced mode enabled to access the MAVLink Inspector page".into()),
            Provenance::source(MAVLINK_INSPECTOR_MENUS, 73),
        )]),
        steps: GroundedSet::known(vec![
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
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn mavlink2rest_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Mavlink2rest,
        Provenance::doc(ADV, 499),
    )])
}

fn operator_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
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
