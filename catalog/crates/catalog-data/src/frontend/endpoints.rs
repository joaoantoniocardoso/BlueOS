use catalog_kernel::id::page::PageId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{
    AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled,
};
use catalog_model::page::{ClientState, ConsumeTarget, Page, PageServiceCall, StateOwnership};

pub const PAGE: Page =
    Page {
        id: PageId::Endpoints,
        route: Observed::known(
            "/vehicle/endpoints",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 37,
                anchor: "path: '/vehicle/endpoints',",
            },
        ),
        name: Observed::known(
            "Endpoints",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 38,
                anchor: "name: 'Endpoints',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/EndpointView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 39,
                anchor: "component: defineAsyncComponent(() => import('../views/Endpo",
            },
        ),
        menu_title: Observed::known(
            "MAVLink Endpoints",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 69,
                anchor: "title: 'MAVLink Endpoints',",
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 72,
                anchor: "advanced: true,",
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "autopilot_data",
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointManager.vue",
                    line: 129,
                    anchor: "import autopilot_data from '@/store/autopilot'",
                },
            ),
            Evidenced::new(
                "autopilot",
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointManager.vue",
                    line: 130,
                    anchor: "import autopilot from '@/store/autopilot_manager'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/endpoints",
                    purpose: "poll configured MAVLink endpoints for card list",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 32,
                    anchor: "const response = await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/board",
                    purpose: "poll current board to gate endpoint creation on autopilot running",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 59,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/available_routers",
                    purpose: "list MAVLink router implementations when more than one is available",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointManager.vue",
                    line: 202,
                    anchor: "back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/preferred_router",
                    purpose: "read active MAVLink router selection",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointManager.vue",
                    line: 215,
                    anchor: "back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/preferred_router",
                    purpose: "switch MAVLink router (mavlink-router vs MAVP2P)",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointManager.vue",
                    line: 229,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/endpoints",
                    purpose: "create new MAVLink endpoint from creation dialog",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointManager.vue",
                    line: 246,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "PUT /ardupilot-manager/v1.0/endpoints",
                    purpose: "update endpoint configuration or enabled toggle from EndpointCard",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointCard.vue",
                    line: 188,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "DELETE /ardupilot-manager/v1.0/endpoints",
                    purpose: "remove unprotected endpoint from EndpointCard trash action",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointCard.vue",
                    line: 167,
                    anchor: "await back_axios({",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "endpoint creation dialog",
                    store: "EndpointManager.vue show_creation_dialog",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "FAB opens CreationDialog for new endpoint draft",
                },
                "creation dialog visibility is ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "router selection draft",
                    store: "EndpointManager.vue selected_router, available_routers, updating_router",
                    ownership: StateOwnership::Shared,
                    notes: "radio group bound to preferred_router GET/POST cycle",
                },
                "router picker mirrors backend preferred_router but holds in-flight selection during POST",
            ),
            Rationaled::new(
                ClientState {
                    name: "heartbeat liveness gate",
                    store: "EndpointManager.vue heartbeat_expired, heartbeat_timeout",
                    ownership: StateOwnership::Shared,
                    notes: "autopilot_data.last_heartbeat_date watched with 3s expiry for board_is_running",
                },
                "endpoint creation is gated client-side on recent MAVLink heartbeat age",
            ),
            Rationaled::new(
                ClientState {
                    name: "per-endpoint edit draft",
                    store: "EndpointCard.vue updated_endpoint, show_edit_dialog",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "local copy for enable toggle and edit dialog before PUT",
                },
                "each card clones endpoint for optimistic enable switch and edit form",
            ),
            Rationaled::new(
                ClientState {
                    name: "configured endpoints list",
                    store: "store/autopilot available_endpoints, updating_endpoints",
                    ownership: StateOwnership::BackendOwned,
                    notes: "AutopilotEndpoint[] mirrored from GET /endpoints poll",
                },
                "endpoint list is fetched from ardupilot_manager and refreshed after mutations",
            ),
        ]),
    };
