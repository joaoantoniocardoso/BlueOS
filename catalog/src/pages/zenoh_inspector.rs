use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::ZenohInspector,
        route: Observed::known(
            "/tools/zenoh-inspector",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 137,
            },
        ),
        name: Observed::known(
            "Zenoh Inspector",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 138,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/ZenohInspectorView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 139,
            },
        ),
        menu_title: Observed::known(
            "Zenoh Inspector",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 142,
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 145,
            },
        ),
        stores: ObservedSet::known(&[]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Zenohd),
                    endpoint: "WS /zenoh-api/",
                    purpose: "open zenoh-ts Session for Topics tab subscriber and Network tab topology queries",
                },
                Evidence {
                    file: "core/frontend/src/libs/zenoh/index.ts",
                    line: 35,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "active tab",
                    store: "ZenohInspectorView.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "page_selected defaults to topics; switches Topics vs Network child",
                },
                "tab selection is ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "topic inspector state",
                    store: "ZenohInspector.vue component data",
                    ownership: StateOwnership::Shared,
                    notes: "topics[], messages{}, topic_liveliness/types/message_types, selected_topic, topic_filter",
                },
                "topics tab accumulates live zenoh samples and liveliness tokens client-side over the shared session",
            ),
            Rationaled::new(
                ClientState {
                    name: "topic message formatting",
                    store: "ZenohInspector.vue method formatMessage",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "JSON pretty-print by encoding (TEXT/JSON/ZENOH_BYTES); video topics decoded via rosmsg reader",
                },
                "payload presentation and CompressedVideo deserialization are client-side",
            ),
            Rationaled::new(
                ClientState {
                    name: "video topic decoder",
                    store: "ZenohInspector.vue video_reader",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "MessageReader built from /msgs/CompressedVideo.msg static definition",
                },
                "ROS message schema is fetched once and held for RawVideoPlayer decode",
            ),
            Rationaled::new(
                ClientState {
                    name: "zenoh network graph",
                    store: "ZenohNetwork.vue networkData/cy",
                    ownership: StateOwnership::Shared,
                    notes: "nodes/edges from zenoh adminspace queries; cytoscape Core frozen to avoid Vue2 reactivity overhead",
                },
                "network topology is queried from zenohd then laid out and styled in the browser",
            ),
            Rationaled::new(
                ClientState {
                    name: "zenoh session singleton",
                    store: "libs/zenoh ZenohManager.sessionPromise",
                    ownership: StateOwnership::Shared,
                    notes: "shared Session reused across inspector tabs and other features app-wide",
                },
                "session is opened on first getSession() and cached module-wide, not page-private",
            ),
        ]),
    };
