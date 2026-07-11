pub mod ardupilot_manager;
pub mod bag_of_holding;
pub mod beacon;
pub mod bridget;
pub mod cable_guy;
pub mod commander;
pub mod customization;
pub mod disk_usage;
pub mod filebrowser;
pub mod helper;
pub mod iperf3;
pub mod kraken;
pub mod linux2rest;
pub mod mavlink2rest;
pub mod mavlink_camera_manager;
pub mod nginx;
pub mod nmea_injector;
pub mod pardal;
pub mod ping;
pub mod recorder;
pub mod recorder_extractor;
pub mod ttyd;
pub mod user_terminal;
pub mod versionchooser;
pub mod wifi;
pub mod zenohd;

use crate::observed::ObservedFacts;
use crate::runtime::RuntimeFacts;
use crate::service::ServiceDefinition;

pub fn all_observed() -> Vec<ObservedFacts> {
    vec![
        ardupilot_manager::OBSERVED_FACTS,
        bag_of_holding::OBSERVED_FACTS,
        beacon::OBSERVED_FACTS,
        bridget::OBSERVED_FACTS,
        cable_guy::OBSERVED_FACTS,
        commander::OBSERVED_FACTS,
        customization::OBSERVED_FACTS,
        disk_usage::OBSERVED_FACTS,
        filebrowser::OBSERVED_FACTS,
        helper::OBSERVED_FACTS,
        iperf3::OBSERVED_FACTS,
        kraken::OBSERVED_FACTS,
        linux2rest::OBSERVED_FACTS,
        mavlink_camera_manager::OBSERVED_FACTS,
        mavlink2rest::OBSERVED_FACTS,
        nmea_injector::OBSERVED_FACTS,
        nginx::OBSERVED_FACTS,
        pardal::OBSERVED_FACTS,
        ping::OBSERVED_FACTS,
        recorder::OBSERVED_FACTS,
        recorder_extractor::OBSERVED_FACTS,
        ttyd::OBSERVED_FACTS,
        user_terminal::OBSERVED_FACTS,
        versionchooser::OBSERVED_FACTS,
        wifi::OBSERVED_FACTS,
        zenohd::OBSERVED_FACTS,
    ]
}

pub fn all_runtime() -> Vec<RuntimeFacts> {
    vec![
        ardupilot_manager::RUNTIME_FACTS,
        bag_of_holding::RUNTIME_FACTS,
        beacon::RUNTIME_FACTS,
        bridget::RUNTIME_FACTS,
        cable_guy::RUNTIME_FACTS,
        commander::RUNTIME_FACTS,
        customization::RUNTIME_FACTS,
        disk_usage::RUNTIME_FACTS,
        filebrowser::RUNTIME_FACTS,
        helper::RUNTIME_FACTS,
        iperf3::RUNTIME_FACTS,
        kraken::RUNTIME_FACTS,
        linux2rest::RUNTIME_FACTS,
        mavlink_camera_manager::RUNTIME_FACTS,
        mavlink2rest::RUNTIME_FACTS,
        nmea_injector::RUNTIME_FACTS,
        nginx::RUNTIME_FACTS,
        pardal::RUNTIME_FACTS,
        ping::RUNTIME_FACTS,
        recorder::RUNTIME_FACTS,
        recorder_extractor::RUNTIME_FACTS,
        ttyd::RUNTIME_FACTS,
        user_terminal::RUNTIME_FACTS,
        versionchooser::RUNTIME_FACTS,
        wifi::RUNTIME_FACTS,
        zenohd::RUNTIME_FACTS,
    ]
}

pub fn all_service_definitions() -> Vec<ServiceDefinition> {
    vec![
        ardupilot_manager::SERVICE_DEFINITION,
        bag_of_holding::SERVICE_DEFINITION,
        beacon::SERVICE_DEFINITION,
        bridget::SERVICE_DEFINITION,
        cable_guy::SERVICE_DEFINITION,
        commander::SERVICE_DEFINITION,
        customization::SERVICE_DEFINITION,
        disk_usage::SERVICE_DEFINITION,
        filebrowser::SERVICE_DEFINITION,
        helper::SERVICE_DEFINITION,
        iperf3::SERVICE_DEFINITION,
        kraken::SERVICE_DEFINITION,
        linux2rest::SERVICE_DEFINITION,
        mavlink_camera_manager::SERVICE_DEFINITION,
        mavlink2rest::SERVICE_DEFINITION,
        nmea_injector::SERVICE_DEFINITION,
        nginx::SERVICE_DEFINITION,
        pardal::SERVICE_DEFINITION,
        ping::SERVICE_DEFINITION,
        recorder::SERVICE_DEFINITION,
        recorder_extractor::SERVICE_DEFINITION,
        ttyd::SERVICE_DEFINITION,
        user_terminal::SERVICE_DEFINITION,
        versionchooser::SERVICE_DEFINITION,
        wifi::SERVICE_DEFINITION,
        zenohd::SERVICE_DEFINITION,
    ]
}
