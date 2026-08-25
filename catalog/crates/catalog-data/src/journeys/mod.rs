mod ardupilot_manager;
mod bag_of_holding;
mod beacon;
mod bridget;
mod cable_guy;
mod commander;
mod customization;
mod disk_usage;
mod filebrowser;
mod frontend_calibration;
mod frontend_configure;
mod frontend_parameters;
mod frontend_video;
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

use catalog_kernel::id::journey::JourneyId;
use catalog_model::journey::UseCase;

pub fn all_journeys() -> Vec<UseCase> {
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
        frontend_calibration::JOURNEYS,
        frontend_configure::JOURNEYS,
        frontend_parameters::JOURNEYS,
        frontend_video::JOURNEYS,
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

pub fn source_file_for_journey(id: JourneyId) -> Option<&'static str> {
    let modules: &[(&str, &[UseCase])] = &[
        ("journeys/ardupilot_manager.rs", ardupilot_manager::JOURNEYS),
        ("journeys/bag_of_holding.rs", bag_of_holding::JOURNEYS),
        ("journeys/beacon.rs", beacon::JOURNEYS),
        ("journeys/bridget.rs", bridget::JOURNEYS),
        ("journeys/cable_guy.rs", cable_guy::JOURNEYS),
        ("journeys/commander.rs", commander::JOURNEYS),
        ("journeys/customization.rs", customization::JOURNEYS),
        ("journeys/disk_usage.rs", disk_usage::JOURNEYS),
        ("journeys/filebrowser.rs", filebrowser::JOURNEYS),
        (
            "journeys/frontend_calibration.rs",
            frontend_calibration::JOURNEYS,
        ),
        (
            "journeys/frontend_configure.rs",
            frontend_configure::JOURNEYS,
        ),
        (
            "journeys/frontend_parameters.rs",
            frontend_parameters::JOURNEYS,
        ),
        ("journeys/frontend_video.rs", frontend_video::JOURNEYS),
        ("journeys/helper.rs", helper::JOURNEYS),
        ("journeys/iperf3.rs", iperf3::JOURNEYS),
        ("journeys/kraken.rs", kraken::JOURNEYS),
        ("journeys/linux2rest.rs", linux2rest::JOURNEYS),
        ("journeys/mavlink2rest.rs", mavlink2rest::JOURNEYS),
        (
            "journeys/mavlink_camera_manager.rs",
            mavlink_camera_manager::JOURNEYS,
        ),
        ("journeys/nginx.rs", nginx::JOURNEYS),
        ("journeys/nmea_injector.rs", nmea_injector::JOURNEYS),
        ("journeys/pardal.rs", pardal::JOURNEYS),
        ("journeys/ping.rs", ping::JOURNEYS),
        ("journeys/recorder.rs", recorder::JOURNEYS),
        (
            "journeys/recorder_extractor.rs",
            recorder_extractor::JOURNEYS,
        ),
        ("journeys/ttyd.rs", ttyd::JOURNEYS),
        ("journeys/user_terminal.rs", user_terminal::JOURNEYS),
        ("journeys/versionchooser.rs", versionchooser::JOURNEYS),
        ("journeys/wifi.rs", wifi::JOURNEYS),
        ("journeys/zenohd.rs", zenohd::JOURNEYS),
    ];
    modules
        .iter()
        .find(|(_, journeys)| journeys.iter().any(|journey| journey.id == id))
        .map(|(path, _)| *path)
}
