use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, DataRequirement, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef,
    SoftwareRequirement, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

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

pub const JOURNEYS: &[UserJourney] = &[
    UPDATE_BLUEOS_VERSION,
    SWITCH_LOCAL_BLUEOS_VERSION,
    PULL_BLUEOS_VERSION_WITHOUT_SWITCH,
    DELETE_LOCAL_BLUEOS_VERSION,
    DOCKER_REGISTRY_LOGIN,
    UPDATE_BOOTSTRAP_IMAGE,
];

const UPDATE_BLUEOS_VERSION: UserJourney =
    UserJourney {
        id: JourneyId::UpdateBlueosVersion,
        summary: Grounded::known(
            "Update BlueOS to the latest available release that is as stable or more stable than the current install"
                ,
            Provenance::doc(ADV, 397),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 397)),
        services: VERSIONCHOOSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::UpdateBlueosVersion,
            "simplified Version Chooser pulls and applies a newer stable or beta release",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(GETTING, 104),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open Settings and select BlueOS Version",
                None,
                Provenance::doc(GETTING, 109),
                None,
            ),
            operator_step(
                "Review the current running version and whether an update button is shown",
                Some(sourced_route(HttpMethod::Get, "/version/current", Some("v1.0"), VERSION_ROUTER, 26)),
                Provenance::doc(GETTING, 112),
                Some(runtime_outcome(
                    200,
                    None,
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
                )),
                Provenance::doc(ADV, 397),
                None,
            ),
            operator_step(
                "Click the update button to download the newer BlueOS core image",
                Some(sourced_route(HttpMethod::Post, "/version/pull", Some("v1.0"), VERSION_ROUTER, 41)),
                Provenance::doc(GETTING, 116),
                Some(source_outcome(200, VERSION_ROUTER, 41)),
            ),
            operator_step(
                "Switch BlueOS core to the downloaded version and restart",
                Some(sourced_route(HttpMethod::Post, "/version/current", Some("v1.0"), VERSION_ROUTER, 34)),
                Provenance::source(VC_COMPONENT, 639),
                Some(source_outcome(200, VERSION_ROUTER, 34)),
            ),
        ]),
        chains_from: None,
    };

const SWITCH_LOCAL_BLUEOS_VERSION: UserJourney =
    UserJourney {
        id: JourneyId::SwitchLocalBlueosVersion,
        summary: Grounded::known(
            "Switch forwards or backwards between locally installed BlueOS versions, including roll-back after undesired changes"
                ,
            Provenance::doc(ADV, 400),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
        services: VERSIONCHOOSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::SwitchBlueosVersion,
            "pirate-mode local version cards apply a previously installed image without re-downloading",
        )]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Software(SoftwareRequirement::PirateMode),
                Provenance::doc(ADV, 399),
            ),
            GroundedItem::new(
                Precondition::Data(DataRequirement::LocalBlueosVersionAvailable),
                Provenance::doc(ADV, 402),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open BlueOS Version with pirate mode enabled to view locally stored installs",
                None,
                Provenance::source(VC_COMPONENT, 26),
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
                )),
                Provenance::doc(ADV, 402),
                Some(runtime_outcome(
                    200,
                    None,
                    "runtime-captures/versionchooser__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Apply the chosen local version",
                Some(sourced_route(HttpMethod::Post, "/version/current", Some("v1.0"), VERSION_ROUTER, 34)),
                Provenance::source(VC_COMPONENT, 639),
                Some(source_outcome(200, VERSION_ROUTER, 34)),
            ),
        ]),
        chains_from: Some(JourneyId::UpdateBlueosVersion),
    };

const PULL_BLUEOS_VERSION_WITHOUT_SWITCH: UserJourney =
    UserJourney {
        id: JourneyId::PullBlueosVersionWithoutSwitch,
        summary: Grounded::known(
            "Download a remote BlueOS core image, including from a custom Docker registry repository, without switching the running version"
                ,
            Provenance::doc(ADV, 406),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
        services: VERSIONCHOOSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::PullBlueosVersion,
            "remote Versions section can fetch an image tag to local storage before apply",
        )]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Network(NetworkState::Online),
                Provenance::doc(ADV, 406),
            ),
            GroundedItem::new(
                Precondition::Software(SoftwareRequirement::PirateMode),
                Provenance::doc(ADV, 399),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Remote Versions section and optionally change the repository name",
                None,
                Provenance::source(VC_COMPONENT, 89),
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
                )),
                Provenance::doc(ADV, 406),
                None,
            ),
            operator_step(
                "Pull the selected remote tag to local storage",
                Some(sourced_route(HttpMethod::Post, "/version/pull", Some("v1.0"), VERSION_ROUTER, 41)),
                Provenance::source(VC_COMPONENT, 553),
                Some(source_outcome(200, VERSION_ROUTER, 41)),
            ),
        ]),
        chains_from: None,
    };

const DELETE_LOCAL_BLUEOS_VERSION: UserJourney = UserJourney {
    id: JourneyId::DeleteLocalBlueosVersion,
    summary: Grounded::known(
        "Delete a previously installed local BlueOS version to free onboard storage",
        Provenance::doc(ADV, 402),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
    services: VERSIONCHOOSER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DeleteLocalBlueosVersion,
        "local version cards expose delete for non-current images when enough versions remain",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareRequirement::PirateMode),
            Provenance::doc(ADV, 399),
        ),
        GroundedItem::new(
            Precondition::Data(DataRequirement::LocalBlueosVersionAvailable),
            Provenance::source(VC_COMPONENT, 72),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Local Versions section in pirate mode",
            None,
            Provenance::source(VC_COMPONENT, 61),
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
            )),
            Provenance::doc(ADV, 402),
            Some(runtime_outcome(
                200,
                None,
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
            )),
            Provenance::source(VC_COMPONENT, 651),
            Some(source_outcome(200, VERSION_ROUTER, 48)),
        ),
    ]),
    chains_from: Some(JourneyId::SwitchLocalBlueosVersion),
};

const DOCKER_REGISTRY_LOGIN: UserJourney =
    UserJourney {
        id: JourneyId::DockerRegistryLogin,
        summary: Grounded::known(
            "Log in to Docker Hub or a custom registry to access private images and reduce rate limiting"
                ,
            Provenance::doc(ADV, 407),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
        services: VERSIONCHOOSER_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::DockerRegistryLogin,
            "Docker Login dialog authenticates the daemon and lists connected accounts",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Software(SoftwareRequirement::PirateMode),
            Provenance::doc(ADV, 399),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Docker Login dialog from the Remote Versions section",
                None,
                Provenance::source(VC_COMPONENT, 93),
                None,
            ),
            operator_step(
                "Submit registry credentials, optionally for the root user or a custom registry index",
                Some(sourced_route(HttpMethod::Post, "/docker/login", Some("v1.0"), DOCKER_ROUTER, 20)),
                Provenance::doc(ADV, 407),
                Some(source_outcome(200, DOCKER_ROUTER, 20)),
            ),
            operator_step(
                "Review connected Docker accounts",
                Some(sourced_route(HttpMethod::Get, "/docker/accounts", Some("v1.0"), DOCKER_ROUTER, 30)),
                Provenance::source(DOCKER_LOGIN, 253),
                Some(runtime_outcome(
                    200,
                    None,
                    "runtime-captures/versionchooser__pi4_navigator_master.json#running_baseline",
                )),
            ),
        ]),
        chains_from: None,
    };

const UPDATE_BOOTSTRAP_IMAGE: UserJourney = UserJourney {
    id: JourneyId::UpdateBootstrapImage,
    summary: Grounded::known(
        "Update the BlueOS-bootstrap image to match the currently running BlueOS core release",
        Provenance::doc(ADV, 405),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
    services: VERSIONCHOOSER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::UpdateBootstrapImage,
        "current-version card offers bootstrap update after core images are loaded",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(BOOTSTRAP, 73),
        ),
        GroundedItem::new(
            Precondition::Software(SoftwareRequirement::PirateMode),
            Provenance::doc(ADV, 399),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Review the running bootstrap version shown on the current core card",
            Some(sourced_route(
                HttpMethod::Get,
                "/bootstrap/current",
                Some("v1.0"),
                BOOTSTRAP_ROUTER,
                26,
            )),
            Provenance::source(VC_UTILS, 161),
            Some(runtime_outcome(
                200,
                None,
                "runtime-captures/versionchooser__pi4_navigator_master.json#running_baseline",
            )),
        ),
        operator_step(
            "Download the bootstrap image tag that matches the running core version",
            Some(sourced_route(
                HttpMethod::Post,
                "/version/pull",
                Some("v1.0"),
                VERSION_ROUTER,
                41,
            )),
            Provenance::source(VC_COMPONENT, 596),
            Some(source_outcome(200, VERSION_ROUTER, 41)),
        ),
        operator_step(
            "Set BlueOS-bootstrap to the downloaded tag",
            Some(sourced_route(
                HttpMethod::Post,
                "/bootstrap/current",
                Some("v1.0"),
                BOOTSTRAP_ROUTER,
                31,
            )),
            Provenance::source(VC_COMPONENT, 614),
            Some(source_outcome(200, BOOTSTRAP_ROUTER, 31)),
        ),
    ]),
    chains_from: Some(JourneyId::UpdateBlueosVersion),
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const VERSIONCHOOSER_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Versionchooser,
    Provenance::doc(ADV, 389),
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
) -> Grounded<RouteRef> {
    Grounded::known(route(method, path, version), Provenance::source(file, line))
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
    key: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}

const fn source_outcome(status: u16, file: &'static str, line: u32) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            transition: None,
        },
        Provenance::source(file, line),
    )
}
