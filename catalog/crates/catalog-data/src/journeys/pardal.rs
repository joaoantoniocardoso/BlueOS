use crate::journey_presence::{PRESENCE_RUN_INTERNET_SPEED_TEST, PRESENCE_RUN_LAN_SPEED_TEST};
use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef,
    StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const PARDAL_MAIN: &str = "core/services/pardal/main.py";
const PARDAL_STORE: &str = "core/frontend/src/store/pardal.ts";
const PARDAL_MENUS: &str = "core/frontend/src/menus.ts";
const NETWORK_TEST_VIEW: &str = "core/frontend/src/views/NetworkTestView.vue";
const NETWORK_SPEED_TEST: &str = "core/frontend/src/components/speedtest/NetworkSpeedTest.vue";
const INTERNET_SPEED_TEST: &str = "core/frontend/src/components/speedtest/InternetSpeedTest.vue";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const JOURNEYS: &[UseCase] = &[RUN_LAN_SPEED_TEST, RUN_INTERNET_SPEED_TEST];

const RUN_LAN_SPEED_TEST: UseCase =
    UseCase {
        id: JourneyId::RunLanSpeedTest,
        summary: Grounded::known(
            "Measure real-time latency and upload/download speeds between BlueOS and the surface computer"
                ,
            Provenance::doc(ADV, 521, "The Local Network Test measures real-time latency between Bl"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 518, "### Network Test")),
        services: PARDAL_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::RunLanSpeedTest,
            "Local network test downloads and uploads a file while measuring latency over WebSocket echo",
        )]),
        preconditions: GroundedSet::known(&[]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Network Test page from the sidebar",
                None,
                Provenance::source(PARDAL_MENUS, 84, "title: 'NMEA Injector',"),
                None,
            ),
            operator_step(
                "Open the Local network test tab",
                None,
                Provenance::source(NETWORK_TEST_VIEW, 56, "{ title: 'Local network test', icon: 'mdi-speedometer', "),
                None,
            ),
            service_step(
                "Echo WebSocket messages back to the client for real-time latency measurement",
                None,
                Provenance::source(PARDAL_MAIN, 45, "async def websocket_echo(request: web.Request) -> web.WebSoc"),
                Some(pending_outcome(
                    "latency measurement requires runtime capture",
                )),
            ),
            operator_step(
                "Start the local network speed test",
                None,
                Provenance::source(NETWORK_SPEED_TEST, 165, "start(): void {"),
                None,
            ),
            operator_step(
                "Download a test file from the vehicle to measure download throughput",
                Some(sourced_route(HttpMethod::Get, "/get_file", None, 149, "app.router.add_get(\"/get_file\", get_f")),
                Provenance::source(NETWORK_SPEED_TEST, 241, "url: '/network-test/get_file',"),
                Some(runtime_outcome(
                    200,
                    Some(
                        "1 MiB random byte stream (Content-Length: 1048576); not JSON",
                    ),
                    BodyKind::Payload,
                    "runtime-captures/pardal__pi4_navigator_master.json#running_baseline",
                )),
            ),
            operator_step(
                "Upload a test file to the vehicle to measure upload throughput",
                Some(sourced_route(HttpMethod::Post, "/post_file", None, 150, "app.router.add_post(\"/post_file\", p")),
                Provenance::source(NETWORK_SPEED_TEST, 202, "url: '/network-test/post_file',"),
                Some(source_outcome(200, 70, "async def post_file(request: web.Request) -> web.Response:")),
            ),
            operator_step(
                "View the speed plot for the test run",
                None,
                Provenance::doc(ADV, 524, "A plot is provided of each test, to help diagnose intermitte"),
                None,
            ),
        ]),
        availability: PRESENCE_RUN_LAN_SPEED_TEST,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted(
                "LAN throughput test exchanges transient test payloads without persisting vehicle configuration",
            ),
        ),
        chains_from: None,
    };

const RUN_INTERNET_SPEED_TEST: UseCase = UseCase {
    id: JourneyId::RunInternetSpeedTest,
    summary: Grounded::known(
        "Measure latency and upload/download speeds between BlueOS and its internet connection",
        Provenance::doc(ADV, 528, "The Internet Speed Test allows measuring the latency and upl"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 518, "### Network Test")),
    services: PARDAL_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RunInternetSpeedTest,
        "Internet speed test selects a speedtest-cli server then measures WAN download and upload",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Network(NetworkState::Online),
        Provenance::doc(ADV, 529, "speeds between BlueOS and its internet connection (if one is"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Network Test page from the sidebar",
            None,
            Provenance::source(PARDAL_MENUS, 84, "title: 'NMEA Injector',"),
            None,
        ),
        operator_step(
            "Open the Internet speed test tab",
            None,
            Provenance::source(NETWORK_TEST_VIEW, 57, "{ title: 'Internet speed test', icon: 'mdi-web', value: 'int"),
            None,
        ),
        operator_step(
            "Load any previous internet speed test result shown on page open",
            Some(sourced_route(HttpMethod::Get, "/internet_test_previous_result", None, 119, "@routes.get(\"/intern")),
            Provenance::source(INTERNET_SPEED_TEST, 180, "this.result = await pardal.checkPreviousInternetTestResul"),
            Some(runtime_outcome(
                200,
                Some("\"download\":"),
                BodyKind::Payload,
                "runtime-captures/pardal__pi4_navigator_master.json#running_baseline",
            )),
        ),
        operator_step(
            "Start the internet speed test",
            None,
            Provenance::source(INTERNET_SPEED_TEST, 194, "async start(): Promise<void> {"),
            None,
        ),
        operator_step(
            "Find the best speedtest server for the vehicle's internet connection",
            Some(sourced_route(HttpMethod::Get, "/internet_best_server", None, 78, "@routes.get(\"/internet_best_se")),
            Provenance::source(PARDAL_STORE, 29, "url: `${this.API_URL}/internet_best_server`,"),
            Some(source_outcome(200, 78, "@routes.get(\"/internet_best_server\")")),
        ),
        operator_step(
            "Measure internet download speed",
            Some(sourced_route(HttpMethod::Get, "/internet_download_speed", None, 95, "@routes.get(\"/internet_down")),
            Provenance::source(PARDAL_STORE, 42, "url: `${this.API_URL}/internet_download_speed`,"),
            Some(source_outcome(200, 95, "@routes.get(\"/internet_download_speed\")")),
        ),
        operator_step(
            "Measure internet upload speed",
            Some(sourced_route(HttpMethod::Get, "/internet_upload_speed", None, 107, "@routes.get(\"/internet_uploa")),
            Provenance::source(PARDAL_STORE, 55, "url: `${this.API_URL}/internet_upload_speed`,"),
            Some(source_outcome(200, 107, "@routes.get(\"/internet_upload_speed\")")),
        ),
    ]),
    availability: PRESENCE_RUN_INTERNET_SPEED_TEST,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "Internet speed test measures WAN throughput via speedtest-cli without altering vehicle settings",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const PARDAL_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Pardal,
    Provenance::doc(
        ADV,
        519,
        "{{ service(service=\"Pardal\", port=9120, link=\"/services/pard",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Pardal,
        method,
        path,
        version,
    }
}

const fn sourced_route(
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    line: u32,
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(PARDAL_MAIN, line, anchor),
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
            actor: Actor::Service(ServiceId::Pardal),
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn pending_outcome(reason: &'static str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
}

const fn source_outcome(status: u16, line: u32, anchor: &'static str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(PARDAL_MAIN, line, anchor),
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
