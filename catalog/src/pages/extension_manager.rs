use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::ExtensionManager,
        route: Observed::known(
            "/tools/extensions-manager",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 127,
            },
        ),
        name: Observed::known(
            "Extension Manager",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 128,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/ExtensionManagerView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 129,
            },
        ),
        menu_title: Observed::unknown("not in menu"),
        advanced_only: Observed::unknown("not in menu"),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "settings",
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 401,
                },
            ),
            Evidenced::new(
                "helper",
                Evidence {
                    file: "core/frontend/src/components/kraken/BackAlleyTab.vue",
                    line: 286,
                },
            ),
            Evidenced::new(
                "bag",
                Evidence {
                    file: "core/frontend/src/components/kraken/BazaarTab.vue",
                    line: 57,
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "GET /kraken/v2.0/manifest/consolidated",
                    purpose: "fetch merged extension store catalog for Back Alley tab",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 906,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "GET /kraken/v2.0/extension/",
                    purpose: "poll installed extensions every 10s on Installed tab",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 912,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "GET /kraken/v2.0/container/",
                    purpose: "poll running extension containers every 10s",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 871,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "GET /kraken/v2.0/container/stats",
                    purpose: "poll per-container CPU and memory metrics every 25s",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 891,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/extension/install",
                    purpose: "install extension from store selection or creation modal",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 943,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "PUT /kraken/v2.0/extension/{identifier}/{version}",
                    purpose: "update installed extension to a newer tag with streamed pull progress",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 804,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "DELETE /kraken/v2.0/extension/{identifier}",
                    purpose: "uninstall extension from Installed tab or details modal",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 994,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/extension/{identifier}/disable",
                    purpose: "stop running extension container",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 1023,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/extension/{identifier}/{tag}/enable",
                    purpose: "enable and start extension at selected tag",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 1034,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/extension/{identifier}/restart",
                    purpose: "restart extension container",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 1047,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/extension/upload",
                    purpose: "sideload extension .tar archive from Installed tab file dialog",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 785,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/extension/upload/keep-alive?temp_tag={tag}",
                    purpose: "keep temporary uploaded image alive during metadata configuration",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 733,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/extension/upload/finalize?temp_tag={tag}",
                    purpose: "finalize sideloaded extension after configure step",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 1137,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "GET /kraken/v2.0/manifest/?data=false&enabled=false",
                    purpose: "list manifest sources when Extension Settings modal opens",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionSettingsModal.vue",
                    line: 311,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/manifest/?validate_url={bool}",
                    purpose: "add custom manifest source from settings modal",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionSettingsModal.vue",
                    line: 323,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "PUT /kraken/v2.0/manifest/{identifier}/details?validate_url={bool}",
                    purpose: "update non-factory manifest source metadata",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionSettingsModal.vue",
                    line: 355,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "DELETE /kraken/v2.0/manifest/{identifier}",
                    purpose: "remove manifest source from settings modal",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionSettingsModal.vue",
                    line: 341,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "PUT /kraken/v2.0/manifest/orders",
                    purpose: "persist drag-and-drop manifest source priority order",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionSettingsModal.vue",
                    line: 374,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/manifest/{identifier}/enable",
                    purpose: "enable factory or custom manifest source",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionSettingsModal.vue",
                    line: 388,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "POST /kraken/v2.0/manifest/{identifier}/disable",
                    purpose: "disable manifest source from settings modal",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionSettingsModal.vue",
                    line: 388,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "zenoh query kraken/extension/logs/request?extension_name={identifier}",
                    purpose: "fetch historical extension logs when logs modal opens",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionLogsModal.vue",
                    line: 243,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Kraken),
                    endpoint: "zenoh subscribe kraken/extension/logs/{topic}",
                    purpose: "stream live extension logs in logs modal",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/modals/ExtensionLogsModal.vue",
                    line: 214,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::BagOfHolding),
                    endpoint: "GET /bag/v1.0/get/major_tom",
                    purpose: "resolve Bazaar iframe inventory_url from Major Tom extension config",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/BazaarTab.vue",
                    line: 103,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::External,
                    endpoint: "GET {major_tom.inventory_url}",
                    purpose: "BrIframe embeds external Bazaar storefront when Major Tom is configured",
                },
                Evidence {
                    file: "core/frontend/src/components/kraken/BazaarTab.vue",
                    line: 45,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Zenohd),
                    endpoint: "WS /zenoh-api/",
                    purpose: "open shared zenoh-ts session on mount for extension log queries",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionManagerView.vue",
                    line: 639,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "active toolbar tab",
                    store: "ExtensionManagerView.vue data tab",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "Store / Bazaar / Installed tabs",
                },
                "tab selection is ephemeral UI routing between child tab components",
            ),
            Rationaled::new(
                ClientState {
                    name: "consolidated store manifest",
                    store: "ExtensionManagerView.vue data manifest",
                    ownership: StateOwnership::Shared,
                    notes: "ExtensionData[] or error string from fetchConsolidatedManifests",
                },
                "store catalog is fetched from kraken then rendered and filtered client-side",
            ),
            Rationaled::new(
                ClientState {
                    name: "installed extensions map",
                    store: "ExtensionManagerView.vue data installed_extensions",
                    ownership: StateOwnership::Shared,
                    notes: "Dictionary keyed by identifier with per-card loading flag",
                },
                "installed list is mirrored from kraken and mutated locally during operations",
            ),
            Rationaled::new(
                ClientState {
                    name: "running containers and metrics",
                    store: "ExtensionManagerView.vue data running_containers / metrics",
                    ownership: StateOwnership::Shared,
                    notes: "joined with installed cards for status CPU/memory display",
                },
                "container list and stats are cached client-side between kraken polls",
            ),
            Rationaled::new(
                ClientState {
                    name: "docker pull progress",
                    store: "ExtensionManagerView.vue data pull_output / download_percentage / extraction_percentage",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "PullTracker parses streamed install/update output",
                },
                "pull progress text and percentages are client-derived from kraken download streams",
            ),
            Rationaled::new(
                ClientState {
                    name: "active install operation",
                    store: "ExtensionManagerView.vue localStorage ACTIVE_OPERATION_KEY",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "persists in-flight install/update identifier across reloads",
                },
                "operation persistence survives refresh until container appears or user clears state",
            ),
            Rationaled::new(
                ClientState {
                    name: "tar sideload wizard",
                    store: "ExtensionManagerView.vue install_from_file_phase / uploadPhases",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "upload/load/configure/install step machine with progress bars",
                },
                "sideload flow state is entirely client-side until finalize POST",
            ),
            Rationaled::new(
                ClientState {
                    name: "extension details modal selection",
                    store: "ExtensionManagerView.vue data selected_extension / show_dialog",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "store card click opens install/uninstall modal",
                },
                "modal holds ephemeral selection for store install actions",
            ),
            Rationaled::new(
                ClientState {
                    name: "manifest sources editor",
                    store: "ExtensionSettingsModal.vue data manifests draggable list",
                    ownership: StateOwnership::Shared,
                    notes: "reordered client-side before PUT /manifest/orders",
                },
                "manifest priority edits are staged in the browser until Apply",
            ),
            Rationaled::new(
                ClientState {
                    name: "bazaar iframe url",
                    store: "BazaarTab.vue data bazaar_url",
                    ownership: StateOwnership::Shared,
                    notes: "resolved from bag major_tom.inventory_url",
                },
                "bazaar embed URL is cached client-side from bag with periodic refresh",
            ),
        ]),
    };
