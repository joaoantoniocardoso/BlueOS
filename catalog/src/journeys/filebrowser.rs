use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, JourneyStep, Precondition, SoftwareAssumption, UseCase, Visibility,
};
use crate::journey_presence::PRESENCE_MANAGE_BLUEOS_FILES;
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const FILE_BROWSER_MENUS: &str = "core/frontend/src/menus.ts";
const FILE_BROWSER_VIEW: &str = "core/frontend/src/views/FileBrowserView.vue";

pub const JOURNEYS: &[UseCase] = &[MANAGE_BLUEOS_FILES];

const MANAGE_BLUEOS_FILES: UseCase =
    UseCase {
        id: JourneyId::ManageBlueosFiles,
        summary: Grounded::known(
            "View, edit, download, and upload BlueOS files using the web File Browser",
            Provenance::doc(ADV, 431, "The File Browser allows viewing, editing, downloading, and u"),
        ),
        visibility: Grounded::known(
            Visibility::Advanced,
            Provenance::source(FILE_BROWSER_MENUS, 42, "advanced: true,"),
        ),
        services: FILEBROWSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ManageBlueosFiles,
            "File Browser page embeds the upstream filebrowser SPA at /file-browser/ for viewing, editing, downloading, and uploading files",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Software(SoftwareAssumption::AdvancedMode),
            Provenance::source(FILE_BROWSER_MENUS, 42, "advanced: true,"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the File Browser page from the sidebar",
                None,
                Provenance::source(FILE_BROWSER_MENUS, 39, "title: 'Feature Provenance',"),
                None,
            ),
            operator_step(
                "Load the embedded filebrowser web app at /file-browser/",
                None,
                Provenance::source(FILE_BROWSER_VIEW, 19, "service_path: '/file-browser/',"),
                None,
            ),
            operator_step(
                "View BlueOS files in the browser",
                None,
                Provenance::doc(ADV, 431, "The File Browser allows viewing, editing, downloading, and u"),
                None,
            ),
            operator_step(
                "Edit BlueOS files in the browser",
                None,
                Provenance::doc(ADV, 431, "The File Browser allows viewing, editing, downloading, and u"),
                None,
            ),
            operator_step(
                "Download BlueOS files from the browser",
                None,
                Provenance::doc(ADV, 431, "The File Browser allows viewing, editing, downloading, and u"),
                None,
            ),
            operator_step(
                "Upload files to BlueOS from the browser",
                None,
                Provenance::doc(ADV, 431, "The File Browser allows viewing, editing, downloading, and u"),
                None,
            ),
        ]),
        availability: PRESENCE_MANAGE_BLUEOS_FILES,
        blast_radius: Grounded::known(
            BlastRadius::Disruptive,
            Provenance::asserted(
                "embedded filebrowser SPA can edit, replace, or upload arbitrary host filesystem paths",
            ),
        ),
        chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const FILEBROWSER_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Filebrowser,
    Provenance::doc(
        ADV,
        429,
        "{{ service(service=\"File Browser\", port=7777, link=\"https://",
    ),
)]);

const fn operator_step(
    description: &'static str,
    route: Option<Grounded<crate::journey::RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<crate::journey::StepOutcome>>,
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
