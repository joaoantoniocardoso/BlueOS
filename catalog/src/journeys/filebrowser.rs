use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{Actor, JourneyStep, Precondition, UserJourney, Visibility};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const FILE_BROWSER_MENUS: &str = "core/frontend/src/menus.ts";
const FILE_BROWSER_VIEW: &str = "core/frontend/src/views/FileBrowserView.vue";

pub fn journeys() -> Vec<UserJourney> {
    vec![manage_blueos_files()]
}

fn manage_blueos_files() -> UserJourney {
    UserJourney {
        id: JourneyId("manage_blueos_files".into()),
        summary: Grounded::known(
            "View, edit, download, and upload BlueOS files using the web File Browser".into(),
            Provenance::doc(ADV, 431),
        ),
        visibility: Grounded::known(
            Visibility::Advanced,
            Provenance::source(FILE_BROWSER_MENUS, 42),
        ),
        services: filebrowser_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "manage_blueos_files",
            "File Browser page embeds the upstream filebrowser SPA at /file-browser/ for viewing, editing, downloading, and uploading files",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Advanced mode enabled to access the File Browser page".into()),
            Provenance::source(FILE_BROWSER_MENUS, 42),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the File Browser page from the sidebar",
                None,
                Provenance::source(FILE_BROWSER_MENUS, 39),
                None,
            ),
            operator_step(
                "Load the embedded filebrowser web app at /file-browser/",
                None,
                Provenance::source(FILE_BROWSER_VIEW, 19),
                None,
            ),
            operator_step(
                "View BlueOS files in the browser",
                None,
                Provenance::doc(ADV, 431),
                None,
            ),
            operator_step(
                "Edit BlueOS files in the browser",
                None,
                Provenance::doc(ADV, 431),
                None,
            ),
            operator_step(
                "Download BlueOS files from the browser",
                None,
                Provenance::doc(ADV, 431),
                None,
            ),
            operator_step(
                "Upload files to BlueOS from the browser",
                None,
                Provenance::doc(ADV, 431),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn filebrowser_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Filebrowser,
        Provenance::doc(ADV, 429),
    )])
}

fn operator_step(
    description: &str,
    route: Option<Grounded<crate::journey::RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<crate::journey::StepOutcome>>,
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
