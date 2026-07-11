use crate::id::{CapabilityId, ServiceId};
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::VideoManager,
        route: Observed::known(
            "/vehicle/video-manager",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 62,
            },
        ),
        name: Observed::known(
            "Video Manager",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 63,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/VideoManagerView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 64,
            },
        ),
        menu_title: Observed::known(
            "Video Streams",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 128,
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 131,
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "video",
                Evidence {
                    file: "core/frontend/src/components/video-manager/VideoManager.vue",
                    line: 92,
                },
            ),
            Evidenced::new(
                "commander",
                Evidence {
                    file: "core/frontend/src/components/video-manager/VideoManager.vue",
                    line: 91,
                },
            ),
            Evidenced::new(
                "beacon",
                Evidence {
                    file: "core/frontend/src/components/video-manager/VideoDiagnosticHelper.vue",
                    line: 29,
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/v4l",
                    purpose: "poll video devices via VideoUpdater (5s interval); also re-fetched after control updates and block/unblock",
                },
                Evidence {
                    file: "core/frontend/src/store/video.ts",
                    line: 144,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/streams",
                    purpose: "poll configured streams via VideoUpdater (5s interval); also re-fetched after block/unblock",
                },
                Evidence {
                    file: "core/frontend/src/store/video.ts",
                    line: 168,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "POST /mavlink-camera-manager/streams",
                    purpose: "create new stream from VideoDevice or replace stream on edit (delete then create)",
                },
                Evidence {
                    file: "core/frontend/src/store/video.ts",
                    line: 120,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "DELETE /mavlink-camera-manager/delete_stream",
                    purpose: "remove stream or precede stream edit with delete",
                },
                Evidence {
                    file: "core/frontend/src/store/video.ts",
                    line: 102,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/thumbnail",
                    purpose: "fetch device preview thumbnails on demand or continuously (1s) from VideoThumbnail",
                },
                Evidence {
                    file: "core/frontend/src/store/video.ts",
                    line: 204,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "POST /mavlink-camera-manager/v4l",
                    purpose: "write V4L device control values (slider, menu, bool) from device controls dialog",
                },
                Evidence {
                    file: "core/frontend/src/components/video-manager/VideoControlsDialog.vue",
                    line: 190,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "POST /mavlink-camera-manager/block_source",
                    purpose: "block video source (pirate mode) from VideoDevice toggle",
                },
                Evidence {
                    file: "core/frontend/src/store/video.ts",
                    line: 252,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "POST /mavlink-camera-manager/unblock_source",
                    purpose: "unblock video source (pirate mode) from VideoDevice toggle",
                },
                Evidence {
                    file: "core/frontend/src/store/video.ts",
                    line: 270,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "POST /mavlink-camera-manager/reset_settings",
                    purpose: "reset mavlink-camera-manager settings to factory defaults from settings dialog",
                },
                Evidence {
                    file: "core/frontend/src/store/video.ts",
                    line: 287,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/sdp",
                    purpose: "download SDP file for UDP streams from VideoStream",
                },
                Evidence {
                    file: "core/frontend/src/components/video-manager/VideoStream.vue",
                    line: 311,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "GET /commander/v1.0/raspi_config/camera_legacy",
                    purpose: "read Raspberry Pi legacy camera toggle state on page mount",
                },
                Evidence {
                    file: "core/frontend/src/store/commander.ts",
                    line: 115,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "POST /commander/v1.0/raspi_config/camera_legacy",
                    purpose: "toggle Raspberry Pi legacy camera support from settings dialog",
                },
                Evidence {
                    file: "core/frontend/src/store/commander.ts",
                    line: 134,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::ConfigureStreamEndpoints,
                "VideoStreamCreationDialog.vue validates UDP/RTSP endpoints, builds default addresses from beacon IPs, and maps encode type to udp/udp265 schemes",
            ),
            Rationaled::new(
                CapabilityId::FilterDisplayableDevices,
                "VideoManager.vue computed video_devices filters unsupported encodes, hides RadCam secondary stream, and sorts devices client-side",
            ),
            Rationaled::new(
                CapabilityId::DiagnoseStreamAccessibility,
                "VideoDiagnosticHelper.vue derives whether any stream endpoint targets the client or vehicle IP from cached streams and beacon addresses",
            ),
            Rationaled::new(
                CapabilityId::ManageThumbnailPreview,
                "VideoThumbnail.vue orchestrates snapshot vs continuous (1s) preview modes with debounce, cooldown, and blob URL lifecycle",
            ),
            Rationaled::new(
                CapabilityId::ReplaceStreamConfiguration,
                "VideoStream.vue editStream deletes the existing stream then creates a new one because mavlink-camera-manager has no in-place update route",
            ),
        ]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "settings dialog state",
                    store: "VideoManager.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "show_settings_dialog and legacy_mode toggle before commander sync",
                },
                "gear-button settings dialog and legacy camera switch are ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "stream creation/editing form",
                    store: "VideoStreamCreationDialog.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "stream_name, selected_encode/size/interval, stream_endpoints, extended_configuration checkboxes",
                },
                "stream configuration wizard holds form fields locally until POST /streams",
            ),
            Rationaled::new(
                ClientState {
                    name: "device dialog visibility",
                    store: "VideoDevice.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "show_controls_dialog and show_stream_creation_dialog flags",
                },
                "per-device dialog open/close state is not persisted",
            ),
            Rationaled::new(
                ClientState {
                    name: "stream card UI state",
                    store: "VideoStream.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "show_stream_edit_dialog and isExpanded error-text expansion",
                },
                "stream card edit dialog and error expand/collapse are local UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "thumbnail preview mode",
                    store: "VideoThumbnail.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "continuous_mode, snapshot_in_progress, snapshot_cooldown, last_fetch_ms, local thumbnail ref",
                },
                "thumbnail component tracks preview mode and timing independent of backend",
            ),
            Rationaled::new(
                ClientState {
                    name: "device control update guard",
                    store: "VideoControlsDialog.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "updating flag disables controls while POST /v4l batch is in flight",
                },
                "controls dialog tracks in-flight submission state locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "video devices cache",
                    store: "store/video",
                    ownership: StateOwnership::BackendOwned,
                    notes: "available_devices mirrored from GET /v4l",
                },
                "device list is fetched from mavlink-camera-manager and displayed without client mutation",
            ),
            Rationaled::new(
                ClientState {
                    name: "video streams cache",
                    store: "store/video",
                    ownership: StateOwnership::BackendOwned,
                    notes: "available_streams mirrored from GET /streams",
                },
                "stream list and per-stream status/error fields come from backend polling",
            ),
            Rationaled::new(
                ClientState {
                    name: "fetch loading and error flags",
                    store: "store/video",
                    ownership: StateOwnership::Shared,
                    notes: "updating_devices, updating_streams, fetch_devices_error, fetch_streams_error",
                },
                "loading/error flags are client-managed around backend fetch lifecycle",
            ),
            Rationaled::new(
                ClientState {
                    name: "thumbnail blob cache",
                    store: "store/video thumbnails Map",
                    ownership: StateOwnership::Shared,
                    notes: "blob URLs created client-side from GET /thumbnail responses; revoked on refresh",
                },
                "thumbnails are fetched from backend but cached and lifecycle-managed in the browser",
            ),
            Rationaled::new(
                ClientState {
                    name: "device control values",
                    store: "device.controls in VideoControlsDialog.vue",
                    ownership: StateOwnership::Shared,
                    notes: "control Slider/Menu/Bool values from v4l response, mutated locally then POSTed",
                },
                "V4L controls are mirrored from backend device payload then edited client-side before submit",
            ),
            Rationaled::new(
                ClientState {
                    name: "filtered device list",
                    store: "VideoManager.vue computed video_devices",
                    ownership: StateOwnership::Shared,
                    notes: "filters H264 support, Fake/RadCam/Redirect rules, sorts by name",
                },
                "displayable devices are derived client-side from backend device and stream caches",
            ),
            Rationaled::new(
                ClientState {
                    name: "per-device stream associations",
                    store: "VideoDevice.vue computed device_streams via utils/video",
                    ownership: StateOwnership::Shared,
                    notes: "available_streams_from_device matches streams to device source paths",
                },
                "streams per device card are client-derived from the global streams cache",
            ),
            Rationaled::new(
                ClientState {
                    name: "beacon IP addresses",
                    store: "store/beacon",
                    ownership: StateOwnership::BackendOwned,
                    notes: "client_ip_address and nginx_ip_address read for endpoint defaults and diagnostics; beacon polls globally in background",
                },
                "page reads beacon IP cache populated by global background fetch, not page-triggered",
            ),
        ]),
    };
