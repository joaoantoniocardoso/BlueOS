use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef, StepOutcome, UserJourney,
    Visibility,
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

pub fn journeys() -> Vec<UserJourney> {
    vec![
        update_blueos_version(),
        switch_local_blueos_version(),
        pull_blueos_version_without_switch(),
        delete_local_blueos_version(),
        docker_registry_login(),
        update_bootstrap_image(),
    ]
}

fn update_blueos_version() -> UserJourney {
    UserJourney {
        id: JourneyId("update_blueos_version".into()),
        summary: Grounded::known(
            "Update BlueOS to the latest available release that is as stable or more stable than the current install"
                .into(),
            Provenance::doc(ADV, 397),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 397)),
        services: versionchooser_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "update_blueos_version",
            "simplified Version Chooser pulls and applies a newer stable or beta release",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(GETTING, 104),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open Settings and select BlueOS Version",
                None,
                Provenance::doc(GETTING, 109),
                None,
            ),
            operator_step(
                "Review the current running version and whether an update button is shown",
                Some(sourced_route(HttpMethod::Get, "/version/current", Some("v1.0"), 26)),
                Provenance::doc(GETTING, 112),
                None,
            ),
            operator_step(
                "Fetch remote stable, beta, and master tags to determine the offered upgrade",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/version/available/{repository}/{image}",
                    Some("v1.0"),
                    60,
                )),
                Provenance::doc(ADV, 397),
                None,
            ),
            operator_step(
                "Click the update button to download the newer BlueOS core image",
                Some(sourced_route(HttpMethod::Post, "/version/pull", Some("v1.0"), 41)),
                Provenance::doc(GETTING, 116),
                None,
            ),
            operator_step(
                "Switch BlueOS core to the downloaded version and restart",
                Some(sourced_route(HttpMethod::Post, "/version/current", Some("v1.0"), 34)),
                Provenance::source(VC_COMPONENT, 639),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn switch_local_blueos_version() -> UserJourney {
    UserJourney {
        id: JourneyId("switch_local_blueos_version".into()),
        summary: Grounded::known(
            "Switch forwards or backwards between locally installed BlueOS versions, including roll-back after undesired changes"
                .into(),
            Provenance::doc(ADV, 400),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
        services: versionchooser_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "switch_blueos_version",
            "pirate-mode local version cards apply a previously installed image without re-downloading",
        )]),
        preconditions: GroundedSet::known(vec![
            GroundedItem::new(
                Precondition::Other("Pirate mode enabled".into()),
                Provenance::doc(ADV, 399),
            ),
            GroundedItem::new(
                Precondition::Other("At least one non-current BlueOS version is installed locally".into()),
                Provenance::doc(ADV, 402),
            ),
        ]),
        steps: GroundedSet::known(vec![
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
                    55,
                )),
                Provenance::doc(ADV, 402),
                None,
            ),
            operator_step(
                "Apply the chosen local version",
                Some(sourced_route(HttpMethod::Post, "/version/current", Some("v1.0"), 34)),
                Provenance::source(VC_COMPONENT, 639),
                None,
            ),
        ]),
        chains_from: Some(JourneyId("update_blueos_version".into())),
    }
}

fn pull_blueos_version_without_switch() -> UserJourney {
    UserJourney {
        id: JourneyId("pull_blueos_version_without_switch".into()),
        summary: Grounded::known(
            "Download a remote BlueOS core image, including from a custom Docker registry repository, without switching the running version"
                .into(),
            Provenance::doc(ADV, 406),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
        services: versionchooser_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "pull_blueos_version",
            "remote Versions section can fetch an image tag to local storage before apply",
        )]),
        preconditions: GroundedSet::known(vec![
            GroundedItem::new(
                Precondition::Network(NetworkState::Online),
                Provenance::doc(ADV, 406),
            ),
            GroundedItem::new(
                Precondition::Other("Pirate mode enabled".into()),
                Provenance::doc(ADV, 399),
            ),
        ]),
        steps: GroundedSet::known(vec![
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
                    60,
                )),
                Provenance::doc(ADV, 406),
                None,
            ),
            operator_step(
                "Pull the selected remote tag to local storage",
                Some(sourced_route(HttpMethod::Post, "/version/pull", Some("v1.0"), 41)),
                Provenance::source(VC_COMPONENT, 553),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn delete_local_blueos_version() -> UserJourney {
    UserJourney {
        id: JourneyId("delete_local_blueos_version".into()),
        summary: Grounded::known(
            "Delete a previously installed local BlueOS version to free onboard storage".into(),
            Provenance::doc(ADV, 402),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
        services: versionchooser_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "delete_local_blueos_version",
            "local version cards expose delete for non-current images when enough versions remain",
        )]),
        preconditions: GroundedSet::known(vec![
            GroundedItem::new(
                Precondition::Other("Pirate mode enabled".into()),
                Provenance::doc(ADV, 399),
            ),
            GroundedItem::new(
                Precondition::Other("More than two local BlueOS versions are installed".into()),
                Provenance::source(VC_COMPONENT, 72),
            ),
        ]),
        steps: GroundedSet::known(vec![
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
                    55,
                )),
                Provenance::doc(ADV, 402),
                None,
            ),
            operator_step(
                "Delete the selected non-current local version",
                Some(sourced_route(
                    HttpMethod::Delete,
                    "/version/delete",
                    Some("v1.0"),
                    48,
                )),
                Provenance::source(VC_COMPONENT, 651),
                None,
            ),
        ]),
        chains_from: Some(JourneyId("switch_local_blueos_version".into())),
    }
}

fn docker_registry_login() -> UserJourney {
    UserJourney {
        id: JourneyId("docker_registry_login".into()),
        summary: Grounded::known(
            "Log in to Docker Hub or a custom registry to access private images and reduce rate limiting"
                .into(),
            Provenance::doc(ADV, 407),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
        services: versionchooser_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "docker_registry_login",
            "Docker Login dialog authenticates the daemon and lists connected accounts",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Pirate mode enabled".into()),
            Provenance::doc(ADV, 399),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Docker Login dialog from the Remote Versions section",
                None,
                Provenance::source(VC_COMPONENT, 93),
                None,
            ),
            operator_step(
                "Submit registry credentials, optionally for the root user or a custom registry index",
                Some(sourced_route(HttpMethod::Post, "/docker/login", Some("v1.0"), 20)),
                Provenance::doc(ADV, 407),
                None,
            ),
            operator_step(
                "Review connected Docker accounts",
                Some(sourced_route(HttpMethod::Get, "/docker/accounts", Some("v1.0"), 30)),
                Provenance::source(DOCKER_LOGIN, 253),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn update_bootstrap_image() -> UserJourney {
    UserJourney {
        id: JourneyId("update_bootstrap_image".into()),
        summary: Grounded::known(
            "Update the BlueOS-bootstrap image to match the currently running BlueOS core release"
                .into(),
            Provenance::doc(ADV, 405),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 399)),
        services: versionchooser_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "update_bootstrap_image",
            "current-version card offers bootstrap update after core images are loaded",
        )]),
        preconditions: GroundedSet::known(vec![
            GroundedItem::new(
                Precondition::Network(NetworkState::Online),
                Provenance::doc(BOOTSTRAP, 73),
            ),
            GroundedItem::new(
                Precondition::Other("Pirate mode enabled".into()),
                Provenance::doc(ADV, 399),
            ),
        ]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Review the running bootstrap version shown on the current core card",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/bootstrap/current",
                    Some("v1.0"),
                    26,
                )),
                Provenance::source(VC_UTILS, 161),
                None,
            ),
            operator_step(
                "Download the bootstrap image tag that matches the running core version",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/version/pull",
                    Some("v1.0"),
                    41,
                )),
                Provenance::source(VC_COMPONENT, 596),
                None,
            ),
            operator_step(
                "Set BlueOS-bootstrap to the downloaded tag",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/bootstrap/current",
                    Some("v1.0"),
                    31,
                )),
                Provenance::source(VC_COMPONENT, 614),
                None,
            ),
        ]),
        chains_from: Some(JourneyId("update_blueos_version".into())),
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn versionchooser_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("versionchooser".into()),
        Provenance::doc(ADV, 389),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId("versionchooser".into()),
        method,
        path: path.into(),
        version: version.map(str::to_string),
    }
}

fn sourced_route(
    method: HttpMethod,
    path: &str,
    version: Option<&str>,
    line: u32,
) -> Grounded<RouteRef> {
    let file = match path {
        p if p.starts_with("/bootstrap") => BOOTSTRAP_ROUTER,
        p if p.starts_with("/docker") => DOCKER_ROUTER,
        _ => VERSION_ROUTER,
    };
    Grounded::known(route(method, path, version), Provenance::source(file, line))
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
