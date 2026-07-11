use std::collections::{HashMap, HashSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::id::ServiceId;
use crate::provenance::AssertedSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct FeatureId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Feature {
    pub id: FeatureId,
    pub aggregate: String,
    pub origin_service: ServiceId,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FeatureCatalog {
    features: Vec<Feature>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AggregateGroup {
    pub aggregate: String,
    pub features: Vec<FeatureId>,
}

impl FeatureCatalog {
    pub fn bootstrap() -> Self {
        Self::from_catalog(&Catalog::bootstrap())
    }

    pub fn from_catalog(catalog: &Catalog) -> Self {
        let mut features = Vec::new();
        for service in catalog.services() {
            if let AssertedSet::Established { items } = &service.capabilities {
                for cap in items {
                    let id = FeatureId(cap.value.0.clone());
                    let aggregate = aggregate_of(&id)
                        .unwrap_or_else(|| panic!("unmapped capability: {}", id.0))
                        .to_string();
                    features.push(Feature {
                        id,
                        aggregate,
                        origin_service: service.id.clone(),
                        rationale: cap.rationale.clone(),
                    });
                }
            }
        }
        Self { features }
    }

    pub fn features(&self) -> &[Feature] {
        &self.features
    }

    pub fn aggregate_view(&self) -> Vec<AggregateGroup> {
        let mut by_aggregate: HashMap<&str, Vec<FeatureId>> = HashMap::new();
        for feature in &self.features {
            by_aggregate
                .entry(feature.aggregate.as_str())
                .or_default()
                .push(feature.id.clone());
        }
        let mut groups: Vec<AggregateGroup> = by_aggregate
            .into_iter()
            .map(|(aggregate, mut features)| {
                features.sort_by(|left, right| left.0.cmp(&right.0));
                AggregateGroup {
                    aggregate: aggregate.to_string(),
                    features,
                }
            })
            .collect();
        groups.sort_by(|left, right| left.aggregate.cmp(&right.aggregate));
        groups
    }

    pub fn validate(&self, catalog: &Catalog) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let known_services: HashSet<&ServiceId> =
            catalog.services().iter().map(|s| &s.id).collect();
        let distinct_capabilities = distinct_capability_count(catalog);

        let mut seen_ids = HashSet::new();
        for feature in &self.features {
            if feature.aggregate.is_empty() {
                errors.push(format!("feature {} has empty aggregate", feature.id.0));
            }
            if !known_services.contains(&feature.origin_service) {
                errors.push(format!(
                    "feature {} references unknown origin_service {}",
                    feature.id.0, feature.origin_service.0
                ));
            }
            if !seen_ids.insert(&feature.id) {
                errors.push(format!("duplicate feature id {}", feature.id.0));
            }
        }

        if self.features.len() != distinct_capabilities {
            errors.push(format!(
                "feature count {} does not match distinct capability count {distinct_capabilities}",
                self.features.len()
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn distinct_capability_count(catalog: &Catalog) -> usize {
    let mut seen = HashSet::new();
    for service in catalog.services() {
        if let AssertedSet::Established { items } = &service.capabilities {
            for item in items {
                seen.insert(&item.value);
            }
        }
    }
    seen.len()
}

fn aggregate_of(id: &FeatureId) -> Option<&'static str> {
    match id.0.as_str() {
        "manage_autopilot_lifecycle"
        | "select_flight_controller_board"
        | "detect_flight_controllers"
        | "configure_sitl_frame"
        | "flash_firmware"
        | "query_vehicle_firmware_info" => Some("autopilot"),
        "manage_mavlink_router"
        | "manage_mavlink_endpoints"
        | "access_mavlink_over_rest"
        | "inspect_live_mavlink_messages"
        | "advertise_cameras_over_mavlink" => Some("mavlink"),
        "list_detected_ping_sensors"
        | "connect_ping_viewer_to_sonar"
        | "enable_ping1d_mavlink_distance" => Some("sonar"),
        "create_nmea_socket" | "remove_nmea_socket" | "list_nmea_sockets" => Some("gps_nmea"),
        "create_serial_to_udp_bridge"
        | "remove_serial_bridge"
        | "list_configured_serial_bridges"
        | "manage_serial_ports" => Some("serial_bridge"),
        "configure_camera_stream"
        | "remove_camera_stream"
        | "view_camera_streams"
        | "configure_legacy_camera"
        | "configure_uvc_device_controls"
        | "provide_webrtc_signalling" => Some("camera"),
        "record_vehicle_data_stream"
        | "browse_video_recordings"
        | "download_video_recording"
        | "delete_video_recording" => Some("recording"),
        "list_network_interfaces"
        | "list_ethernet_interfaces"
        | "set_interface_priority"
        | "assign_static_ip"
        | "acquire_dynamic_ip"
        | "get_interface_routes"
        | "configure_host_dns"
        | "retrieve_host_dns"
        | "enable_dhcp_server"
        | "disable_dhcp_server"
        | "get_dhcp_server_leases"
        | "get_dhcp_server_details" => Some("wired_network"),
        "scan_wifi_networks"
        | "connect_wifi_network"
        | "disconnect_wifi_network"
        | "list_saved_wifi_networks"
        | "remove_saved_wifi_network"
        | "get_wifi_status"
        | "toggle_hotspot"
        | "toggle_smart_hotspot"
        | "set_hotspot_credentials"
        | "get_hotspot_status" => Some("wireless_network"),
        "run_lan_speed_test"
        | "run_internet_speed_test"
        | "serve_iperf_bandwidth_test"
        | "probe_interface_connectivity"
        | "check_internet_connectivity"
        | "report_client_ip" => Some("net_diagnostics"),
        "advertise_mdns_domains"
        | "list_mdns_domains"
        | "set_mdns_hostname"
        | "get_mdns_hostname"
        | "set_vehicle_name"
        | "get_vehicle_name"
        | "discover_web_services"
        | "register_web_service"
        | "report_hardware_id"
        | "report_software_id" => Some("identity_discovery"),
        "update_blueos_version"
        | "switch_blueos_version"
        | "pull_blueos_version"
        | "list_remote_blueos_versions"
        | "list_local_blueos_versions"
        | "get_current_blueos_version"
        | "delete_local_blueos_version"
        | "update_bootstrap_image"
        | "get_current_bootstrap_version"
        | "reset_blueos_settings" => Some("versioning"),
        "install_extension"
        | "uninstall_extension"
        | "configure_extension"
        | "manage_extension_lifecycle"
        | "manage_manifests"
        | "browse_extension_store"
        | "docker_registry_login"
        | "list_docker_accounts" => Some("extensions"),
        "inspect_disk_usage"
        | "navigate_disk_usage"
        | "run_disk_speed_test"
        | "run_multi_size_disk_speed_test"
        | "delete_disk_paths" => Some("storage"),
        "manage_blueos_files"
        | "serve_webdav_uploads"
        | "set_bag_value"
        | "get_bag_value"
        | "overwrite_bag_store"
        | "edit_bag_json_store" => Some("files_kv"),
        "upload_branding_logo"
        | "get_branding_logo"
        | "remove_branding_logo"
        | "upload_branding_vehicle_image"
        | "get_branding_vehicle_image"
        | "remove_branding_vehicle_image"
        | "set_theme_color"
        | "reset_theme_color"
        | "get_theme_configuration"
        | "upload_model_override"
        | "list_model_overrides"
        | "delete_model_override"
        | "serve_frontend_spa"
        | "access_blueos_web_interface" => Some("branding_ui"),
        "run_host_command"
        | "shutdown_onboard_computer"
        | "reboot_onboard_computer"
        | "sync_system_time"
        | "setup_ssh"
        | "provide_system_information_over_rest"
        | "view_system_information"
        | "update_raspberry_eeprom"
        | "inspect_raspberry_eeprom" => Some("host_control"),
        "access_web_terminal"
        | "provide_shell_over_websocket"
        | "provide_interactive_root_shell" => Some("shell_access"),
        "route_pubsub_messages"
        | "inspect_zenoh_network"
        | "reverse_proxy_backend_services"
        | "reload_nginx"
        | "cache_external_http" => Some("platform_infra"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::AssertedSet;

    const EXPECTED_FEATURE_COUNT: usize = 129;
    const EXPECTED_AGGREGATE_COUNT: usize = 19;

    #[test]
    fn from_catalog_builds_expected_features() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        assert_eq!(features.features().len(), EXPECTED_FEATURE_COUNT);
    }

    #[test]
    fn aggregate_view_returns_expected_aggregates() {
        let features = FeatureCatalog::bootstrap();
        let view = features.aggregate_view();
        assert_eq!(view.len(), EXPECTED_AGGREGATE_COUNT);
        let total: usize = view.iter().map(|group| group.features.len()).sum();
        assert_eq!(total, EXPECTED_FEATURE_COUNT);
    }

    #[test]
    fn validate_passes_on_bootstrap() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        assert!(features.validate(&catalog).is_ok());
    }

    #[test]
    fn round_trips_through_serde_json() {
        let features = FeatureCatalog::bootstrap();
        let json = serde_json::to_string(&features).unwrap();
        let decoded: FeatureCatalog = serde_json::from_str(&json).unwrap();
        assert_eq!(features, decoded);
    }

    #[test]
    fn every_capability_becomes_exactly_one_feature() {
        let catalog = Catalog::bootstrap();
        let mut distinct = HashSet::new();
        for service in catalog.services() {
            if let AssertedSet::Established { items } = &service.capabilities {
                for item in items {
                    distinct.insert(item.value.0.clone());
                }
            }
        }
        let features = FeatureCatalog::from_catalog(&catalog);
        assert_eq!(features.features().len(), distinct.len());
        assert!(features.validate(&catalog).is_ok());
    }
}
