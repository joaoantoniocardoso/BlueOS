pub mod ardupilot_manager;
pub mod bag_of_holding;
pub mod beacon;
pub mod bridget;
pub mod cable_guy;
pub mod commander;
pub mod customization;
pub mod disk_usage;
pub mod helper;
pub mod kraken;
pub mod linux2rest;
pub mod mavlink2rest;
pub mod mavlink_camera_manager;
pub mod nginx;
pub mod nmea_injector;
pub mod pardal;
pub mod ping;
pub mod recorder_extractor;
pub mod ttyd;
pub mod versionchooser;
pub mod wifi;
pub mod zenohd;

use crate::observed::ObservedFacts;
use crate::runtime::RuntimeFacts;
use crate::service::ServiceDefinition;

pub fn all_observed() -> Vec<ObservedFacts> {
    vec![
        ardupilot_manager::observed_facts(),
        bag_of_holding::observed_facts(),
        beacon::observed_facts(),
        bridget::observed_facts(),
        cable_guy::observed_facts(),
        commander::observed_facts(),
        customization::observed_facts(),
        disk_usage::observed_facts(),
        helper::observed_facts(),
        kraken::observed_facts(),
        linux2rest::observed_facts(),
        mavlink_camera_manager::observed_facts(),
        mavlink2rest::observed_facts(),
        nmea_injector::observed_facts(),
        nginx::observed_facts(),
        pardal::observed_facts(),
        ping::observed_facts(),
        recorder_extractor::observed_facts(),
        ttyd::observed_facts(),
        versionchooser::observed_facts(),
        wifi::observed_facts(),
        zenohd::observed_facts(),
    ]
}

pub fn all_runtime() -> Vec<RuntimeFacts> {
    vec![
        ardupilot_manager::runtime_facts(),
        bag_of_holding::runtime_facts(),
        beacon::runtime_facts(),
        bridget::runtime_facts(),
        cable_guy::runtime_facts(),
        commander::runtime_facts(),
        customization::runtime_facts(),
        disk_usage::runtime_facts(),
        helper::runtime_facts(),
        kraken::runtime_facts(),
        linux2rest::runtime_facts(),
        mavlink_camera_manager::runtime_facts(),
        mavlink2rest::runtime_facts(),
        nmea_injector::runtime_facts(),
        nginx::runtime_facts(),
        pardal::runtime_facts(),
        ping::runtime_facts(),
        recorder_extractor::runtime_facts(),
        ttyd::runtime_facts(),
        versionchooser::runtime_facts(),
        wifi::runtime_facts(),
        zenohd::runtime_facts(),
    ]
}

pub fn all_service_definitions() -> Vec<ServiceDefinition> {
    vec![
        ardupilot_manager::service_definition(),
        bag_of_holding::service_definition(),
        beacon::service_definition(),
        bridget::service_definition(),
        cable_guy::service_definition(),
        commander::service_definition(),
        customization::service_definition(),
        disk_usage::service_definition(),
        helper::service_definition(),
        kraken::service_definition(),
        linux2rest::service_definition(),
        mavlink_camera_manager::service_definition(),
        mavlink2rest::service_definition(),
        nmea_injector::service_definition(),
        nginx::service_definition(),
        pardal::service_definition(),
        ping::service_definition(),
        recorder_extractor::service_definition(),
        ttyd::service_definition(),
        versionchooser::service_definition(),
        wifi::service_definition(),
        zenohd::service_definition(),
    ]
}
