use crate::journey_presence::{
    PRESENCE_ADD_CUSTOM_MANIFEST, PRESENCE_BROWSE_EXTENSION_STORE,
    PRESENCE_CONFIGURE_INSTALLED_EXTENSION, PRESENCE_EDIT_EXTENSION_DEV_VERSION,
    PRESENCE_INSTALL_CUSTOM_EXTENSION, PRESENCE_INSTALL_EXTENSION, PRESENCE_UNINSTALL_EXTENSION,
};
use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, DataAssumption, HttpMethod, JourneyStep, NetworkState,
    Precondition, RouteRef, StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const DEV: &str = "content/development/extensions/index.md";
const V1_EXT: &str = "core/services/kraken/api/v1/routers/extension.py";
const V1_INDEX: &str = "core/services/kraken/api/v1/routers/index.py";
const V2_EXT: &str = "core/services/kraken/api/v2/routers/extension.py";
const V2_CONTAINER: &str = "core/services/kraken/api/v2/routers/container.py";
const V2_JOBS: &str = "core/services/kraken/api/v2/routers/jobs.py";
const V2_MANIFEST: &str = "core/services/kraken/api/v2/routers/manifest.py";
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
        steps: GroundedSet::known(&[
            operator_step(
                "Specify your own external collection of extensions in the Extensions Manager store",
                Some(sourced_route(
                    V2_MANIFEST,
                    HttpMethod::Post,
                    "/manifest/",
                    Some("v2.0"),
                    101,
                    "@manifest_router_v2.post(\"/\", status_code=status.HTTP_201_C",
                )),
                Provenance::doc(ADV, 855, "of extensions:"),
                Some(runtime_outcome(
                    201,
                    None,
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
            operator_step(
                "Open the newly added collection to confirm it registered",
                Some(sourced_route(
                    V2_MANIFEST,
                    HttpMethod::Get,
                    "/manifest/{identifier}/details",
                    Some("v2.0"),
                    58,
                    "@manifest_router_v2.get(\"/{identifier}/details\", status_code",
                )),
                Provenance::doc(ADV, 855, "of extensions:"),
                None,
            ),
            operator_step(
                "Enable the custom collection so its extensions appear in the store",
                Some(sourced_route(
                    V2_MANIFEST,
                    HttpMethod::Post,
                    "/manifest/{identifier}/enable",
                    Some("v2.0"),
                    110,
                    "@manifest_router_v2.post(\"/{identifier}/enable\", status_code",
                )),
                Provenance::doc(ADV, 855, "of extensions:"),
                None,
            ),
            operator_step(
                "Disable the custom collection without deleting it",
                Some(sourced_route(
                    V2_MANIFEST,
                    HttpMethod::Post,
                    "/manifest/{identifier}/disable",
                    Some("v2.0"),
                    119,
                    "@manifest_router_v2.post(\"/{identifier}/disable\", status_cod",
                )),
                Provenance::doc(ADV, 855, "of extensions:"),
                None,
            ),
            operator_step(
                "Rename or retarget the custom collection URL",
                Some(sourced_route(
                    V2_MANIFEST,
                    HttpMethod::Put,
                    "/manifest/{identifier}/details",
                    Some("v2.0"),
                    128,
                    "@manifest_router_v2.put(\"/{identifier}/details\", status_code",
                )),
                Provenance::doc(ADV, 855, "of extensions:"),
                None,
            ),
            operator_step(
                "Change the custom collection's search priority",
                Some(sourced_route(
                    V2_MANIFEST,
                    HttpMethod::Put,
                    "/manifest/{identifier}/order/{order}",
                    Some("v2.0"),
                    146,
                    "@manifest_router_v2.put(\"/{identifier}/order/{order}\", statu",
                )),
                Provenance::doc(ADV, 855, "of extensions:"),
                None,
            ),
            operator_step(
                "Restore factory-first collection order after experimenting with priority",
                Some(sourced_route(
                    V2_MANIFEST,
                    HttpMethod::Put,
                    "/manifest/orders",
                    Some("v2.0"),
                    137,
                    "@manifest_router_v2.put(\"/orders\", status_code=status.HTTP_2",
                )),
                Provenance::doc(ADV, 855, "of extensions:"),
                Some(source_outcome(
                    204,
                    BodyKind::Empty,
                    V2_MANIFEST,
                    137,
                    "@manifest_router_v2.put(\"/orders\", status_code=status.HTTP_2",
                )),
            ),
            operator_step(
                "Remove the custom collection when it is no longer wanted",
                Some(sourced_route(
                    V2_MANIFEST,
                    HttpMethod::Delete,
                    "/manifest/{identifier}",
                    Some("v2.0"),
                    155,
                    "@manifest_router_v2.delete(\"/{identifier}\", status_code=stat",
                )),
                Provenance::doc(ADV, 855, "of extensions:"),
                None,
            ),
        ]),
        availability: PRESENCE_ADD_CUSTOM_MANIFEST,
        blast_radius: Grounded::known(
            BlastRadius::Reversible,
            Provenance::asserted(
                "POST /manifest/ registers an external collection URL in store config",
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
            Some(sourced_route(
                V2_MANIFEST,
                HttpMethod::Get,
                "/manifest/consolidated",
                Some("v2.0"),
                67,
                "@manifest_router_v2.get(\"/consolidated\", status_code=status.",
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
        operator_step(
            "See which extensions are already installed while browsing the store",
            Some(sourced_route(
                V2_EXT,
                HttpMethod::Get,
                "/extension/",
                Some("v2.0"),
                48,
                "@extension_router_v2.get(\"/\", status_code=status.HTTP_200_OK",
            )),
            Provenance::doc(
                ADV,
                841,
                "The Store tab shows the available extensions, with a default",
            ),
            Some(source_outcome(
                200,
                BodyKind::Payload,
                V2_EXT,
                48,
                "@extension_router_v2.get(\"/\", status_code=status.HTTP_200_OK",
            )),
        ),
        operator_step(
            "List installed extensions through the v1 store door still served for the same store",
            Some(sourced_route(
                V1_INDEX,
                HttpMethod::Get,
                "/installed_extensions",
                Some("v1.0"),
                27,
                "@index_router_v1.get(\"/installed_extensions\", status_code=st",
            )),
            Provenance::doc(
                ADV,
                841,
                "The Store tab shows the available extensions, with a default",
            ),
            Some(source_outcome(
                200,
                BodyKind::Payload,
                V1_INDEX,
                27,
                "@index_router_v1.get(\"/installed_extensions\", status_code=st",
            )),
        ),
        operator_step(
            "Load the store catalog through the v1 store door still served for the same store",
            Some(sourced_route(
                V1_INDEX,
                HttpMethod::Get,
                "/extensions_manifest",
                Some("v1.0"),
                22,
                "@index_router_v1.get(\"/extensions_manifest\", status_code=sta",
            )),
            Provenance::doc(
                ADV,
                841,
                "The Store tab shows the available extensions, with a default",
            ),
            Some(source_outcome(
                200,
                BodyKind::Payload,
                V1_INDEX,
                22,
                "@index_router_v1.get(\"/extensions_manifest\", status_code=sta",
            )),
        ),
        operator_step(
            "See which extension collections the store is searching",
            Some(sourced_route(
                V2_MANIFEST,
                HttpMethod::Get,
                "/manifest/",
                Some("v2.0"),
                48,
                "@manifest_router_v2.get(\"/\", status_code=status.HTTP_200_OK)",
            )),
            Provenance::doc(ADV, 852, "By default, the store searches"),
            Some(source_outcome(
                200,
                BodyKind::Payload,
                V2_MANIFEST,
                48,
                "@manifest_router_v2.get(\"/\", status_code=status.HTTP_200_OK)",
            )),
        ),
        operator_step(
            "Open the default BlueOS Extensions Repository collection details",
            Some(sourced_route(
                V2_MANIFEST,
                HttpMethod::Get,
                "/manifest/{identifier}/details",
                Some("v2.0"),
                58,
                "@manifest_router_v2.get(\"/{identifier}/details\", status_code",
            )),
            Provenance::doc(
                ADV,
                853,
                "[BlueOS Extensions Repository](https://docs.bluerobotics.com",
            ),
            None,
        ),
        operator_step(
            "See available versions for an extension across collections",
            Some(sourced_route(
                V2_MANIFEST,
                HttpMethod::Get,
                "/manifest/tags/{extension_identifier}",
                Some("v2.0"),
                90,
                "@manifest_router_v2.get(\"/tags/{extension_identifier}\", stat",
            )),
            Provenance::doc(
                ADV,
                848,
                "version of the extension to install (or uninstall):",
            ),
            None,
        ),
        operator_step(
            "See available versions for an extension in a specific collection",
            Some(sourced_route(
                V2_MANIFEST,
                HttpMethod::Get,
                "/manifest/tags/{manifest_identifier}/{extension_identifier}/",
                Some("v2.0"),
                77,
                "@manifest_router_v2.get(\"/tags/{manifest_identifier}/{extens",
            )),
            Provenance::doc(
                ADV,
                848,
                "version of the extension to install (or uninstall):",
            ),
            None,
        ),
    ]),
    availability: PRESENCE_BROWSE_EXTENSION_STORE,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "GET /manifest/consolidated lists store cards without mutating store state",
        ),
    ),
    chains_from: None,
};

const CONFIGURE_INSTALLED_EXTENSION: UseCase =
    UseCase {
        id: JourneyId::ConfigureInstalledExtension,
        summary: Grounded::known(
            "Manage installed extensions: view resource usage, configure permissions, read logs, restart, disable, or enable"
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
                "Installed tab restarts, disables, or enables running extensions",
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
                Some(sourced_route(
                    V2_CONTAINER,
                    HttpMethod::Get,
                    "/container/",
                    Some("v2.0"),
                    33,
                    "@container_router_v2.get(\"/\", status_code=status.HTTP_200_OK",
                )),
                Provenance::doc(DEV, 355, "- Track CPU and memory usage (per Extension)"),
                Some(runtime_outcome(
                    200,
                    Some("array of container descriptors {name,status,image}"),
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "View CPU and memory stats for all running extension containers",
                Some(sourced_route(
                    V2_CONTAINER,
                    HttpMethod::Get,
                    "/container/stats",
                    Some("v2.0"),
                    66,
                    "@container_router_v2.get(\"/stats\", status_code=status.HTTP_2",
                )),
                Provenance::doc(DEV, 355, "- Track CPU and memory usage (per Extension)"),
                Some(source_outcome(
                    200,
                    BodyKind::Payload,
                    V2_CONTAINER,
                    66,
                    "@container_router_v2.get(\"/stats\", status_code=status.HTTP_2",
                )),
            ),
            operator_step(
                "View CPU and memory stats through the v1 store door still served for the Installed tab",
                Some(sourced_route(
                    V1_INDEX,
                    HttpMethod::Get,
                    "/stats",
                    Some("v1.0"),
                    52,
                    "@index_router_v1.get(\"/stats\", status_code=status.HTTP_200_O",
                )),
                Provenance::doc(DEV, 355, "- Track CPU and memory usage (per Extension)"),
                Some(source_outcome(
                    200,
                    BodyKind::Payload,
                    V1_INDEX,
                    52,
                    "@index_router_v1.get(\"/stats\", status_code=status.HTTP_200_O",
                )),
            ),
            operator_step(
                "List running extension containers through the v1 store door still served for the Installed tab",
                Some(sourced_route(
                    V1_INDEX,
                    HttpMethod::Get,
                    "/list_containers",
                    Some("v1.0"),
                    33,
                    "@index_router_v1.get(\"/list_containers\", status_code=status.",
                )),
                Provenance::doc(DEV, 355, "- Track CPU and memory usage (per Extension)"),
                Some(source_outcome(
                    200,
                    BodyKind::Payload,
                    V1_INDEX,
                    33,
                    "@index_router_v1.get(\"/list_containers\", status_code=status.",
                )),
            ),
            operator_step(
                "Inspect a single running extension container",
                Some(sourced_route(
                    V2_CONTAINER,
                    HttpMethod::Get,
                    "/container/{container_name}/details",
                    Some("v2.0"),
                    42,
                    "@container_router_v2.get(\"/{container_name}/details\", status",
                )),
                Provenance::doc(DEV, 355, "- Track CPU and memory usage (per Extension)"),
                None,
            ),
            operator_step(
                "View CPU and memory stats for a single running extension container",
                Some(sourced_route(
                    V2_CONTAINER,
                    HttpMethod::Get,
                    "/container/{container_name}/stats",
                    Some("v2.0"),
                    75,
                    "@container_router_v2.get(\"/{container_name}/stats\", status_c",
                )),
                Provenance::doc(DEV, 355, "- Track CPU and memory usage (per Extension)"),
                None,
            ),
            operator_step(
                "View extension logs",
                Some(sourced_route(
                    V2_CONTAINER,
                    HttpMethod::Get,
                    "/container/{container_name}/log",
                    Some("v2.0"),
                    51,
                    "@container_router_v2.get(\"/{container_name}/log\", status_cod",
                )),
                Provenance::doc(DEV, 357, "- View Extension logs"),
                Some(runtime_outcome(
                    200,
                    Some("base64-encoded log fragments"),
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
            operator_step(
                "View extension logs through the v1 store door still served for the Installed tab",
                Some(sourced_route(
                    V1_INDEX,
                    HttpMethod::Get,
                    "/log",
                    Some("v1.0"),
                    38,
                    "@index_router_v1.get(\"/log\", status_code=status.HTTP_200_OK,",
                )),
                Provenance::doc(DEV, 357, "- View Extension logs"),
                Some(source_outcome(
                    200,
                    BodyKind::Payload,
                    V1_INDEX,
                    38,
                    "@index_router_v1.get(\"/log\", status_code=status.HTTP_200_OK,",
                )),
            ),
            operator_step(
                "Restart an installed extension",
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Post,
                    "/extension/{identifier}/restart",
                    Some("v2.0"),
                    129,
                    "@extension_router_v2.post(\"/{identifier}/restart\", status_co",
                )),
                Provenance::doc(ADV, 859, "configuring them, checking their logs, and restarting or dis"),
                Some(runtime_outcome(
                    202,
                    None,
                    BodyKind::Empty,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
            operator_step(
                "Restart an installed extension through the v1 store door still served for the Installed tab",
                Some(sourced_route(
                    V1_EXT,
                    HttpMethod::Post,
                    "/extension/restart",
                    Some("v1.0"),
                    55,
                    "@extension_router_v1.post(\"/restart\", status_code=status.HTT",
                )),
                Provenance::doc(ADV, 859, "configuring them, checking their logs, and restarting or dis"),
                Some(source_outcome(
                    202,
                    BodyKind::Empty,
                    V1_EXT,
                    55,
                    "@extension_router_v1.post(\"/restart\", status_code=status.HTT",
                )),
            ),
            operator_step(
                "Disable an installed extension",
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Post,
                    "/extension/{identifier}/disable",
                    Some("v2.0"),
                    119,
                    "@extension_router_v2.post(\"/{identifier}/disable\", status_co",
                )),
                Provenance::doc(ADV, 859, "configuring them, checking their logs, and restarting or dis"),
                Some(runtime_outcome(
                    204,
                    None,
                    BodyKind::Empty,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
            operator_step(
                "Disable an installed extension through the v1 store door still served for the Installed tab",
                Some(sourced_route(
                    V1_EXT,
                    HttpMethod::Post,
                    "/extension/disable",
                    Some("v1.0"),
                    48,
                    "@extension_router_v1.post(\"/disable\", status_code=status.HTT",
                )),
                Provenance::doc(ADV, 859, "configuring them, checking their logs, and restarting or dis"),
                Some(source_outcome(
                    200,
                    BodyKind::Empty,
                    V1_EXT,
                    48,
                    "@extension_router_v1.post(\"/disable\", status_code=status.HTT",
                )),
            ),
            operator_step(
                "Enable a disabled extension",
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Post,
                    "/extension/{identifier}/{tag}/enable",
                    Some("v2.0"),
                    109,
                    "@extension_router_v2.post(\"/{identifier}/{tag}/enable\", stat",
                )),
                Provenance::doc(
                    DEV,
                    52,
                    "Once installed, an Extension Package can be run as a [Doc",
                ),
                Some(source_outcome(
                    204,
                    BodyKind::Empty,
                    V2_EXT,
                    109,
                    "@extension_router_v2.post(\"/{identifier}/{tag}/enable\", stat",
                )),
            ),
            operator_step(
                "Enable a disabled extension through the v1 store door still served for the Installed tab",
                Some(sourced_route(
                    V1_EXT,
                    HttpMethod::Post,
                    "/extension/enable",
                    Some("v1.0"),
                    41,
                    "@extension_router_v1.post(\"/enable\", status_code=status.HTTP",
                )),
                Provenance::doc(
                    DEV,
                    52,
                    "Once installed, an Extension Package can be run as a [Doc",
                ),
                Some(source_outcome(
                    200,
                    BodyKind::Empty,
                    V1_EXT,
                    41,
                    "@extension_router_v1.post(\"/enable\", status_code=status.HTTP",
                )),
            ),
            service_step(
                "List background jobs used to queue extension API work",
                Some(sourced_route(
                    V2_JOBS,
                    HttpMethod::Get,
                    "/jobs/",
                    Some("v2.0"),
                    42,
                    "@jobs_router_v2.get(\"/\", status_code=status.HTTP_200_OK)",
                )),
                Provenance::source(
                    V2_JOBS,
                    42,
                    "@jobs_router_v2.get(\"/\", status_code=status.HTTP_200_OK)",
                ),
                Some(source_outcome(
                    200,
                    BodyKind::Payload,
                    V2_JOBS,
                    42,
                    "@jobs_router_v2.get(\"/\", status_code=status.HTTP_200_OK)",
                )),
            ),
            service_step(
                "Enqueue a background job for a safe GET (never an install)",
                Some(sourced_route(
                    V2_JOBS,
                    HttpMethod::Post,
                    "/jobs/{route}",
                    Some("v2.0"),
                    32,
                    "@jobs_router_v2.post(\"/{route:path}\", status_code=status.HTT",
                )),
                Provenance::source(
                    V2_JOBS,
                    32,
                    "@jobs_router_v2.post(\"/{route:path}\", status_code=status.HTT",
                ),
                None,
            ),
            service_step(
                "Inspect a background job by id",
                Some(sourced_route(
                    V2_JOBS,
                    HttpMethod::Get,
                    "/jobs/{identifier}",
                    Some("v2.0"),
                    48,
                    "@jobs_router_v2.get(\"/{identifier}\", status_code=status.HTTP",
                )),
                Provenance::source(
                    V2_JOBS,
                    48,
                    "@jobs_router_v2.get(\"/{identifier}\", status_code=status.HTTP",
                ),
                None,
            ),
            service_step(
                "Cancel a queued background job",
                Some(sourced_route(
                    V2_JOBS,
                    HttpMethod::Delete,
                    "/jobs/{identifier}",
                    Some("v2.0"),
                    54,
                    "@jobs_router_v2.delete(\"/{identifier}\", status_code=status.H",
                )),
                Provenance::source(
                    V2_JOBS,
                    54,
                    "@jobs_router_v2.delete(\"/{identifier}\", status_code=status.H",
                ),
                None,
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
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Put,
                    "/extension/{identifier}/{tag}",
                    Some("v2.0"),
                    150,
                    "@extension_router_v2.put(\"/{identifier}/{tag}\", status_code=",
                )),
                Provenance::doc(ADV, 866, "versions by setting the docker tag."),
                Some(runtime_outcome(
                    200,
                    None,
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
            operator_step(
                "Switch version through the v1 store door still served for the same Edit flow",
                Some(sourced_route(
                    V1_EXT,
                    HttpMethod::Post,
                    "/extension/update_to_version",
                    Some("v1.0"),
                    34,
                    "@extension_router_v1.post(\"/update_to_version\", status_code=",
                )),
                Provenance::doc(ADV, 866, "versions by setting the docker tag."),
                Some(source_outcome(
                    201,
                    BodyKind::Payload,
                    V1_EXT,
                    34,
                    "@extension_router_v1.post(\"/update_to_version\", status_code=",
                )),
            ),
            operator_step(
                "Update the installed extension to the latest compatible store version",
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Put,
                    "/extension/{identifier}",
                    Some("v2.0"),
                    139,
                    "@extension_router_v2.put(\"/{identifier}\", status_code=status",
                )),
                Provenance::doc(ADV, 866, "versions by setting the docker tag."),
                None,
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
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Post,
                    "/extension/",
                    Some("v2.0"),
                    78,
                    "@extension_router_v2.post(\"/\", status_code=status.HTTP_201_C",
                )),
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
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Get,
                    "/extension/{identifier}/details",
                    Some("v2.0"),
                    58,
                    "@extension_router_v2.get(\"/{identifier}/details\", status_cod",
                )),
                Provenance::doc(ADV, 846, "Clicking an extension card displays the developer informatio"),
                None,
            ),
            operator_step(
                "Confirm the selected version details before installing",
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Get,
                    "/extension/{identifier}/{tag}/details",
                    Some("v2.0"),
                    68,
                    "@extension_router_v2.get(\"/{identifier}/{tag}/details\", stat",
                )),
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
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Post,
                    "/extension/{identifier}/{tag}/install",
                    Some("v2.0"),
                    99,
                    "@extension_router_v2.post(\"/{identifier}/{tag}/install\", sta",
                )),
                Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
                Some(runtime_outcome(
                    200,
                    None,
                    BodyKind::Payload,
                    "runtime-captures/kraken__pi4_navigator_master.json#transitions",
                )),
            ),
            operator_step(
                "Install the latest compatible version from the store without picking a tag",
                Some(sourced_route(
                    V2_EXT,
                    HttpMethod::Post,
                    "/extension/{identifier}/install",
                    Some("v2.0"),
                    89,
                    "@extension_router_v2.post(\"/{identifier}/install\", status_co",
                )),
                Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
                None,
            ),
            operator_step(
                "Install the selected extension through the v1 store door still served for the store card",
                Some(sourced_route(
                    V1_EXT,
                    HttpMethod::Post,
                    "/extension/install",
                    Some("v1.0"),
                    20,
                    "@extension_router_v1.post(\"/install\", status_code=status.HTT",
                )),
                Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
                Some(source_outcome(
                    201,
                    BodyKind::Payload,
                    V1_EXT,
                    20,
                    "@extension_router_v1.post(\"/install\", status_code=status.HTT",
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
            Some(sourced_route(
                V2_EXT,
                HttpMethod::Delete,
                "/extension/{identifier}/{tag}",
                Some("v2.0"),
                171,
                "@extension_router_v2.delete(\"/{identifier}/{tag}\", status_co",
            )),
            Provenance::doc(ADV, 848, "version of the extension to install (or uninstall):"),
            Some(runtime_outcome(
                202,
                None,
                BodyKind::Empty,
                "runtime-captures/kraken__pi4_navigator_master.json#transitions",
            )),
        ),
        operator_step(
            "Uninstall every installed version of the extension",
            Some(sourced_route(
                V2_EXT,
                HttpMethod::Delete,
                "/extension/{identifier}",
                Some("v2.0"),
                161,
                "@extension_router_v2.delete(\"/{identifier}\", status_code=sta",
            )),
            Provenance::doc(DEV, 359, "- Uninstall Extensions that are no longer wanted"),
            None,
        ),
        operator_step(
            "Uninstall the extension through the v1 store door still served for the store card",
            Some(sourced_route(
                V1_EXT,
                HttpMethod::Post,
                "/extension/uninstall",
                Some("v1.0"),
                27,
                "@extension_router_v1.post(\"/uninstall\", status_code=status.H",
            )),
            Provenance::doc(DEV, 359, "- Uninstall Extensions that are no longer wanted"),
            Some(source_outcome(
                200,
                BodyKind::Empty,
                V1_EXT,
                27,
                "@extension_router_v1.post(\"/uninstall\", status_code=status.H",
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

const fn sourced_route(
    file: &'static str,
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    line: u32,
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(file, line, anchor),
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

const fn service_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId::Kraken),
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

const fn source_outcome(
    status: u16,
    body_kind: BodyKind,
    file: &'static str,
    line: u32,
    anchor: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind,
            transition: None,
        },
        Provenance::source(file, line, anchor),
    )
}
