use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::Disk,
        route: Observed::known(
            "/tools/disk",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 47,
                anchor: "path: '/tools/file-browser/:path*',",
            },
        ),
        name: Observed::known(
            "Disk",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 48,
                anchor: "name: 'File Browser',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/Disk.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 49,
                anchor: "component: defineAsyncComponent(() => import('../views/FileB",
            },
        ),
        menu_title: Observed::known(
            "Disk",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 47,
                anchor: "icon: 'mdi-file-tree',",
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 50,
                anchor: "text: 'Browse all the files in BlueOS. Useful for fetching l",
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "disk",
                Evidence {
                    file: "core/frontend/src/views/Disk.vue",
                    line: 240,
                    anchor: "import disk_store from '@/store/disk'",
                },
            ),
            Evidenced::new(
                "settings",
                Evidence {
                    file: "core/frontend/src/components/disk/DiskSpeedGraph.vue",
                    line: 13,
                    anchor: "import settingsStore from '@/store/settings'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::DiskUsage),
                    endpoint: "GET /disk-usage/v1.0/disk/usage",
                    purpose: "fetch du-backed usage tree for current path (depth, include_files, min_size_bytes query params)",
                },
                Evidence {
                    file: "core/frontend/src/store/disk.ts",
                    line: 87,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::DiskUsage),
                    endpoint: "DELETE /disk-usage/v1.0/disk/paths/{target_path}",
                    purpose: "delete selected files or folders after browser confirm dialog",
                },
                Evidence {
                    file: "core/frontend/src/store/disk.ts",
                    line: 115,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::DiskUsage),
                    endpoint: "GET /disk-usage/v1.0/disk/speed/stream",
                    purpose: "stream multi-size disktest benchmark points for Speed Test tab",
                },
                Evidence {
                    file: "core/frontend/src/store/disk.ts",
                    line: 192,
                    anchor: "await back_axios({",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "active tab",
                    store: "Disk.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "activeTab selects Disk Usage vs Speed Test tab",
                },
                "tab selection is ephemeral UI state with no backend persistence",
            ),
            Rationaled::new(
                ClientState {
                    name: "directory navigation path",
                    store: "Disk.vue component data",
                    ownership: StateOwnership::Shared,
                    notes: "path updated by navigate/goUp; drives GET /disk/usage query",
                },
                "current directory path is held client-side and sent to disk_usage on each drill-down",
            ),
            Rationaled::new(
                ClientState {
                    name: "usage query defaults",
                    store: "Disk.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "depth=2, includeFiles=true, minSizeKb=0 passed to fetchUsage",
                },
                "fixed fetch parameters are component defaults not exposed in the UI",
            ),
            Rationaled::new(
                ClientState {
                    name: "delete selection",
                    store: "Disk.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "selectedPaths, allSelected/someSelected, toggleSelectAll",
                },
                "checkbox selection for bulk delete is ephemeral until confirmDeleteSelected runs",
            ),
            Rationaled::new(
                ClientState {
                    name: "disk usage tree",
                    store: "store/disk usage field",
                    ownership: StateOwnership::BackendOwned,
                    notes: "DiskUsageResponse mirrored from GET /disk/usage",
                },
                "usage tree is fetched from disk_usage and displayed without client mutation",
            ),
            Rationaled::new(
                ClientState {
                    name: "fetch and delete status flags",
                    store: "store/disk loading, deleting, error",
                    ownership: StateOwnership::Shared,
                    notes: "loading/deleting booleans and error strings around axios lifecycle",
                },
                "request status flags are client-managed wrappers around backend calls",
            ),
            Rationaled::new(
                ClientState {
                    name: "speed test results",
                    store: "store/disk speedResults",
                    ownership: StateOwnership::BackendOwned,
                    notes: "DiskSpeedTestPoint[] accumulated from streaming NDJSON fragments",
                },
                "benchmark points originate from disk_usage /speed/stream and are appended client-side as they arrive",
            ),
            Rationaled::new(
                ClientState {
                    name: "speed test progress",
                    store: "store/disk speedTesting, speedTestProgress, speedError",
                    ownership: StateOwnership::Shared,
                    notes: "in-flight flag, per-size progress text, and error message during stream parse",
                },
                "speed test UI state tracks streaming download progress locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "sorted usage children",
                    store: "Disk.vue computed sortedChildren",
                    ownership: StateOwnership::Shared,
                    notes: "usage.root.children sorted descending by size_bytes for table display",
                },
                "table ordering is client-derived from the cached usage tree",
            ),
            Rationaled::new(
                ClientState {
                    name: "usage bar percentages",
                    store: "Disk.vue method getPercentage",
                    ownership: StateOwnership::Shared,
                    notes: "child size_bytes as percentage of parent root size_bytes",
                },
                "progress bar widths are computed client-side from backend-reported sizes",
            ),
            Rationaled::new(
                ClientState {
                    name: "average speed summaries",
                    store: "Disk.vue computed avgWriteSpeed / avgReadSpeed",
                    ownership: StateOwnership::Shared,
                    notes: "mean of non-null write_speed/read_speed across speedResults",
                },
                "headline MiB/s averages are client-computed from streamed benchmark points",
            ),
        ]),
    };
