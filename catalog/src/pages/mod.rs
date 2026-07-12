mod autopilot;
mod available_services;
mod bag_editor;
mod bridges;
mod disk;
mod endpoints;
mod extension_manager;
mod extensions;
mod file_browser;
mod log_browser;
mod main;
mod mavlink_inspector;
mod network_test;
mod nmea_injector;
mod parameter_editor;
mod pings;
mod records;
mod settings;
mod system_information;
mod terminal;
mod vehicle_setup;
mod version_chooser;
mod video_manager;
mod zenoh_inspector;

use crate::page::Page;

pub const PAGES: &[Page] = &[
    main::PAGE,
    autopilot::PAGE,
    vehicle_setup::PAGE,
    pings::PAGE,
    log_browser::PAGE,
    endpoints::PAGE,
    parameter_editor::PAGE,
    file_browser::PAGE,
    disk::PAGE,
    terminal::PAGE,
    version_chooser::PAGE,
    video_manager::PAGE,
    records::PAGE,
    bridges::PAGE,
    nmea_injector::PAGE,
    available_services::PAGE,
    system_information::PAGE,
    mavlink_inspector::PAGE,
    network_test::PAGE,
    bag_editor::PAGE,
    extensions::PAGE,
    extension_manager::PAGE,
    zenoh_inspector::PAGE,
    settings::PAGE,
];

pub fn all_pages() -> Vec<Page> {
    PAGES.to_vec()
}
