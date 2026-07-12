use std::collections::{HashMap, HashSet};

use schemars::JsonSchema;
use serde::Serialize;

use crate::catalog::Catalog;
use crate::cluster::{greedy_modularity_communities, modularity_q_indices};
use crate::id::{JourneyId, ServiceId};
use crate::provenance::{AssertedSet, GroundedSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct FeatureId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Feature {
    pub id: FeatureId,
    pub aggregate: String,
    pub origin_service: ServiceId,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FeatureCatalog {
    features: Vec<Feature>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct AggregateGroup {
    pub aggregate: String,
    pub features: Vec<FeatureId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FeatureCommunity {
    pub members: Vec<FeatureId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct JourneyView {
    pub communities: Vec<FeatureCommunity>,
    pub modularity: f64,
    pub unreferenced_features: Vec<FeatureId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Divergence {
    pub split_by_journey: Vec<(FeatureId, FeatureId, String)>,
    pub joined_by_journey: Vec<(FeatureId, FeatureId, String)>,
}

impl FeatureCatalog {
    pub fn bootstrap() -> Self {
        Self::from_catalog(&Catalog::bootstrap())
    }

    pub fn from_catalog(catalog: &Catalog) -> Self {
        let mut features = Vec::new();
        for service in catalog.services() {
            if let AssertedSet::Established { items } = &service.definition.capabilities {
                for cap in items.iter() {
                    let id = FeatureId(cap.value.to_string());
                    let aggregate = aggregate_of(&id)
                        .unwrap_or_else(|| panic!("unmapped capability: {}", id.0))
                        .to_string();
                    features.push(Feature {
                        id,
                        aggregate,
                        origin_service: service.id,
                        rationale: cap.rationale.to_string(),
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

    pub fn journey_view(&self, catalog: &Catalog) -> JourneyView {
        let n = self.features.len();
        let index: HashMap<&FeatureId, usize> = self
            .features
            .iter()
            .enumerate()
            .map(|(idx, feature)| (&feature.id, idx))
            .collect();
        let mut weights = vec![vec![0.0; n]; n];
        let mut referenced = HashSet::new();

        for journey in catalog.journeys() {
            let journey_features = journey_feature_indices(journey, catalog, &index);
            for &idx in &journey_features {
                referenced.insert(idx);
            }
            for left in 0..journey_features.len() {
                for right in (left + 1)..journey_features.len() {
                    let a = journey_features[left];
                    let b = journey_features[right];
                    weights[a][b] += 1.0;
                    weights[b][a] += 1.0;
                }
            }
        }

        let communities_idx = greedy_modularity_communities(n, &weights);
        let modularity = modularity_q_indices(&weights, &communities_idx);
        let mut communities: Vec<FeatureCommunity> = communities_idx
            .into_iter()
            .map(|community| {
                let mut members: Vec<FeatureId> = community
                    .iter()
                    .map(|idx| self.features[*idx].id.clone())
                    .collect();
                members.sort_by(|left, right| left.0.cmp(&right.0));
                FeatureCommunity { members }
            })
            .collect();
        sort_feature_communities(&mut communities);

        let mut unreferenced_features: Vec<FeatureId> = self
            .features
            .iter()
            .filter(|feature| !referenced.contains(index.get(&feature.id).expect("feature index")))
            .map(|feature| feature.id.clone())
            .collect();
        unreferenced_features.sort_by(|left, right| left.0.cmp(&right.0));

        JourneyView {
            communities,
            modularity,
            unreferenced_features,
        }
    }

    pub fn view_divergence(&self, catalog: &Catalog) -> Divergence {
        let journey_view = self.journey_view(catalog);
        let feature_aggregate: HashMap<&str, &str> = self
            .features
            .iter()
            .map(|feature| (feature.id.0.as_str(), feature.aggregate.as_str()))
            .collect();
        let feature_community = feature_community_map(&journey_view.communities);
        let community_sizes: HashMap<usize, usize> = journey_view
            .communities
            .iter()
            .enumerate()
            .map(|(idx, community)| (idx, community.members.len()))
            .collect();
        let pair_journeys = journey_cooccurrence_pairs(catalog, &index_from_features(self));

        let mut joined_by_journey = Vec::new();
        for (left_id, right_id) in pair_journeys.keys() {
            let left_agg = feature_aggregate[left_id.0.as_str()];
            let right_agg = feature_aggregate[right_id.0.as_str()];
            if left_agg == right_agg {
                continue;
            }
            let left_comm = feature_community[left_id];
            let right_comm = feature_community[right_id];
            if left_comm != right_comm {
                continue;
            }
            if community_sizes[&left_comm] < 2 {
                continue;
            }
            let bridge = format_aggregate_bridge(left_agg, right_agg);
            joined_by_journey.push((left_id.clone(), right_id.clone(), bridge));
        }
        joined_by_journey.sort_by(|left, right| {
            left.0
                 .0
                .cmp(&right.0 .0)
                .then_with(|| left.1 .0.cmp(&right.1 .0))
        });
        joined_by_journey.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);

        let mut split_by_journey = Vec::new();
        for aggregate_group in self.aggregate_view() {
            for left in 0..aggregate_group.features.len() {
                for right in (left + 1)..aggregate_group.features.len() {
                    let left_id = &aggregate_group.features[left];
                    let right_id = &aggregate_group.features[right];
                    let left_comm = feature_community[left_id];
                    let right_comm = feature_community[right_id];
                    if left_comm == right_comm {
                        continue;
                    }
                    if community_sizes[&left_comm] < 2 || community_sizes[&right_comm] < 2 {
                        continue;
                    }
                    let (left_id, right_id) = ordered_pair(left_id.clone(), right_id.clone());
                    split_by_journey.push((left_id, right_id, aggregate_group.aggregate.clone()));
                }
            }
        }
        split_by_journey.sort_by(|left, right| {
            left.0
                 .0
                .cmp(&right.0 .0)
                .then_with(|| left.1 .0.cmp(&right.1 .0))
        });

        Divergence {
            split_by_journey,
            joined_by_journey,
        }
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
                    feature.id.0,
                    feature.origin_service.as_str()
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
        if let AssertedSet::Established { items } = &service.definition.capabilities {
            for item in items.iter() {
                seen.insert(&item.value);
            }
        }
    }
    seen.len()
}

fn index_from_features(catalog: &FeatureCatalog) -> HashMap<FeatureId, usize> {
    catalog
        .features
        .iter()
        .enumerate()
        .map(|(idx, feature)| (feature.id.clone(), idx))
        .collect()
}

fn sort_feature_communities(communities: &mut [FeatureCommunity]) {
    for community in communities.iter_mut() {
        community
            .members
            .sort_by(|left, right| left.0.cmp(&right.0));
    }
    communities.sort_by(|left, right| {
        right.members.len().cmp(&left.members.len()).then_with(|| {
            left.members
                .first()
                .map(|id| id.0.as_str())
                .cmp(&right.members.first().map(|id| id.0.as_str()))
        })
    });
}

fn feature_community_map(communities: &[FeatureCommunity]) -> HashMap<FeatureId, usize> {
    let mut map = HashMap::new();
    for (idx, community) in communities.iter().enumerate() {
        for member in &community.members {
            map.insert(member.clone(), idx);
        }
    }
    map
}

fn journey_feature_indices(
    journey: &crate::journey::UserJourney,
    catalog: &Catalog,
    index: &HashMap<&FeatureId, usize>,
) -> Vec<usize> {
    let mut features = Vec::new();
    if let GroundedSet::Known { items } = &journey.capability_refs {
        for item in items.iter() {
            let id = FeatureId(item.value.to_string());
            if let Some(&idx) = index.get(&id) {
                features.push(idx);
            }
        }
    }
    if let Some(chain_id) = &journey.chains_from {
        if let Some(parent) = catalog
            .journeys()
            .iter()
            .find(|candidate| &candidate.id == chain_id)
        {
            features.extend(journey_feature_indices(parent, catalog, index));
        }
    }
    features.sort_unstable();
    features.dedup();
    features
}

fn journey_cooccurrence_pairs(
    catalog: &Catalog,
    feature_index: &HashMap<FeatureId, usize>,
) -> HashMap<(FeatureId, FeatureId), Vec<JourneyId>> {
    let known: HashSet<FeatureId> = feature_index.keys().cloned().collect();
    let reverse_index: HashMap<usize, FeatureId> = feature_index
        .iter()
        .map(|(id, idx)| (*idx, id.clone()))
        .collect();
    let forward_index: HashMap<&FeatureId, usize> =
        feature_index.iter().map(|(id, idx)| (id, *idx)).collect();
    let mut pairs: HashMap<(FeatureId, FeatureId), Vec<JourneyId>> = HashMap::new();

    for journey in catalog.journeys() {
        let journey_features: Vec<FeatureId> =
            journey_feature_indices(journey, catalog, &forward_index)
                .into_iter()
                .filter_map(|idx| reverse_index.get(&idx).cloned())
                .filter(|id| known.contains(id))
                .collect();
        for left in 0..journey_features.len() {
            for right in (left + 1)..journey_features.len() {
                let (pair_left, pair_right) = ordered_pair(
                    journey_features[left].clone(),
                    journey_features[right].clone(),
                );
                pairs
                    .entry((pair_left, pair_right))
                    .or_default()
                    .push(journey.id);
            }
        }
    }

    for journeys in pairs.values_mut() {
        journeys.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        journeys.dedup();
    }
    pairs
}

fn ordered_pair(left: FeatureId, right: FeatureId) -> (FeatureId, FeatureId) {
    if left.0 <= right.0 {
        (left, right)
    } else {
        (right, left)
    }
}

fn format_aggregate_bridge(left: &str, right: &str) -> String {
    if left <= right {
        format!("{left} + {right}")
    } else {
        format!("{right} + {left}")
    }
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
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.is_object());
    }

    #[test]
    fn every_capability_becomes_exactly_one_feature() {
        let catalog = Catalog::bootstrap();
        let mut distinct = HashSet::new();
        for service in catalog.services() {
            if let AssertedSet::Established { items } = &service.definition.capabilities {
                for item in items.iter() {
                    distinct.insert(item.value);
                }
            }
        }
        let features = FeatureCatalog::from_catalog(&catalog);
        assert_eq!(features.features().len(), distinct.len());
        assert!(features.validate(&catalog).is_ok());
    }

    #[test]
    fn journey_view_is_deterministic() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let first = features.journey_view(&catalog);
        let second = features.journey_view(&catalog);
        assert_eq!(first, second);
    }

    #[test]
    fn journey_view_co_referenced_capabilities_share_community() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let view = features.journey_view(&catalog);
        let flash = FeatureId("flash_firmware".to_string());
        let detect = FeatureId("detect_flight_controllers".to_string());
        let flash_community = view
            .communities
            .iter()
            .find(|community| community.members.contains(&flash))
            .expect("flash_firmware community");
        assert!(flash_community.members.contains(&detect));
    }

    #[test]
    fn journey_view_unreferenced_features_are_absent_from_journeys() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let view = features.journey_view(&catalog);
        assert!(!view.unreferenced_features.is_empty());

        let mut referenced = HashSet::new();
        for journey in catalog.journeys() {
            let GroundedSet::Known { items } = &journey.capability_refs else {
                continue;
            };
            for item in items.iter() {
                referenced.insert(item.value.as_str());
            }
        }
        for id in &view.unreferenced_features {
            assert!(!referenced.contains(id.0.as_str()));
        }
    }

    #[test]
    fn view_divergence_finds_cross_aggregate_joins() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let divergence = features.view_divergence(&catalog);
        assert!(!divergence.joined_by_journey.is_empty());
    }

    #[test]
    fn journey_view_and_divergence_round_trip_serde() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let view = features.journey_view(&catalog);
        let divergence = features.view_divergence(&catalog);

        let view_json = serde_json::to_string(&view).unwrap();
        let view_value: serde_json::Value = serde_json::from_str(&view_json).unwrap();
        assert!(view_value.is_object());

        let divergence_json = serde_json::to_string(&divergence).unwrap();
        let divergence_value: serde_json::Value = serde_json::from_str(&divergence_json).unwrap();
        assert!(divergence_value.is_object());
    }
}
