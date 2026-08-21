use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::NetworkTest,
        route: Observed::known(
            "/tools/network-test",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 97,
                anchor: "path: '/tools/mavlink-inspector',",
            },
        ),
        name: Observed::known(
            "Network Test",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 98,
                anchor: "name: 'Mavlink Inspector',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/NetworkTestView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 99,
                anchor: "component: defineAsyncComponent(() => import('../views/Mavli",
            },
        ),
        menu_title: Observed::known(
            "Network Test",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 84,
                anchor: "title: 'NMEA Injector',",
            },
        ),
        advanced_only: Observed::unknown("menu entry has show:true but omits advanced field"),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "pardal",
                Evidence {
                    file: "core/frontend/src/components/speedtest/InternetSpeedTest.vue",
                    line: 130,
                    anchor: "import pardal from '@/store/pardal'",
                },
            ),
            Evidenced::new(
                "helper",
                Evidence {
                    file: "core/frontend/src/components/speedtest/InternetSpeedTest.vue",
                    line: 129,
                    anchor: "import helper from '@/store/helper'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Pardal),
                    endpoint: "WS /network-test/ws",
                    purpose: "echo latency probe every 200ms on Local network test tab mount",
                },
                Evidence {
                    file: "core/frontend/src/components/speedtest/NetworkSpeedTest.vue",
                    line: 147,
                    anchor: "this.websocket = new WebSocket(`${protocol}://${window.locat",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Pardal),
                    endpoint: "GET /network-test/get_file",
                    purpose: "download 100MB buffer to measure LAN download speed after Start",
                },
                Evidence {
                    file: "core/frontend/src/components/speedtest/NetworkSpeedTest.vue",
                    line: 241,
                    anchor: "url: '/network-test/get_file',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Pardal),
                    endpoint: "POST /network-test/post_file",
                    purpose: "upload 100MB buffer to measure LAN upload speed after download completes",
                },
                Evidence {
                    file: "core/frontend/src/components/speedtest/NetworkSpeedTest.vue",
                    line: 202,
                    anchor: "url: '/network-test/post_file',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Pardal),
                    endpoint: "GET /network-test/internet_test_previous_result",
                    purpose: "load last internet speedtest result on Internet speed test tab mount",
                },
                Evidence {
                    file: "core/frontend/src/store/pardal.ts",
                    line: 68,
                    anchor: "url: `${this.API_URL}/internet_test_previous_result`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Pardal),
                    endpoint: "GET /network-test/internet_best_server",
                    purpose: "select speedtest-cli server at start of internet test sequence",
                },
                Evidence {
                    file: "core/frontend/src/store/pardal.ts",
                    line: 29,
                    anchor: "url: `${this.API_URL}/internet_best_server`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Pardal),
                    endpoint: "GET /network-test/internet_download_speed",
                    purpose: "measure WAN download speed during internet test sequence",
                },
                Evidence {
                    file: "core/frontend/src/store/pardal.ts",
                    line: 42,
                    anchor: "url: `${this.API_URL}/internet_download_speed`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Pardal),
                    endpoint: "GET /network-test/internet_upload_speed",
                    purpose: "measure WAN upload speed during internet test sequence",
                },
                Evidence {
                    file: "core/frontend/src/store/pardal.ts",
                    line: 55,
                    anchor: "url: `${this.API_URL}/internet_upload_speed`,",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "active tab",
                    store: "NetworkTestView.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "page_selected chooses Local network test vs Internet speed test",
                },
                "tab selection is ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "LAN test state machine",
                    store: "NetworkSpeedTest.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "State enum (None/DownloadSpeed/UploadSpeed/Done), speed gauge, websocket handle",
                },
                "local network test orchestrates download-then-upload sequence client-side",
            ),
            Rationaled::new(
                ClientState {
                    name: "LAN speed samples",
                    store: "NetworkSpeedTest.vue series/download_speed/upload_speed/latency_ms",
                    ownership: StateOwnership::Shared,
                    notes: "Mbps computed from axios progress with 0.7 EMA; graph points {x:%, y:Mbps}",
                },
                "headline and graph speeds are client-derived from pardal transfer progress events",
            ),
            Rationaled::new(
                ClientState {
                    name: "LAN upload buffer",
                    store: "NetworkSpeedTest.vue upload_buffer",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "100MB ArrayBuffer allocated once for POST body",
                },
                "upload payload is preallocated in the browser to avoid per-test allocation",
            ),
            Rationaled::new(
                ClientState {
                    name: "internet test progress",
                    store: "InternetSpeedTest.vue started/message/result",
                    ownership: StateOwnership::Shared,
                    notes: "started flag, status text, SpeedTestResult from sequential pardal GETs",
                },
                "internet tab tracks multi-step test status while mirroring pardal speedtest-cli results",
            ),
            Rationaled::new(
                ClientState {
                    name: "internet connectivity gate",
                    store: "InternetSpeedTest.vue computed internet_offline",
                    ownership: StateOwnership::BackendOwned,
                    notes: "helper.has_internet === OFFLINE disables overlay; helper polls globally",
                },
                "page reads helper internet state from global background polling, not page-triggered",
            ),
        ]),
    };
