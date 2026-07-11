use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef, StepOutcome, UserJourney,
    Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const DEV: &str = "content/development/extensions/index.md";
const RUNTIME_CAPTURE: &str = "runtime-captures/kraken__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4";

pub fn journeys() -> Vec<UserJourney> {
    vec![
        add_custom_manifest(),
        browse_extension_store(),
        configure_installed_extension(),
        edit_extension_dev_version(),
        install_custom_extension(),
        install_extension(),
        uninstall_extension(),
    ]
}

fn add_custom_manifest() -> UserJourney {
    UserJourney {
        id: JourneyId("add_custom_manifest".into()),
        summary: Grounded::known(
            "Add an external extension collection manifest beyond the default BlueOS Extensions Repository".into(),
            Provenance::doc(ADV, 854),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 852)),
        services: kraken_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "manage_manifests",
            "operator registers an external manifest source for the store",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(ADV, 853),
        )]),
        steps: GroundedSet::known(vec![operator_step(
            "Specify your own external collection of extensions in the Extensions Manager store",
            Some(doc_route(HttpMethod::Post, "/manifest/", None, ADV, 855)),
            Provenance::doc(ADV, 855),
            Some(runtime_outcome(201, None, "#transitions")),
        )]),
        chains_from: Some(JourneyId("browse_extension_store".into())),
    }
}

fn browse_extension_store() -> UserJourney {
    UserJourney {
        id: JourneyId("browse_extension_store".into()),
        summary: Grounded::known(
            "Browse available extensions in the Store tab, including beta-marked releases".into(),
            Provenance::doc(ADV, 841),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 841)),
        services: kraken_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "browse_extension_store",
            "Store tab lists extensions from configured manifests with default filters applied",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(ADV, 853),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Extensions Manager Store tab",
                None,
                Provenance::doc(ADV, 841),
                None,
            ),
            operator_step(
                "Browse extension cards; beta versions show a red marker on the card corner",
                Some(doc_route(
                    HttpMethod::Get,
                    "/manifest/consolidated",
                    None,
                    ADV,
                    842,
                )),
                Provenance::doc(ADV, 842),
                Some(runtime_outcome(
                    200,
                    Some("large consolidated manifest of all extensions across sources".into()),
                    "#running_baseline",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn configure_installed_extension() -> UserJourney {
    UserJourney {
        id: JourneyId("configure_installed_extension".into()),
        summary: Grounded::known(
            "Manage installed extensions: view resource usage, configure permissions, read logs, restart, or disable"
                .into(),
            Provenance::doc(ADV, 859),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 858)),
        services: kraken_services(),
        capability_refs: GroundedSet::known(vec![
            cap(
                "configure_extension",
                "Installed tab edits permissions and custom extension configuration",
            ),
            cap(
                "manage_extension_lifecycle",
                "Installed tab restarts or disables running extensions",
            ),
        ]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("At least one extension is installed".into()),
            Provenance::doc(DEV, 347),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Extensions Manager Installed tab",
                None,
                Provenance::doc(ADV, 858),
                None,
            ),
            operator_step(
                "View CPU and memory resource usage for installed extensions",
                Some(doc_route(HttpMethod::Get, "/container/", None, DEV, 355)),
                Provenance::doc(DEV, 355),
                Some(runtime_outcome(
                    200,
                    Some("array of container descriptors {name,status,image}".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Configure extension permissions and custom settings",
                Some(doc_route(HttpMethod::Put, "/extension/{identifier}", None, DEV, 356)),
                Provenance::doc(DEV, 356),
                None,
            ),
            operator_step(
                "View extension logs",
                Some(doc_route(
                    HttpMethod::Get,
                    "/container/{container_name}/log",
                    None,
                    DEV,
                    357,
                )),
                Provenance::doc(DEV, 357),
                Some(runtime_outcome(
                    200,
                    Some("base64-encoded log fragments".into()),
                    "#transitions",
                )),
            ),
            operator_step(
                "Restart an installed extension",
                Some(doc_route(
                    HttpMethod::Post,
                    "/extension/{identifier}/restart",
                    None,
                    ADV,
                    859,
                )),
                Provenance::doc(ADV, 859),
                Some(runtime_outcome(202, None, "#transitions")),
            ),
            operator_step(
                "Disable an installed extension",
                Some(doc_route(
                    HttpMethod::Post,
                    "/extension/{identifier}/disable",
                    None,
                    ADV,
                    859,
                )),
                Provenance::doc(ADV, 859),
                Some(runtime_outcome(204, None, "#transitions")),
            ),
        ]),
        chains_from: Some(JourneyId("install_extension".into())),
    }
}

fn edit_extension_dev_version() -> UserJourney {
    UserJourney {
        id: JourneyId("edit_extension_dev_version".into()),
        summary: Grounded::known(
            "Switch an installed extension to an alternative or development version by editing its docker tag".into(),
            Provenance::doc(ADV, 866),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 864)),
        services: kraken_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "manage_extension_lifecycle",
            "Edit button changes the docker tag to an alternative development version",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("The extension is listed on the Installed tab".into()),
            Provenance::doc(ADV, 865),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Click the Edit button on an installed extension listing",
                None,
                Provenance::doc(ADV, 865),
                None,
            ),
            operator_step(
                "Set the docker tag to switch to the desired alternative or development version",
                Some(doc_route(
                    HttpMethod::Put,
                    "/extension/{identifier}/{tag}",
                    None,
                    ADV,
                    866,
                )),
                Provenance::doc(ADV, 866),
                Some(runtime_outcome(200, None, "#transitions")),
            ),
        ]),
        chains_from: Some(JourneyId("configure_installed_extension".into())),
    }
}

fn install_custom_extension() -> UserJourney {
    UserJourney {
        id: JourneyId("install_custom_extension".into()),
        summary: Grounded::known(
            "Install a custom extension by registering a Docker image through the blue plus button".into(),
            Provenance::doc(ADV, 863),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 863)),
        services: kraken_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "install_extension",
            "blue plus button registers and installs a custom Docker image extension",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(DEV, 460),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Extensions Manager",
                None,
                Provenance::doc(DEV, 457),
                None,
            ),
            operator_step(
                "Open the Installed tab",
                None,
                Provenance::doc(DEV, 458),
                None,
            ),
            operator_step(
                "Click the blue plus icon in the bottom right corner",
                None,
                Provenance::doc(ADV, 863),
                None,
            ),
            operator_step(
                "Enter the extension identifier, name, Docker image, tag, and custom settings so the image can be fetched from Docker Hub",
                Some(doc_route(HttpMethod::Post, "/extension/", None, DEV, 473)),
                Provenance::doc(DEV, 473),
                Some(runtime_outcome(
                    200,
                    Some("streams docker pull progress; container created".into()),
                    "#transitions",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn install_extension() -> UserJourney {
    UserJourney {
        id: JourneyId("install_extension".into()),
        summary: Grounded::known(
            "Install an extension from the store by selecting a version from its card dropdown".into(),
            Provenance::doc(ADV, 848),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 846)),
        services: kraken_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "install_extension",
            "version dropdown on a store card installs the selected extension release",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(ADV, 853),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Click an extension card to view developer information, default settings, permissions, and usage instructions",
                Some(doc_route(
                    HttpMethod::Get,
                    "/extension/{identifier}/details",
                    None,
                    ADV,
                    846,
                )),
                Provenance::doc(ADV, 846),
                None,
            ),
            operator_step(
                "Select the extension version to install from the dropdown",
                None,
                Provenance::doc(ADV, 848),
                None,
            ),
            operator_step(
                "Install the selected extension version",
                Some(doc_route(
                    HttpMethod::Post,
                    "/extension/{identifier}/{tag}/install",
                    None,
                    ADV,
                    848,
                )),
                Provenance::doc(ADV, 848),
                Some(runtime_outcome(200, None, "#transitions")),
            ),
        ]),
        chains_from: Some(JourneyId("browse_extension_store".into())),
    }
}

fn uninstall_extension() -> UserJourney {
    UserJourney {
        id: JourneyId("uninstall_extension".into()),
        summary: Grounded::known(
            "Uninstall an extension version from the store card version dropdown".into(),
            Provenance::doc(ADV, 848),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 848)),
        services: kraken_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "uninstall_extension",
            "version dropdown on a store card uninstalls the selected extension release",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("The extension version is installed".into()),
            Provenance::doc(DEV, 359),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open an extension card and select the installed version from the dropdown",
                None,
                Provenance::doc(ADV, 848),
                None,
            ),
            operator_step(
                "Uninstall the selected extension version",
                Some(doc_route(
                    HttpMethod::Delete,
                    "/extension/{identifier}/{tag}",
                    None,
                    ADV,
                    848,
                )),
                Provenance::doc(ADV, 848),
                Some(runtime_outcome(202, None, "#transitions")),
            ),
        ]),
        chains_from: Some(JourneyId("install_extension".into())),
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn kraken_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Kraken,
        Provenance::doc(ADV, 836),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Kraken,
        method,
        path: path.into(),
        version: version.map(str::to_string),
    }
}

fn doc_route(
    method: HttpMethod,
    path: &str,
    version: Option<&str>,
    file: &str,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(route(method, path, version), Provenance::doc(file, line))
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

fn runtime_outcome(status: u16, body: Option<String>, key: &str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            transition: None,
        },
        Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV),
    )
}
