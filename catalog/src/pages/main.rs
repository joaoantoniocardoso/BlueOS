use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::Main,
        route: Observed::known(
            "/",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 12,
            },
        ),
        name: Observed::known(
            "Main",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 13,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/MainView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 14,
            },
        ),
        menu_title: Observed::unknown("not in menu"),
        advanced_only: Observed::unknown("not in menu"),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "helper",
                Evidence {
                    file: "core/frontend/src/views/MainView.vue",
                    line: 97,
                },
            ),
            Evidenced::new(
                "mavlink",
                Evidence {
                    file: "core/frontend/src/views/MainView.vue",
                    line: 98,
                },
            ),
            Evidenced::new(
                "video",
                Evidence {
                    file: "core/frontend/src/views/MainView.vue",
                    line: 99,
                },
            ),
            Evidenced::new(
                "system",
                Evidence {
                    file: "core/frontend/src/widgets/CpuPie.vue",
                    line: 51,
                },
            ),
            Evidenced::new(
                "customization",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/viewers/GenericViewer.vue",
                    line: 104,
                },
            ),
            Evidenced::new(
                "autopilot",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/viewers/modelHelper.ts",
                    line: 3,
                },
            ),
            Evidenced::new(
                "autopilot_data",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/viewers/modelHelper.ts",
                    line: 2,
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest COMMAND_LONG (REQUEST_MESSAGE for ATTITUDE @ 10Hz)",
                    purpose: "poll vehicle attitude for Digital Twin widget orientation prop",
                },
                Evidence {
                    file: "core/frontend/src/views/MainView.vue",
                    line: 266,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/streams",
                    purpose: "refresh stream list every 10s to build video preview dashboard cards",
                },
                Evidence {
                    file: "core/frontend/src/views/MainView.vue",
                    line: 267,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "GET /version-chooser/v1.0/version/current/",
                    purpose: "SelfHealthTest checks whether running tag is factory on mount",
                },
                Evidence {
                    file: "core/frontend/src/components/health/SelfHealthTest.vue",
                    line: 46,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Filebrowser),
                    endpoint: "GET /file-browser/api/resources/system_logs",
                    purpose: "download system logs link in factory-version recovery alert",
                },
                Evidence {
                    file: "core/frontend/src/components/health/SelfHealthTest.vue",
                    line: 55,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/system/cpu",
                    purpose: "CpuPie widget polls CPU usage every 2s",
                },
                Evidence {
                    file: "core/frontend/src/widgets/CpuPie.vue",
                    line: 89,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/system/memory",
                    purpose: "CpuPie widget polls memory usage every 2s",
                },
                Evidence {
                    file: "core/frontend/src/widgets/CpuPie.vue",
                    line: 90,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/system/disk",
                    purpose: "CpuPie widget polls root disk usage every 2s",
                },
                Evidence {
                    file: "core/frontend/src/widgets/CpuPie.vue",
                    line: 91,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::External,
                    endpoint: "HEAD /userdata/modeloverrides/{ALL.glb|vehicle/frame.glb}",
                    purpose: "GenericViewer resolves 3D model override path on mount via modelHelper",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/viewers/modelHelper.ts",
                    line: 69,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "dashboard widget grid",
                    store: "MainView.vue computed apps / baseApps / videoStreamWidgets",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "responsive CSS grid layout composing Digital Twin, CPU, networking, and per-stream cards",
                },
                "home dashboard assembles child widgets client-side; docs describe this as the first-open interface overview",
            ),
            Rationaled::new(
                ClientState {
                    name: "debounced attitude orientation",
                    store: "MainView.vue data lastOrientation + watch attitudeMessage",
                    ownership: StateOwnership::Shared,
                    notes: "3-degree threshold before updating GenericViewer orientation string prop",
                },
                "digital twin orientation is derived from mavlink ATTITUDE with client-side deadband to limit redraws",
            ),
            Rationaled::new(
                ClientState {
                    name: "internet onboarding alert visibility",
                    store: "MainView.vue computed has_internet",
                    ownership: StateOwnership::Shared,
                    notes: "reads helper.has_internet; dismissible welcome alert when offline",
                },
                "offline welcome banner mirrors helper connectivity state for getting-started update guidance",
            ),
            Rationaled::new(
                ClientState {
                    name: "factory version warning",
                    store: "SelfHealthTest.vue data is_running_factory",
                    ownership: StateOwnership::Shared,
                    notes: "true when current version tag equals factory",
                },
                "recovery alert state is derived client-side from versionchooser current tag",
            ),
            Rationaled::new(
                ClientState {
                    name: "video stream dashboard cards",
                    store: "MainView.vue computed videoStreamWidgets",
                    ownership: StateOwnership::Shared,
                    notes: "one card per running stream with icon/title/link to video manager",
                },
                "stream widgets are client-built from cached video.available_streams",
            ),
            Rationaled::new(
                ClientState {
                    name: "viewport dimensions",
                    store: "MainView.vue data windowHeight/windowWidth",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "updated on window resize handler",
                },
                "resize listener holds ephemeral layout inputs for responsive grid",
            ),
            Rationaled::new(
                ClientState {
                    name: "MAVLink ATTITUDE cache",
                    store: "store/mavlink available_messages",
                    ownership: StateOwnership::Shared,
                    notes: "ATTITUDE refreshed at 10Hz for orientation widget",
                },
                "attitude telemetry is cached client-side from mavlink2rest listeners",
            ),
            Rationaled::new(
                ClientState {
                    name: "video streams list",
                    store: "store/video available_streams",
                    ownership: StateOwnership::BackendOwned,
                    notes: "mirrored from GET /mavlink-camera-manager/streams",
                },
                "stream metadata displayed on dashboard originates from mavlink_camera_manager",
            ),
            Rationaled::new(
                ClientState {
                    name: "system information snapshot",
                    store: "store/system system cpu/memory/disk/network",
                    ownership: StateOwnership::BackendOwned,
                    notes: "CpuPie and Networking widgets read linux2rest-backed fields",
                },
                "CPU/memory/disk/network widgets mirror system-information REST responses",
            ),
        ]),
    };
