use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::Endpoints,
        route: Observed::known(
            "/vehicle/endpoints",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 37,
            },
        ),
        name: Observed::known(
            "Endpoints",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 38,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/EndpointView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 39,
            },
        ),
        menu_title: Observed::known(
            "MAVLink Endpoints",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 62,
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 65,
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "autopilot_data",
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointManager.vue",
                    line: 129,
                },
            ),
            Evidenced::new(
                "autopilot",
                Evidence {
                    file: "core/frontend/src/components/autopilot/EndpointManager.vue",
                    line: 130,
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
