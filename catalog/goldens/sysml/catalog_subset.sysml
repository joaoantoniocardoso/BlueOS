// Generated from blueos-catalog. Rust catalog is source of truth.
package BlueOS_Catalog {
    private import ScalarValues::*;
    private import Requirements::*;
    private import VerificationCases::*;
    private import ModelingMetadata::*;
    private import Actions::*;
    private import UseCases::*;
    private import Parts::*;

    metadata def Evidence {
        attribute file : String;
        attribute line : Integer;
        attribute quote : String;
    }

    metadata def VersionPresence {
        attribute intro_commit : String;
        attribute present_in_tags : String;
        attribute present_on_master : Boolean;
        attribute present_on_1_4_dev : Boolean;
    }

    requirement def SystemRequirementCheck :> RequirementCheck {
        subject subj : Part;
    }
    requirement def RobustnessRequirementCheck :> RequirementCheck {
        subject subj : Part;
    }

    package peripherals {
        package sonar {
            port def RestPort {
                attribute path_prefix : String = "/ping/";
                attribute tcp_port : String = "9110";
                attribute versions : String = "v1.0";
            }

            port def SubprocessPort {
                attribute command : String = "{bridges} -u {ip}:{port} -p {serial_port.device}:{baud} {automatic_disconnect_clients}";
            }

            port def OutboundHttpPort {
                attribute url : String = "localhost:6040";
            }

            port def SettingsPort {
                attribute path : String = "/usr/blueos/userdata/settings/ping";
            }

            port def FilePort {
                attribute path : String = "/usr/blueos/userdata/settings/ping";
                attribute mode : String = "ReadWrite";
            }

            port def ZenohPort {
                attribute topics_produced : String = "services/ping/log";
                attribute topics_consumed : String = "";
            }

            part def ping {
                attribute tmux_name : String = "ping";
                @Evidence { file = "core/start-blueos-core"; line = 140; quote = "'ping',0,0,0,0,'nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICE"; }
                attribute entrypoint : String = "nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICES_PATH/ping/main.py $RUN_AS_REGULAR_USER_END";
                @Evidence { file = "core/start-blueos-core"; line = 140; quote = "'ping',0,0,0,0,'nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICE"; }
                attribute git_path : String = "core/services/ping";
                @Evidence { file = "core/services/ping/main.py"; line = 1; quote = "#! /usr/bin/env python3"; }
                attribute listen : String = "9110";
                @Evidence { file = "core/services/ping/main.py"; line = 86; quote = "config = Config(app=app, host='0.0.0.0', port=9110, log_conf"; }
                attribute bounded_context : String = "ping-sonar-integration";
                @Rationale { text = "provisional 2.0 domain: auto-detect Ping-family sonar devices, spawn UDP bridges, and optional MAVLink rangefinder forwarding"; }
                attribute tier : String = "Auxiliary";
                @Rationale { text = "opt-in Ping sonar peripheral; vehicle flight and MAVLink control do not require sonar bridging or rangefinder forwarding"; }
                attribute health : String = "implicit: process liveness via tmux; REST GET /sensors availability serves as health signal";
                @Rationale { text = "no dedicated /health route; uvicorn availability and probe loop continuity serve as health signal"; }
                @StatusInfo { status = StatusKind::tbd; }
                @Issue { text = "no CODEOWNERS or team metadata in observed artifact"; }
            }

            action def connect_ping_viewer_to_sonar_cbf29ce484222325 {
                doc
                /*
                 * connect_ping_viewer_to_sonar/cbf29ce484222325
                 */
            }

            use case def connect_ping_viewer_to_sonar {
                @VersionPresence { intro_commit = "5de1feb47221db3c88d2ed669ae91de43aeefd6d"; present_in_tags = "1.0.0.beta1,1.0.0.beta2,1.0.0.beta3,1.0.0.beta4,1.0.0.beta5,1.0.0.beta6,1.0.0.beta7,1.0.0.beta8,1.0.0.beta9,1.0.0.beta10,1.0.0.beta11,1.0.0.beta12,1.0.0-beta.13,1.0.0-beta.14,1.0.0-beta.15,1.0.0-beta.16,1.0.0,1.0.1,1.0.2-beta.1,1.1.0-beta.1,1.1.0-beta.2,1.1.0-beta.3,1.1.0-beta.4,1.1.0-beta5,1.1.0-beta.6,1.1.0-beta.7,1.1.0-beta.8,1.1.0-beta.9,1.1.0-beta.10,1.1.0-beta.11,1.1.0-beta.12,1.1.0-beta.13,1.1.0-beta.14,1.1.0-beta.15,1.1.0-beta.16,1.1.0-beta.17,1.1.0-beta.18,1.1.0-beta.19,1.1.0-beta.20,1.1.0-beta.21,1.1.0-beta.22,1.1.0-beta.23,1.1.0-beta.24,1.1.0-beta.25,1.1.0-beta.26,1.1.0-beta.27,1.1.0-beta.28,1.1.0-beta.29,1.1.0-beta.30,1.1.0,1.1.1-beta.1,1.1.1,1.1.2-beta.1,1.1.2-beta.2,1.2.0-beta.1,1.2.0-beta.2,1.2.0-beta.3,1.2.0-beta.4,1.2.0-beta.5,1.2.0-beta.6,1.2.0-beta.7,1.2.0-beta.8,1.2.0-beta.9,1.2.0,1.2.1,1.2.2,1.2.3,1.2.4,1.2.5,1.2.6,1.3.0-beta.1,1.3.0-beta.2,1.3.0-beta.3,1.3.0-beta.4,1.3.0-beta.5,1.3.0-beta.6,1.3.0-beta.7,1.3.0-beta.8,1.3.0-beta.9,1.3.0-beta.10,1.3.0,1.3.1,1.4.0-beta.1,1.4.0-beta.2,1.4.0-beta.3,1.4.0-beta.4,1.4.0-beta.5,1.4.0-beta.6,1.4.0-beta.7,1.4.0-beta.8,1.4.0-beta.9,1.4.0-beta.10,1.4.0-beta.11,1.4.0-beta.12,1.4.0-beta.13,1.4.0-beta.14,1.4.0-beta.15,1.4.0-beta.16,1.4.0-beta.17,1.4.0-beta.18,1.4.0-beta.19,1.4.0-beta.20,1.4.0-beta.21,1.4.0,1.4.1,1.4.2,1.4.3-beta.1,1.4.3-beta.2,1.4.3-beta.3,1.4.3-beta.4,1.4.3-beta.5,1.4.3-beta.6,1.4.3-beta.7,1.4.3-beta.8,1.4.3-beta.9,1.4.3-beta.10,1.4.3-beta.11,1.4.3,1.4.4-beta.1,1.4.4-beta.2,1.4.4-beta.3,1.4.4-beta.4,1.4.4-beta.5,1.4.4-beta.6,1.4.4-beta.7,1.4.4-beta.8,1.4.4-beta.9,1.4.4-beta.10,1.4.4-beta.11,1.4.4-beta.12,1.4.4-beta.13,1.4.4-beta.14,1.4.4-beta.15,1.4.4-beta.16,1.4.4-beta.17,1.4.4-beta.18,1.4.4-beta.19,1.4.4-beta.20,1.4.4-beta.21,1.4.4-beta.22,1.4.4-beta.23,1.4.4,1.4.5,1.5.0-beta.1,1.5.0-beta.2,1.5.0-beta.3,1.5.0-beta.4,1.5.0-beta.5,1.5.0-beta.6,1.5.0-beta.7,1.5.0-beta.8,1.5.0-beta.9,1.5.0-beta.10,1.5.0-beta.11,1.5.0-beta.12,1.5.0-beta.13,1.5.0-beta.14,1.5.0-beta.15,1.5.0-beta.16,1.5.0-beta.17,1.5.0-beta.18,1.5.0-beta.19,1.5.0-beta.20,1.5.0-beta.21,1.5.0-beta.22,1.5.0-beta.23,1.5.0-beta.24,1.5.0-beta.25,1.5.0-beta.26,1.5.0-beta.27,1.5.0-beta.28,1.5.0-beta.29,1.5.0-beta.30,1.5.0-beta.31,1.5.0-beta.32,1.5.0-beta.33,1.5.0-beta.34,1.5.0-beta.35,1.5.0-beta.36,1.5.0-beta.37,1.5.0-beta.38,1.5.0-beta.39,1.5.0-beta.40,1.4.3-beta.1.parameter-update"; present_on_master = true; present_on_1_4_dev = true; }
                subject subj : ping;
                actor operator;
                objective {
                    doc
                    /*
                     * assume: Ping family sonar device
                     * step 0: Open the Ping Sonar Devices page from the sidebar
                     * step 1: Note the connection endpoint shown on the device card (UDP bridge port or ethernet IP)
                     * step 2: Connect Ping Viewer on the surface computer to the sonar
                     */
                }
            }

            requirement def REQ_peripherals_sonar_FUN_connect_ping_viewer_to_sonar_cbf29ce484222325 :> FunctionalRequirementCheck {
                @VersionPresence { intro_commit = "5de1feb47221db3c88d2ed669ae91de43aeefd6d"; present_in_tags = "1.0.0.beta1,1.0.0.beta2,1.0.0.beta3,1.0.0.beta4,1.0.0.beta5,1.0.0.beta6,1.0.0.beta7,1.0.0.beta8,1.0.0.beta9,1.0.0.beta10,1.0.0.beta11,1.0.0.beta12,1.0.0-beta.13,1.0.0-beta.14,1.0.0-beta.15,1.0.0-beta.16,1.0.0,1.0.1,1.0.2-beta.1,1.1.0-beta.1,1.1.0-beta.2,1.1.0-beta.3,1.1.0-beta.4,1.1.0-beta5,1.1.0-beta.6,1.1.0-beta.7,1.1.0-beta.8,1.1.0-beta.9,1.1.0-beta.10,1.1.0-beta.11,1.1.0-beta.12,1.1.0-beta.13,1.1.0-beta.14,1.1.0-beta.15,1.1.0-beta.16,1.1.0-beta.17,1.1.0-beta.18,1.1.0-beta.19,1.1.0-beta.20,1.1.0-beta.21,1.1.0-beta.22,1.1.0-beta.23,1.1.0-beta.24,1.1.0-beta.25,1.1.0-beta.26,1.1.0-beta.27,1.1.0-beta.28,1.1.0-beta.29,1.1.0-beta.30,1.1.0,1.1.1-beta.1,1.1.1,1.1.2-beta.1,1.1.2-beta.2,1.2.0-beta.1,1.2.0-beta.2,1.2.0-beta.3,1.2.0-beta.4,1.2.0-beta.5,1.2.0-beta.6,1.2.0-beta.7,1.2.0-beta.8,1.2.0-beta.9,1.2.0,1.2.1,1.2.2,1.2.3,1.2.4,1.2.5,1.2.6,1.3.0-beta.1,1.3.0-beta.2,1.3.0-beta.3,1.3.0-beta.4,1.3.0-beta.5,1.3.0-beta.6,1.3.0-beta.7,1.3.0-beta.8,1.3.0-beta.9,1.3.0-beta.10,1.3.0,1.3.1,1.4.0-beta.1,1.4.0-beta.2,1.4.0-beta.3,1.4.0-beta.4,1.4.0-beta.5,1.4.0-beta.6,1.4.0-beta.7,1.4.0-beta.8,1.4.0-beta.9,1.4.0-beta.10,1.4.0-beta.11,1.4.0-beta.12,1.4.0-beta.13,1.4.0-beta.14,1.4.0-beta.15,1.4.0-beta.16,1.4.0-beta.17,1.4.0-beta.18,1.4.0-beta.19,1.4.0-beta.20,1.4.0-beta.21,1.4.0,1.4.1,1.4.2,1.4.3-beta.1,1.4.3-beta.2,1.4.3-beta.3,1.4.3-beta.4,1.4.3-beta.5,1.4.3-beta.6,1.4.3-beta.7,1.4.3-beta.8,1.4.3-beta.9,1.4.3-beta.10,1.4.3-beta.11,1.4.3,1.4.4-beta.1,1.4.4-beta.2,1.4.4-beta.3,1.4.4-beta.4,1.4.4-beta.5,1.4.4-beta.6,1.4.4-beta.7,1.4.4-beta.8,1.4.4-beta.9,1.4.4-beta.10,1.4.4-beta.11,1.4.4-beta.12,1.4.4-beta.13,1.4.4-beta.14,1.4.4-beta.15,1.4.4-beta.16,1.4.4-beta.17,1.4.4-beta.18,1.4.4-beta.19,1.4.4-beta.20,1.4.4-beta.21,1.4.4-beta.22,1.4.4-beta.23,1.4.4,1.4.5,1.5.0-beta.1,1.5.0-beta.2,1.5.0-beta.3,1.5.0-beta.4,1.5.0-beta.5,1.5.0-beta.6,1.5.0-beta.7,1.5.0-beta.8,1.5.0-beta.9,1.5.0-beta.10,1.5.0-beta.11,1.5.0-beta.12,1.5.0-beta.13,1.5.0-beta.14,1.5.0-beta.15,1.5.0-beta.16,1.5.0-beta.17,1.5.0-beta.18,1.5.0-beta.19,1.5.0-beta.20,1.5.0-beta.21,1.5.0-beta.22,1.5.0-beta.23,1.5.0-beta.24,1.5.0-beta.25,1.5.0-beta.26,1.5.0-beta.27,1.5.0-beta.28,1.5.0-beta.29,1.5.0-beta.30,1.5.0-beta.31,1.5.0-beta.32,1.5.0-beta.33,1.5.0-beta.34,1.5.0-beta.35,1.5.0-beta.36,1.5.0-beta.37,1.5.0-beta.38,1.5.0-beta.39,1.5.0-beta.40,1.4.3-beta.1.parameter-update"; present_on_master = true; present_on_1_4_dev = true; }
                subject subj : connect_ping_viewer_to_sonar_cbf29ce484222325;
                assume constraint {
                    doc
                    /*
                     * Ping family sonar device hardware is present
                     */
                }
                require constraint {
                    doc
                    /*
                     * step 0: Open the Ping Sonar Devices page from the sidebar
                     */
                }
                require constraint {
                    doc
                    /*
                     * step 1: Note the connection endpoint shown on the device card (UDP bridge port or ethernet IP)
                     */
                }
                require constraint {
                    doc
                    /*
                     * step 2: Connect Ping Viewer on the surface computer to the sonar
                     */
                }
            }

        }
    }
    package vehicle {
        package mavlink {
            port def RestPort {
                attribute path_prefix : String = "/mavlink2rest/";
                attribute tcp_port : String = "6040";
                attribute versions : String = "";
            }

            port def MavlinkPort {
                attribute role : String = "Consumer";
                attribute connect_string : String = "udpout:127.0.0.1:14001";
            }

            part def mavlink2rest {
                attribute tmux_name : String = "mavlink2rest";
                @Evidence { file = "core/start-blueos-core"; line = 121; quote = "'mavlink2rest',0,0,0,0,'mavlink2rest --connect=udpout:127.0."; }
                attribute entrypoint : String = "mavlink2rest --connect=udpout:127.0.0.1:14001 --server [::]:6040 --system-id $MAV_SYSTEM_ID --component-id $MAV_COMPONENT_ID_ONBOARD_COMPUTER4";
                @Evidence { file = "core/start-blueos-core"; line = 121; quote = "'mavlink2rest',0,0,0,0,'mavlink2rest --connect=udpout:127.0."; }
                @StatusInfo { status = StatusKind::tbd; }
                @Issue { text = "external Rust binary (upstream github.com/bluerobotics/mavlink2rest); no source tree in this repository"; }
                attribute listen : String = "6040";
                @Evidence { file = "core/start-blueos-core"; line = 121; quote = "'mavlink2rest',0,0,0,0,'mavlink2rest --connect=udpout:127.0."; }
                attribute bounded_context : String = "vehicle-mavlink-access";
                @Rationale { text = "provisional 2.0 domain: bridge vehicle MAVLink telemetry and commands to HTTP REST and WebSocket consumers"; }
                attribute tier : String = "Important";
                @Rationale { text = "Priority-tier data-plane bridge; frontend vehicle store, nmea_injector, ping, and helper lose MAVLink REST access when down, but GCS and router paths still reach the autopilot"; }
                attribute health : String = "implicit: process liveness via tmux; REST /mavlink2rest/ and WebSocket stream availability serve as health signals";
                @Rationale { text = "no dedicated /health route traced; bridge process continuity and HTTP listener serve as health signal"; }
                @StatusInfo { status = StatusKind::tbd; }
                @Issue { text = "no CODEOWNERS or team metadata in observed artifact"; }
            }

        }
    }
    part mavlink2rest : vehicle::mavlink::mavlink2rest;
    part ping : peripherals::sonar::ping;

    interface def RestBus {
        end a;
        end b;
    }
    interface ping_mavlink2rest_RestBus : RestBus connect ping to mavlink2rest {
        doc
        /*
         * observed OutboundHttp localhost:6040 (MavlinkComm.py:24) pairs with mavlink2rest listen 6040
         */
    }

    requirement blueosVerification {
        use case connect_ping_viewer_to_sonar_uc : connect_ping_viewer_to_sonar {
            subject subj : ping;
        }

    }

    requirement <REQ_peripherals_sonar_FUN_connect_ping_viewer_to_sonar_cbf29ce484222325> REQ_peripherals_sonar_FUN_connect_ping_viewer_to_sonar_cbf29ce484222325_u : REQ_peripherals_sonar_FUN_connect_ping_viewer_to_sonar_cbf29ce484222325 {
        subject subj : connect_ping_viewer_to_sonar_cbf29ce484222325;
        verify requirement connect_ping_viewer_to_sonar_uc;
    }

}
