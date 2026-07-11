use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef, StepOutcome, UserJourney,
    Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const PARDAL_MAIN: &str = "core/services/pardal/main.py";
const PARDAL_STORE: &str = "core/frontend/src/store/pardal.ts";
const PARDAL_MENUS: &str = "core/frontend/src/menus.ts";
const NETWORK_TEST_VIEW: &str = "core/frontend/src/views/NetworkTestView.vue";
const NETWORK_SPEED_TEST: &str = "core/frontend/src/components/speedtest/NetworkSpeedTest.vue";
const INTERNET_SPEED_TEST: &str = "core/frontend/src/components/speedtest/InternetSpeedTest.vue";
const RUNTIME_CAPTURE: &str = "runtime-captures/pardal__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn journeys() -> Vec<UserJourney> {
    vec![run_lan_speed_test(), run_internet_speed_test()]
}

fn run_lan_speed_test() -> UserJourney {
    UserJourney {
        id: JourneyId("run_lan_speed_test".into()),
        summary: Grounded::known(
            "Measure real-time latency and upload/download speeds between BlueOS and the surface computer"
                .into(),
            Provenance::doc(ADV, 521),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 518)),
        services: pardal_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "run_lan_speed_test",
            "Local network test downloads and uploads a file while measuring latency over WebSocket echo",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Network Test page from the sidebar",
                None,
                Provenance::source(PARDAL_MENUS, 84),
                None,
            ),
            operator_step(
                "Open the Local network test tab",
                None,
                Provenance::source(NETWORK_TEST_VIEW, 56),
                None,
            ),
            service_step(
                "Echo WebSocket messages back to the client for real-time latency measurement",
                None,
                Provenance::source(PARDAL_MAIN, 45),
                Some(pending_outcome(
                    "latency measurement requires runtime capture",
                )),
            ),
            operator_step(
                "Start the local network speed test",
                None,
                Provenance::source(NETWORK_SPEED_TEST, 165),
                None,
            ),
            operator_step(
                "Download a test file from the vehicle to measure download throughput",
                Some(sourced_route(HttpMethod::Get, "/get_file", None, 149)),
                Provenance::source(NETWORK_SPEED_TEST, 241),
                Some(runtime_outcome(
                    200,
                    Some(
                        "1 MiB random byte stream (Content-Length: 1048576); not JSON".into(),
                    ),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Upload a test file to the vehicle to measure upload throughput",
                Some(sourced_route(HttpMethod::Post, "/post_file", None, 150)),
                Provenance::source(NETWORK_SPEED_TEST, 202),
                Some(pending_outcome(
                    "upload throughput requires runtime capture",
                )),
            ),
            operator_step(
                "View the speed plot for the test run",
                None,
                Provenance::doc(ADV, 524),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn run_internet_speed_test() -> UserJourney {
    UserJourney {
        id: JourneyId("run_internet_speed_test".into()),
        summary: Grounded::known(
            "Measure latency and upload/download speeds between BlueOS and its internet connection"
                .into(),
            Provenance::doc(ADV, 528),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 518)),
        services: pardal_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "run_internet_speed_test",
            "Internet speed test selects a speedtest-cli server then measures WAN download and upload",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(ADV, 529),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Network Test page from the sidebar",
                None,
                Provenance::source(PARDAL_MENUS, 84),
                None,
            ),
            operator_step(
                "Open the Internet speed test tab",
                None,
                Provenance::source(NETWORK_TEST_VIEW, 57),
                None,
            ),
            operator_step(
                "Load any previous internet speed test result shown on page open",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/internet_test_previous_result",
                    None,
                    119,
                )),
                Provenance::source(INTERNET_SPEED_TEST, 180),
                Some(runtime_outcome(
                    200,
                    Some("\"download\":".into()),
                    "#running_baseline",
                )),
            ),
            operator_step(
                "Start the internet speed test",
                None,
                Provenance::source(INTERNET_SPEED_TEST, 194),
                None,
            ),
            operator_step(
                "Find the best speedtest server for the vehicle's internet connection",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/internet_best_server",
                    None,
                    78,
                )),
                Provenance::source(PARDAL_STORE, 29),
                Some(pending_outcome(
                    "speedtest server selection requires runtime capture",
                )),
            ),
            operator_step(
                "Measure internet download speed",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/internet_download_speed",
                    None,
                    95,
                )),
                Provenance::source(PARDAL_STORE, 42),
                Some(pending_outcome(
                    "internet download speed requires runtime capture",
                )),
            ),
            operator_step(
                "Measure internet upload speed",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/internet_upload_speed",
                    None,
                    107,
                )),
                Provenance::source(PARDAL_STORE, 55),
                Some(pending_outcome(
                    "internet upload speed requires runtime capture",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn pardal_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Pardal,
        Provenance::doc(ADV, 519),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Pardal,
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
    Grounded::known(
        route(method, path, version),
        Provenance::source(PARDAL_MAIN, line),
    )
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

fn service_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId::Pardal),
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}

fn pending_outcome(reason: &str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
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
