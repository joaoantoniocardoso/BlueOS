use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::Autopilot,
        route: Observed::known(
            "/vehicle/autopilot",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 17,
                anchor: "path: '/vehicle/autopilot',",
            },
        ),
        name: Observed::known(
            "Autopilot",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 18,
                anchor: "name: 'Autopilot',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/Autopilot.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 19,
                anchor: "component: defineAsyncComponent(() => import('../views/Autop",
            },
        ),
        menu_title: Observed::known(
            "Autopilot Firmware",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 3,
                anchor: "title: 'Autopilot Firmware',",
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 6,
                anchor: "advanced: false,",
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "autopilot_data",
                Evidence {
                    file: "core/frontend/src/views/Autopilot.vue",
                    line: 158,
                    anchor: "import autopilot_data from '@/store/autopilot'",
                },
            ),
            Evidenced::new(
                "autopilot",
                Evidence {
                    file: "core/frontend/src/views/Autopilot.vue",
                    line: 159,
                    anchor: "import autopilot from '@/store/autopilot_manager'",
                },
            ),
            Evidenced::new(
                "commander",
                Evidence {
                    file: "core/frontend/src/components/autopilot/FirmwareManager.vue",
                    line: 231,
                    anchor: "import commander from '@/store/commander'",
                },
            ),
            Evidenced::new(
                "beacon",
                Evidence {
                    file: "core/frontend/src/components/autopilot/MasterEndpointManager.vue",
                    line: 74,
                    anchor: "import beacon from '@/store/beacon'",
                },
            ),
            Evidenced::new(
                "system",
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotSerialConfiguration.vue",
                    line: 95,
                    anchor: "import system_information from '@/store/system-information'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/available_boards",
                    purpose: "poll detected flight controllers for board-change dialog and firmware selector",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 46,
                    anchor: "const response: AxiosResponse = await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/board",
                    purpose: "poll active flight controller board shown in autopilot info header",
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
                    endpoint: "POST /ardupilot-manager/v1.0/board",
                    purpose: "change running board from BoardChangeDialog (pirate mode)",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/BoardChangeDialog.vue",
                    line: 94,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/firmware_info",
                    purpose: "poll firmware version and type for summary card",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 75,
                    anchor: "const response: AxiosResponse = await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/vehicle_type",
                    purpose: "poll configured vehicle type for summary card",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 91,
                    anchor: "const response: AxiosResponse = await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/firmware_vehicle_type",
                    purpose: "poll firmware vehicle type for summary card and firmware manager defaults",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 105,
                    anchor: "const response: AxiosResponse = await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/start",
                    purpose: "start autopilot process (pirate mode)",
                },
                Evidence {
                    file: "core/frontend/src/views/Autopilot.vue",
                    line: 284,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/stop",
                    purpose: "stop autopilot process (pirate mode)",
                },
                Evidence {
                    file: "core/frontend/src/views/Autopilot.vue",
                    line: 298,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/restart",
                    purpose: "restart autopilot after firmware install, SITL frame change, or serial config save",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 177,
                    anchor: "return back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/available_firmwares",
                    purpose: "list cloud firmware builds for selected board and vehicle in FirmwareManager",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/FirmwareManager.vue",
                    line: 412,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/install_firmware_from_url",
                    purpose: "flash firmware downloaded from ArduPilot cloud URL",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/FirmwareManager.vue",
                    line: 441,
                    anchor: "url: `${autopilot.API_URL}/install_firmware_from_url`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/restore_default_firmware",
                    purpose: "restore board default ArduSub firmware image",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/FirmwareManager.vue",
                    line: 447,
                    anchor: "url: `${autopilot.API_URL}/restore_default_firmware`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/install_firmware_from_file",
                    purpose: "upload and flash custom firmware binary from surface computer",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/FirmwareManager.vue",
                    line: 460,
                    anchor: "url: `${autopilot.API_URL}/install_firmware_from_file`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/endpoints/manual_board_master_endpoint",
                    purpose: "load master MAVLink endpoint for Manual external boards",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/MasterEndpointManager.vue",
                    line: 171,
                    anchor: "const response = await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/endpoints/manual_board_master_endpoint",
                    purpose: "save master MAVLink endpoint for Manual external boards",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/MasterEndpointManager.vue",
                    line: 193,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/sitl_frame",
                    purpose: "poll current SITL vehicle frame for dropdown",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 119,
                    anchor: "const response: AxiosResponse = await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/sitl_frame",
                    purpose: "set SITL frame then restart autopilot to apply",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/SitlConfiguration.vue",
                    line: 75,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/serials",
                    purpose: "poll serial port mappings for pirate-mode serial configuration panel",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 15,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "PUT /ardupilot-manager/v1.0/serials",
                    purpose: "save serial port mappings and restart autopilot",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotSerialConfiguration.vue",
                    line: 180,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::BagOfHolding),
                    endpoint: "POST /bag/v1.0/set/wizard",
                    purpose: "re-enable setup wizard from wizard hat button",
                },
                Evidence {
                    file: "core/frontend/src/store/bag.ts",
                    line: 40,
                    anchor: "return back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/serial?udev=true",
                    purpose: "list serial devices with udev metadata for port picker comboboxes",
                },
                Evidence {
                    file: "core/frontend/src/store/system-information.ts",
                    line: 276,
                    anchor: "await back_axios({",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "board change dialog visibility",
                    store: "Autopilot.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "show_board_change_dialog toggled by openBoardChangeDialog",
                },
                "board picker dialog open state is ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "autopilot info summary",
                    store: "Autopilot.vue computed autopilot_info",
                    ownership: StateOwnership::Shared,
                    notes: "Record assembled from current_board, firmware_info, vehicle_type fields",
                },
                "summary card derives display strings from polled ardupilot_manager metadata",
            ),
            Rationaled::new(
                ClientState {
                    name: "firmware install wizard state",
                    store: "FirmwareManager.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "InstallStatus, CloudFirmwareOptionsStatus, chosen_board/vehicle/url, install_result_message",
                },
                "firmware panel tracks multi-step install progress and user selections locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "master endpoint form draft",
                    store: "MasterEndpointManager.vue component data",
                    ownership: StateOwnership::Shared,
                    notes: "endpoint vs original_endpoint; validated against beacon IP lists before POST",
                },
                "master endpoint editor holds draft values and client-side IP validation before save",
            ),
            Rationaled::new(
                ClientState {
                    name: "serial port mapping draft",
                    store: "AutopilotSerialConfiguration.vue ports dict",
                    ownership: StateOwnership::Shared,
                    notes: "port_map C..I to combobox selections merged into new_data for PUT /serials",
                },
                "serial configuration maps UI comboboxes to SerialEndpoint list before save",
            ),
            Rationaled::new(
                ClientState {
                    name: "autopilot manager metadata",
                    store: "store/autopilot current_board, firmware_info, vehicle_type, sitl_frame, autopilot_serials",
                    ownership: StateOwnership::BackendOwned,
                    notes: "mirrored from ardupilot_manager REST polls",
                },
                "board/firmware/vehicle/endpoint metadata is fetched and displayed without client mutation beyond cache",
            ),
            Rationaled::new(
                ClientState {
                    name: "vehicle safety and type flags",
                    store: "store/autopilot_data is_safe, autopilot_type",
                    ownership: StateOwnership::Shared,
                    notes: "is_safe gates pirate-mode actions; autopilot_type selects banner image",
                },
                "safety overlay and banner derive from mavlink-fed autopilot_data getters",
            ),
            Rationaled::new(
                ClientState {
                    name: "restarting in-flight flag",
                    store: "store/autopilot restarting",
                    ownership: StateOwnership::Shared,
                    notes: "setRestarting wraps start/stop/restart/install API lifecycles",
                },
                "restarting boolean is client-managed request status around lifecycle calls",
            ),
        ]),
    };
