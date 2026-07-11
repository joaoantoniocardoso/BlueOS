use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{Actor, JourneyStep, Precondition, StepOutcome, UserJourney, Visibility};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const TERMINAL_MENUS: &str = "core/frontend/src/menus.ts";
const TERMINAL_VIEW: &str = "core/frontend/src/views/TerminalView.vue";
const NGINX: &str = "core/tools/nginx/nginx.conf";

pub fn journeys() -> Vec<UserJourney> {
    vec![access_web_terminal()]
}

fn access_web_terminal() -> UserJourney {
    UserJourney {
        id: JourneyId("access_web_terminal".into()),
        summary: Grounded::known(
            "Access a web-based terminal with tmux session and direct access into the core BlueOS docker container"
                .into(),
            Provenance::doc(ADV, 623),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(TERMINAL_MENUS, 117)),
        services: ttyd_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "access_web_terminal",
            "Terminal page embeds ttyd web terminal over WebSocket at /terminal/ attached to user_terminal tmux",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Advanced mode enabled to access the Terminal page".into()),
            Provenance::source(TERMINAL_MENUS, 117),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Terminal page from the sidebar",
                None,
                Provenance::source(TERMINAL_MENUS, 114),
                None,
            ),
            operator_step(
                "Load the embedded web terminal connecting to /terminal/",
                None,
                Provenance::source(TERMINAL_VIEW, 19),
                None,
            ),
            operator_step(
                "Connect over WebSocket to /terminal/ proxied to ttyd on port 8088",
                None,
                Provenance::source(NGINX, 216),
                None,
            ),
            operator_step(
                "Use the interactive Linux shell attached to the persistent user_terminal tmux session",
                None,
                Provenance::doc(OVERVIEW, 125),
                Some(live_outcome(
                    "interactive terminal session cannot be captured by a simple HTTP probe",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn ttyd_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("ttyd".into()),
        Provenance::doc(ADV, 620),
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
