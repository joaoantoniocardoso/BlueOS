use crate::journey_presence::{
    PRESENCE_DELETE_LOCAL_BLUEOS_VERSION, PRESENCE_DOCKER_REGISTRY_LOGIN,
    PRESENCE_PULL_BLUEOS_VERSION_WITHOUT_SWITCH, PRESENCE_SWITCH_LOCAL_BLUEOS_VERSION,
    PRESENCE_UPDATE_BLUEOS_VERSION, PRESENCE_UPDATE_BOOTSTRAP_IMAGE,
};
use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, DataAssumption, HttpMethod, JourneyStep, NetworkState,
    Precondition, RouteRef, SoftwareAssumption, StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const GETTING: &str = "content/usage/getting-started/index.md";
const BOOTSTRAP: &str = "content/development/bootstrap/index.md";
const VERSION_ROUTER: &str = "core/services/versionchooser/api/v1/routers/version.py";
const BOOTSTRAP_ROUTER: &str = "core/services/versionchooser/api/v1/routers/bootstrap.py";
const DOCKER_ROUTER: &str = "core/services/versionchooser/api/v1/routers/docker.py";
const VC_COMPONENT: &str = "core/frontend/src/components/version-chooser/VersionChooser.vue";
const VC_UTILS: &str = "core/frontend/src/utils/version_chooser.ts";
const DOCKER_LOGIN: &str = "core/frontend/src/components/version-chooser/DockerLogin.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const BR_CORE_SWITCH: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "POST /version/current switches the running BlueOS core image and restarts the stack",
    ),
);

pub const JOURNEYS: &[UseCase] = &[
    UPDATE_BLUEOS_VERSION,
    SWITCH_LOCAL_BLUEOS_VERSION,
    PULL_BLUEOS_VERSION_WITHOUT_SWITCH,
    DELETE_LOCAL_BLUEOS_VERSION,
    DOCKER_REGISTRY_LOGIN,
    UPDATE_BOOTSTRAP_IMAGE,
];

const UPDATE_BLUEOS_VERSION: UseCase =
    UseCase {
        id: JourneyId::UpdateBlueosVersion,
        summary: Grounded::known(
            "Update BlueOS to the latest available release that is as stable or more stable than the current install"
                ,
            Provenance::doc(ADV, 397, "- The simplified interface provides an easy way to update to"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 397, "- The simplified interface pro")),
        services: VERSIONCHOOSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::UpdateBlueosVersion,
            "simplified Version Chooser pulls and applies a newer stable or beta release",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(GETTING, 104, "Now that your BlueOS has an internet connection, you can per"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open Settings and select BlueOS Version",
                None,
                Provenance::doc(GETTING, 109, "1. Under **Settings**, select [**BlueOS Version**](../advanc"),
                None,
            ),
            operator_step(
                "Review the current running version and whether an update button is shown",
                Some(sourced_route(HttpMethod::Get, "/version/current", Some("v1.0"), VERSION_ROUTER, 26, "")),
                Provenance::doc(GETTING, 112, "1. If you're already on the latest version, the right side o"),
                Some(runtime_outcome(
                    200,
                    None,
                    BodyKind::Payload,
                    "runtime-captures/versionchooser__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Fetch remote stable, beta, and master tags to determine the offered upgrade",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/version/available/{repository}/{image}",
                    Some("v1.0"),
                    VERSION_ROUTER,
                    60,
                    "@version_router_v1.get(",
                )),
                Provenance::doc(ADV, 397, "- The simplified interface provides an easy way to update to"),
                None,
            ),
            operator_step(
                "Click the update button to download the newer BlueOS core image",
                Some(sourced_route(HttpMethod::Post, "/version/pull", Some("v1.0"), VERSION_ROUTER, 41, "@version_rou")),
                Provenance::doc(GETTING, 116, "1. Once the update button is clicked the update process will"),
                Some(source_outcome(200, VERSION_ROUTER, 41, "@version_router_v1.post(\"/pull\", summary=\"Pulls a v")),
            ),
            operator_step(
                "Switch BlueOS core to the downloaded version and restart",
                Some(sourced_route(HttpMethod::Post, "/version/current", Some("v1.0"), VERSION_ROUTER, 34, "@version_rou")),
                Provenance::source(VC_COMPONENT, 639, "async setVersion(args: string | string[]) {"),
                Some(source_outcome(200, VERSION_ROUTER, 34, "@version_router_v1.post(\"/current\", summary=\"Sets t")),
            ),
        ]),
        availability: PRESENCE_UPDATE_BLUEOS_VERSION,
        blast_radius: BR_CORE_SWITCH,
        chains_from: None,
    };

const SWITCH_LOCAL_BLUEOS_VERSION: UseCase =
    UseCase {
        id: JourneyId::SwitchLocalBlueosVersion,
        summary: Grounded::known(
            "Switch forwards or backwards between locally installed BlueOS versions, including roll-back after undesired changes"
                ,
            Provenance::doc(ADV, 400, "- The full interface supports easily changing forwards _and "),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399, "{% pirate() %}")),
        services: VERSIONCHOOSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::SwitchBlueosVersion,
            "pirate-mode local version cards apply a previously installed image without re-downloading",
        )]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Software(SoftwareAssumption::PirateMode),
                Provenance::doc(ADV, 399, "{% pirate() %}"),
            ),
            GroundedItem::new(
                Precondition::Data(DataAssumption::LocalBlueosVersionAvailable),
                Provenance::doc(ADV, 402, "- Previously-installed versions are kept locally on the devi"),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open BlueOS Version with pirate mode enabled to view locally stored installs",
                None,
                Provenance::source(VC_COMPONENT, 26, "Turn on Pirate mode to view all available BlueOS versions, i"),
                None,
            ),
            operator_step(
                "Browse locally installed BlueOS core images kept on the device",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/version/available/local",
                    Some("v1.0"),
                    VERSION_ROUTER,
                    55,
                    "@version_router_v1.get(\"/available/local\", summary=\"Returns ",
                )),
                Provenance::doc(ADV, 402, "- Previously-installed versions are kept locally on the devi"),
                Some(runtime_outcome(
                    200,
                    None,
                    BodyKind::Payload,
                    "runtime-captures/versionchooser__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Apply the chosen local version",
                Some(sourced_route(HttpMethod::Post, "/version/current", Some("v1.0"), VERSION_ROUTER, 34, "@version_rou")),
                Provenance::source(VC_COMPONENT, 639, "async setVersion(args: string | string[]) {"),
                Some(source_outcome(200, VERSION_ROUTER, 34, "@version_router_v1.post(\"/current\", summary=\"Sets t")),
            ),
        ]),
        availability: PRESENCE_SWITCH_LOCAL_BLUEOS_VERSION,
        blast_radius: BR_CORE_SWITCH,
        chains_from: Some(JourneyId::UpdateBlueosVersion),
    };

const PULL_BLUEOS_VERSION_WITHOUT_SWITCH: UseCase =
    UseCase {
        id: JourneyId::PullBlueosVersionWithoutSwitch,
        summary: Grounded::known(
            "Download a remote BlueOS core image, including from a custom Docker registry repository, without switching the running version"
                ,
            Provenance::doc(ADV, 406, "- Allows loading remote versions (including from custom dock"),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399, "{% pirate() %}")),
        services: VERSIONCHOOSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::PullBlueosVersion,
            "remote Versions section can fetch an image tag to local storage before apply",
        )]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Network(NetworkState::Online),
                Provenance::doc(ADV, 406, "- Allows loading remote versions (including from custom dock"),
            ),
            GroundedItem::new(
                Precondition::Software(SoftwareAssumption::PirateMode),
                Provenance::doc(ADV, 399, "{% pirate() %}"),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Remote Versions section and optionally change the repository name",
                None,
                Provenance::source(VC_COMPONENT, 89, "<div class=\"d-flex justify-space-between pb-3\">"),
                None,
            ),
            operator_step(
                "Browse remote tags for the selected repository",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/version/available/{repository}/{image}",
                    Some("v1.0"),
                    VERSION_ROUTER,
                    60,
                    "@version_router_v1.get(",
                )),
                Provenance::doc(ADV, 406, "- Allows loading remote versions (including from custom dock"),
                None,
            ),
            operator_step(
                "Pull the selected remote tag to local storage",
                Some(sourced_route(HttpMethod::Post, "/version/pull", Some("v1.0"), VERSION_ROUTER, 41, "@version_rou")),
                Provenance::source(VC_COMPONENT, 553, "async pullVersion(image: string) {"),
                Some(source_outcome(200, VERSION_ROUTER, 41, "@version_router_v1.post(\"/pull\", summary=\"Pulls a v")),
            ),
        ]),
        availability: PRESENCE_PULL_BLUEOS_VERSION_WITHOUT_SWITCH,
        blast_radius: Grounded::known(
            BlastRadius::Reversible,
            Provenance::asserted(
                "POST /version/pull downloads a core image tag to local storage without applying it",
            ),
        ),
        chains_from: None,
    };

const DELETE_LOCAL_BLUEOS_VERSION: UseCase = UseCase {
    id: JourneyId::DeleteLocalBlueosVersion,
    summary: Grounded::known(
        "Delete a previously installed local BlueOS version to free onboard storage",
        Provenance::doc(
            ADV,
            402,
            "- Previously-installed versions are kept locally on the devi",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 399, "{% pirate() %}"),
    ),
    services: VERSIONCHOOSER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DeleteLocalBlueosVersion,
        "local version cards expose delete for non-current images when enough versions remain",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::PirateMode),
            Provenance::doc(ADV, 399, "{% pirate() %}"),
        ),
        GroundedItem::new(
            Precondition::Data(DataAssumption::LocalBlueosVersionAvailable),
            Provenance::source(
                VC_COMPONENT,
                72,
                ":enable-delete=\"local_versions.result.local.length > 2\"",
            ),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Local Versions section in pirate mode",
            None,
            Provenance::source(VC_COMPONENT, 61, "<h2>Local Versions</h2>"),
            None,
        ),
        operator_step(
            "List locally installed BlueOS core images",
            Some(sourced_route(
                HttpMethod::Get,
                "/version/available/local",
                Some("v1.0"),
                VERSION_ROUTER,
                55,
                "@version_router_v1.get(\"/available/local\", summary=\"Returns ",
            )),
            Provenance::doc(
                ADV,
                402,
                "- Previously-installed versions are kept locally on the devi",
            ),
            Some(runtime_outcome(
                200,
                None,
                BodyKind::Payload,
                "runtime-captures/versionchooser__pi4_navigator_master.json#running_baseline",
            )),
        ),
        operator_step(
            "Delete the selected non-current local version",
            Some(sourced_route(
                HttpMethod::Delete,
                "/version/delete",
                Some("v1.0"),
                VERSION_ROUTER,
                48,
                "@version_router_v1.delete(\"/delete\", summary=\"Delete the sel",
            )),
            Provenance::source(
                VC_COMPONENT,
                651,
                "async deleteVersion(args: string | string[]) {",
            ),
            Some(source_outcome(
                200,
                VERSION_ROUTER,
                48,
                "@version_router_v1.delete(\"/delete\", summary=\"Delete the sel",
            )),
        ),
    ]),
    availability: PRESENCE_DELETE_LOCAL_BLUEOS_VERSION,
    blast_radius: Grounded::known(
        BlastRadius::Reversible,
        Provenance::asserted(
            "DELETE /version/delete removes a cached non-current core image from onboard storage",
        ),
    ),
    chains_from: Some(JourneyId::SwitchLocalBlueosVersion),
};

const DOCKER_REGISTRY_LOGIN: UseCase =
    UseCase {
        id: JourneyId::DockerRegistryLogin,
        summary: Grounded::known(
            "Log in to Docker Hub or a custom registry to access private images and reduce rate limiting"
                ,
            Provenance::doc(ADV, 407, "- Allows logging in to one or more docker registries, to acc"),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399, "{% pirate() %}")),
        services: VERSIONCHOOSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::DockerRegistryLogin,
            "Docker Login dialog authenticates the daemon and lists connected accounts",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Software(SoftwareAssumption::PirateMode),
            Provenance::doc(ADV, 399, "{% pirate() %}"),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Docker Login dialog from the Remote Versions section",
                None,
                Provenance::source(VC_COMPONENT, 93, "@click=\"show_docker_login_dialog = true\""),
                None,
            ),
            operator_step(
                "Submit registry credentials, optionally for the root user or a custom registry index",
                Some(sourced_route(HttpMethod::Post, "/docker/login", Some("v1.0"), DOCKER_ROUTER, 20, "@docker_rout")),
                Provenance::doc(ADV, 407, "- Allows logging in to one or more docker registries, to acc"),
                Some(source_outcome(200, DOCKER_ROUTER, 20, "@docker_router_v1.post(\"/login\", summary=\"Login Dock")),
            ),
            operator_step(
                "Review connected Docker accounts",
                Some(sourced_route(HttpMethod::Get, "/docker/accounts/", Some("v1.0"), DOCKER_ROUTER, 30, "@docker_rout")),
                Provenance::source(DOCKER_LOGIN, 253, "async fetchAccounts() {"),
                Some(runtime_outcome(
                    200,
                    None,
                    BodyKind::Payload,
                    "runtime-captures/versionchooser__pi4_navigator_master.json#running_baseline",
                )),
            ),
        ]),
        availability: PRESENCE_DOCKER_REGISTRY_LOGIN,
        blast_radius: Grounded::known(
            BlastRadius::Reversible,
            Provenance::asserted(
                "POST /docker/login stores registry credentials in the daemon config and can be replaced",
            ),
        ),
        chains_from: None,
    };

const UPDATE_BOOTSTRAP_IMAGE: UseCase = UseCase {
    id: JourneyId::UpdateBootstrapImage,
    summary: Grounded::known(
        "Update the BlueOS-bootstrap image to match the currently running BlueOS core release",
        Provenance::doc(ADV, 405, "- Allows updating the [bootstrap image](@/development/bootst"),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399, "{% pirate() %}")),
    services: VERSIONCHOOSER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::UpdateBootstrapImage,
        "current-version card offers bootstrap update after core images are loaded",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(BOOTSTRAP, 73, "BlueOS-bootstrap versions are built at the same time as Blue"),
        ),
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::PirateMode),
            Provenance::doc(ADV, 399, "{% pirate() %}"),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Review the running bootstrap version shown on the current core card",
            Some(sourced_route(HttpMethod::Get, "/bootstrap/current", Some("v1.0"), BOOTSTRAP_ROUTER, 26, "@bootstrap_r")),
            Provenance::source(VC_UTILS, 161, "async function loadBootstrapCurrentVersion(): Promise<string"),
            Some(runtime_outcome(
                200,
                None,
                BodyKind::Payload,
                "runtime-captures/versionchooser__pi4_navigator_master.json#running_baseline",
            )),
        ),
        operator_step(
            "Download the bootstrap image tag that matches the running core version",
            Some(sourced_route(HttpMethod::Post, "/version/pull", Some("v1.0"), VERSION_ROUTER, 41, "@version_rout")),
            Provenance::source(VC_COMPONENT, 596, "async updateBootstrap(image: string) {"),
            Some(source_outcome(200, VERSION_ROUTER, 41, "@version_router_v1.post(\"/pull\", summary=\"Pulls a versi")),
        ),
        operator_step(
            "Set BlueOS-bootstrap to the downloaded tag",
            Some(sourced_route(HttpMethod::Post, "/bootstrap/current", Some("v1.0"), BOOTSTRAP_ROUTER, 31, "@bootstrap_r")),
            Provenance::source(VC_COMPONENT, 614, "async setBootstrapVersion(version: string) {"),
            Some(source_outcome(200, BOOTSTRAP_ROUTER, 31, "@bootstrap_router_v1.post(\"/current\", summary=\"Sets t")),
        ),
    ]),
    availability: PRESENCE_UPDATE_BOOTSTRAP_IMAGE,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "POST /bootstrap/current switches the bootstrap stack used for core recovery and updates",
        ),
    ),
    chains_from: Some(JourneyId::UpdateBlueosVersion),
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const VERSIONCHOOSER_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Versionchooser,
    Provenance::doc(
        ADV,
        389,
        "{{ service(service=\"Version Chooser\", port=8081, link=\"/serv",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Versionchooser,
        method,
        path,
        version,
    }
}

const fn sourced_route(
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    file: &'static str,
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
    file: &'static str,
    line: u32,
    anchor: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(file, line, anchor),
    )
}
