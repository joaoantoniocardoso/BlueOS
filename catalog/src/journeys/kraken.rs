use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, BodyKind, DataAssumption, HttpMethod, JourneyStep, NetworkState,
    Precondition, RouteRef, StepOutcome, UseCase, Visibility,
};
use crate::journey_presence::{
    PRESENCE_ADD_CUSTOM_MANIFEST, PRESENCE_BROWSE_EXTENSION_STORE,
    PRESENCE_CONFIGURE_INSTALLED_EXTENSION, PRESENCE_EDIT_EXTENSION_DEV_VERSION,
    PRESENCE_INSTALL_CUSTOM_EXTENSION, PRESENCE_INSTALL_EXTENSION, PRESENCE_UNINSTALL_EXTENSION,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const DEV: &str = "content/development/extensions/index.md";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4;

pub const JOURNEYS: &[UseCase] = &[
    ADD_CUSTOM_MANIFEST,
    BROWSE_EXTENSION_STORE,
    CONFIGURE_INSTALLED_EXTENSION,
    EDIT_EXTENSION_DEV_VERSION,
    INSTALL_CUSTOM_EXTENSION,
    INSTALL_EXTENSION,
    UNINSTALL_EXTENSION,
];

const ADD_CUSTOM_MANIFEST: UseCase =
    UseCase {
        id: JourneyId::AddCustomManifest,
        summary: Grounded::known(
            "Add an external extension collection manifest beyond the default BlueOS Extensions Repository",
            Provenance::doc(ADV, 854, "for available extensions, but it is also possible to specify"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 852, "By default, the store searches")),
        services: KRAKEN_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ManageManifests,
            "operator registers an external manifest source for the store",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(ADV, 853, "[BlueOS Extensions Repository](https://docs.bluerobotics.com"),
        )]),
        steps: GroundedSet::known(&[operator_step(
            "Specify your own external collection of extensions in the Extensions Manager store",
            Some(doc_route(HttpMethod::Post, "/manifest/", Some("v2.0"), ADV, 855, "of extensions:")),
            Provenance::doc(ADV, 855, "of extensions:"),
            Some(runtime_outcome(
                201,
                None,
                BodyKind::Payload,
                "runtime-captures/kraken__pi4_navigator_master.json#transitions",
            )),
        )]),
        availability: PRESENCE_ADD_CUSTOM_MANIFEST,
        blast_radius: Grounded::known(
            BlastRadius::Reversible,
            Provenance::asserted(
                "POST /manifest/ registers an external collection URL in Kraken store config",
            ),
        ),
        chains_from: Some(JourneyId::BrowseExtensionStore),
    };

const BROWSE_EXTENSION_STORE: UseCase = UseCase {
    id: JourneyId::BrowseExtensionStore,
    summary: Grounded::known(
        "Browse available extensions in the Store tab, including beta-marked releases",
        Provenance::doc(
            ADV,
            841,
            "The Store tab shows the available extensions, with a default",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(
            ADV,
            841,
            "The Store tab shows the available extensions, with a default",
        ),
    ),
    services: KRAKEN_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::BrowseExtensionStore,
        "Store tab lists extensions from configured manifests with default filters applied",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Network(NetworkState::Online),
        Provenance::doc(
            ADV,
            853,
            "[BlueOS Extensions Repository](https://docs.bluerobotics.com",
        ),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Extensions Manager Store tab",
            None,
            Provenance::doc(
                ADV,
                841,
                "The Store tab shows the available extensions, with a default",
            ),
            None,
        ),
        operator_step(
            "Browse extension cards; beta versions show a red marker on the card corner",
            Some(doc_route(
                HttpMethod::Get,
                "/manifest/consolidated",
                Some("v2.0"),
                ADV,
                842,
                "the development example extensions. Beta versions show a red",
            )),
            Provenance::doc(
                ADV,
                842,
                "the development example extensions. Beta versions show a red",
            ),
            Some(runtime_outcome(
                200,
                Some("large consolidated manifest of all extensions across sources"),
                BodyKind::Payload,
                "runtime-captures/kraken__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_BROWSE_EXTENSION_STORE,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "GET /manifest/consolidated lists store cards without mutating Kraken state",
        ),
    ),
    chains_from: None,
};

const CONFIGURE_INSTALLED_EXTENSION: UseCase =
    UseCase {
        id: JourneyId::ConfigureInstalledExtension,
        summary: Grounded::known(
            "Manage installed extensions: view resource usage, configure permissions, read logs, restart, or disable"
                ,
            Provenance::doc(ADV, 859, "configuring them, checking their logs, and restarting or dis"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 858, "The Installed tab shows the re")),
        services: KRAKEN_SERVICES,
        capability_refs: GroundedSet::known(&[
            cap(CapabilityId::ConfigureExtension,
                "Installed tab edits permissions and custom extension configuration",
            ),
            cap(CapabilityId::ManageExtensionLifecycle,
                "Installed tab restarts or disables running extensions",
            ),
        ]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Data(DataAssumption::ExtensionInstalled),
            Provenance::doc(DEV, 347, "Once installed on the [Onboard Computer](@/integrations/hard"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Extensions Manager Installed tab",
                None,
                Provenance::doc(ADV, 858, "The Installed tab shows the resource usage of the installed "),
                None,
            ),
            operator_step(
                "View CPU and memory resource usage for installed extensions",
                Some(doc_route(HttpMethod::Get, "/container/", Some("v2.0"), DEV, 355, "- Track CPU and memory usage")),
                Provenance::doc(DEV, 355, "- Track CPU and memory usage (per Extension)"),
                Some(runtime_outcome(
                    200,
                    Some("array of container descriptors {name,status,image}"),
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Configure extension permissions and custom settings",
                Some(doc_route(HttpMethod::Put, "/extension/{identifier}", Some("v2.0"), DEV, 356, "- Manage/edit pe")),
                Provenance::doc(DEV, 356, "- Manage/edit permissions (including limiting hardware resou"),
                None,
            ),
            operator_step(
                "View extension logs",
                Some(doc_route(HttpMethod::Get, "/container/{container_name}/log", Some("v2.0"), DEV, 357, "- View Exten")),
                Provenance::doc(DEV, 357, "- View Extension logs"),
                Some(runtime_outcome(
                    200,
                    Some("base64-encoded log fragments"),
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
            operator_step(
                "Restart an installed extension",
                Some(doc_route(HttpMethod::Post, "/extension/{identifier}/restart", Some("v2.0"), ADV, 859, "configuring ")),
                Provenance::doc(ADV, 859, "configuring them, checking their logs, and restarting or dis"),
                Some(runtime_outcome(
                    202,
                    None,
                    BodyKind::Empty,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
            operator_step(
                "Disable an installed extension",
                Some(doc_route(HttpMethod::Post, "/extension/{identifier}/disable", Some("v2.0"), ADV, 859, "configuring ")),
                Provenance::doc(ADV, 859, "configuring them, checking their logs, and restarting or dis"),
                Some(runtime_outcome(
                    204,
                    None,
                    BodyKind::Empty,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
        ]),
        availability: PRESENCE_CONFIGURE_INSTALLED_EXTENSION,
        blast_radius: Grounded::known(
            BlastRadius::Disruptive,
            Provenance::asserted(
                "Installed-tab PUT/POST restart or disable stops or recreates extension containers",
            ),
        ),
        chains_from: Some(JourneyId::InstallExtension),
    };

const EDIT_EXTENSION_DEV_VERSION: UseCase =
    UseCase {
        id: JourneyId::EditExtensionDevVersion,
        summary: Grounded::known(
            "Switch an installed extension to an alternative or development version by editing its docker tag",
            Provenance::doc(ADV, 866, "versions by setting the docker tag."),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 864, "{% pirate() %}")),
        services: KRAKEN_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::ManageExtensionLifecycle,
            "Edit button changes the docker tag to an alternative development version",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Data(DataAssumption::ExtensionInstalled),
            Provenance::doc(ADV, 865, "The \"Edit\" button on installed extension listings allows cha"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Click the Edit button on an installed extension listing",
                None,
                Provenance::doc(ADV, 865, "The \"Edit\" button on installed extension listings allows cha"),
                None,
            ),
            operator_step(
                "Set the docker tag to switch to the desired alternative or development version",
                Some(doc_route(HttpMethod::Put, "/extension/{identifier}/{tag}", Some("v2.0"), ADV, 866, "versions by ")),
                Provenance::doc(ADV, 866, "versions by setting the docker tag."),
                Some(runtime_outcome(
                    200,
                    None,
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
        ]),
        availability: PRESENCE_EDIT_EXTENSION_DEV_VERSION,
        blast_radius: Grounded::known(
            BlastRadius::Disruptive,
            Provenance::asserted(
                "PUT /extension/{identifier}/{tag} recreates the container on a different image tag",
            ),
        ),
        chains_from: Some(JourneyId::ConfigureInstalledExtension),
    };

const INSTALL_CUSTOM_EXTENSION: UseCase =
    UseCase {
        id: JourneyId::InstallCustomExtension,
        summary: Grounded::known(
            "Install a custom extension by registering a Docker image through the blue plus button",
            Provenance::doc(ADV, 863, "The blue \"+\" button in the bottom right corner allows instal"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 863, "The blue \"+\" button in the bot")),
        services: KRAKEN_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::InstallExtension,
            "blue plus button registers and installs a custom Docker image extension",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(DEV, 460, "1. Enter the relevant information for your Docker Image, so "),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Extensions Manager",
                None,
                Provenance::doc(DEV, 457, "1. Go to the [Extensions Manager](../../usage/advanced/#exte"),
                None,
            ),
            operator_step(
                "Open the Installed tab",
                None,
                Provenance::doc(DEV, 458, "1. Click on the \"Installed\" tab"),
                None,
            ),
            operator_step(
                "Click the blue plus icon in the bottom right corner",
                None,
                Provenance::doc(ADV, 863, "The blue \"+\" button in the bottom right corner allows instal"),
                None,
            ),
            operator_step(
                "Enter the extension identifier, name, Docker image, tag, and custom settings so the image can be fetched from Docker Hub",
                Some(doc_route(HttpMethod::Post, "/extension/", Some("v2.0"), DEV, 473, "- Used for configuration of")),
                Provenance::doc(DEV, 473, "- Used for configuration of the Docker container when it's r"),
                Some(runtime_outcome(
                    200,
                    Some("streams docker pull progress; container created"),
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
        ]),
        availability: PRESENCE_INSTALL_CUSTOM_EXTENSION,
        blast_radius: Grounded::known(
            BlastRadius::Disruptive,
            Provenance::asserted(
                "POST /extension/ pulls a Docker image and starts a new extension container",
            ),
        ),
        chains_from: None,
    };

const INSTALL_EXTENSION: UseCase =
    UseCase {
        id: JourneyId::InstallExtension,
        summary: Grounded::known(
            "Install an extension from the store by selecting a version from its card dropdown",
            Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 846, "Clicking an extension card dis")),
        services: KRAKEN_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::InstallExtension,
            "version dropdown on a store card installs the selected extension release",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(ADV, 853, "[BlueOS Extensions Repository](https://docs.bluerobotics.com"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Click an extension card to view developer information, default settings, permissions, and usage instructions",
                Some(doc_route(HttpMethod::Get, "/extension/{identifier}/details", Some("v2.0"), ADV, 846, "Clicking an ")),
                Provenance::doc(ADV, 846, "Clicking an extension card displays the developer informatio"),
                None,
            ),
            operator_step(
                "Select the extension version to install from the dropdown",
                None,
                Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
                None,
            ),
            operator_step(
                "Install the selected extension version",
                Some(doc_route(
                    HttpMethod::Post,
                    "/extension/{identifier}/{tag}/install",
                    Some("v2.0"),
                    ADV,
                    848,
                    "version of the extension to install (or uninstall):",
                )),
                Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
                Some(runtime_outcome(
                    200,
                    None,
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
        ]),
        availability: PRESENCE_INSTALL_EXTENSION,
        blast_radius: Grounded::known(
            BlastRadius::Disruptive,
            Provenance::asserted(
                "POST /extension/{identifier}/{tag}/install pulls and starts a store extension container",
            ),
        ),
        chains_from: Some(JourneyId::BrowseExtensionStore),
    };

const UNINSTALL_EXTENSION: UseCase = UseCase {
    id: JourneyId::UninstallExtension,
    summary: Grounded::known(
        "Uninstall an extension version from the store card version dropdown",
        Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 848, "version of the extension to instal")),
    services: KRAKEN_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::UninstallExtension,
        "version dropdown on a store card uninstalls the selected extension release",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Data(DataAssumption::ExtensionInstalled),
        Provenance::doc(DEV, 359, "- Uninstall Extensions that are no longer wanted"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open an extension card and select the installed version from the dropdown",
            None,
            Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
            None,
        ),
        operator_step(
            "Uninstall the selected extension version",
            Some(doc_route(HttpMethod::Delete, "/extension/{identifier}/{tag}", Some("v2.0"), ADV, 848, "version of t")),
            Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
            Some(runtime_outcome(
                202,
                None,
                BodyKind::Empty,
                "runtime-captures/kraken__pi4_navigator_master.json#transitions",
            )),
        ),
    ]),
    availability: PRESENCE_UNINSTALL_EXTENSION,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "DELETE /extension/{identifier}/{tag} removes the container and can be restored from the store",
        ),
    ),
    chains_from: Some(JourneyId::InstallExtension),
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const KRAKEN_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Kraken,
    Provenance::doc(
        ADV,
        836,
        "{{ service(service=\"Kraken\", port=9134, link=\"/services/krak",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Kraken,
        method,
        path,
        version,
    }
}

const fn doc_route(
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    file: &'static str,
    line: u32,
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::doc(file, line, anchor),
    )
}

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

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    body_kind: BodyKind,
    key: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            body_kind,
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}
