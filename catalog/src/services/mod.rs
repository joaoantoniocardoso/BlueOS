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

use crate::service::Service;

pub const SERVICES: &[Service] = &[
    ardupilot_manager::SERVICE,
    bag_of_holding::SERVICE,
    beacon::SERVICE,
    bridget::SERVICE,
    cable_guy::SERVICE,
    commander::SERVICE,
    customization::SERVICE,
    disk_usage::SERVICE,
    filebrowser::SERVICE,
    helper::SERVICE,
    iperf3::SERVICE,
    kraken::SERVICE,
    linux2rest::SERVICE,
    mavlink_camera_manager::SERVICE,
    mavlink2rest::SERVICE,
    nmea_injector::SERVICE,
    nginx::SERVICE,
    pardal::SERVICE,
    ping::SERVICE,
    recorder::SERVICE,
    recorder_extractor::SERVICE,
    ttyd::SERVICE,
    user_terminal::SERVICE,
    versionchooser::SERVICE,
    wifi::SERVICE,
    zenohd::SERVICE,
];

pub fn all_services() -> Vec<Service> {
    SERVICES.to_vec()
}
