mod ardupilot_manager;
mod bag_of_holding;
mod beacon;
mod bridget;
mod cable_guy;
mod commander;
mod customization;
mod disk_usage;
mod filebrowser;
mod helper;
mod iperf3;
mod kraken;
mod linux2rest;
mod mavlink2rest;
mod mavlink_camera_manager;
mod nginx;
mod nmea_injector;
mod pardal;
mod ping;
mod recorder;
mod recorder_extractor;
mod ttyd;
mod user_terminal;
mod versionchooser;
mod wifi;
mod zenohd;

use crate::journey::UserJourney;

pub fn all_journeys() -> Vec<UserJourney> {
    [
        ardupilot_manager::JOURNEYS,
        bag_of_holding::JOURNEYS,
        beacon::JOURNEYS,
        bridget::JOURNEYS,
        cable_guy::JOURNEYS,
        commander::JOURNEYS,
        customization::JOURNEYS,
        disk_usage::JOURNEYS,
        filebrowser::JOURNEYS,
        helper::JOURNEYS,
        iperf3::JOURNEYS,
        kraken::JOURNEYS,
        linux2rest::JOURNEYS,
        mavlink2rest::JOURNEYS,
        mavlink_camera_manager::JOURNEYS,
        nginx::JOURNEYS,
        nmea_injector::JOURNEYS,
        pardal::JOURNEYS,
        ping::JOURNEYS,
        recorder::JOURNEYS,
        recorder_extractor::JOURNEYS,
        ttyd::JOURNEYS,
        user_terminal::JOURNEYS,
        versionchooser::JOURNEYS,
        wifi::JOURNEYS,
        zenohd::JOURNEYS,
    ]
    .concat()
}
