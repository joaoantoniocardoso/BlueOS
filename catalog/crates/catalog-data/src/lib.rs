pub mod backend;
pub mod capability_registry;
pub mod feature_intro;
pub mod feature_trace;
pub mod frontend;
pub mod journey_presence;
pub mod journeys;

use catalog_model::journey::UseCase;
use catalog_model::page::Page;
use catalog_model::service::Service;

pub const SERVICES: &[Service] = &[
    backend::services::ardupilot_manager::SERVICE,
    backend::services::bag_of_holding::SERVICE,
    backend::services::beacon::SERVICE,
    backend::services::bridget::SERVICE,
    backend::services::cable_guy::SERVICE,
    backend::services::commander::SERVICE,
    backend::services::customization::SERVICE,
    backend::services::disk_usage::SERVICE,
    backend::tools::filebrowser::SERVICE,
    backend::services::helper::SERVICE,
    backend::tools::iperf3::SERVICE,
    backend::services::kraken::SERVICE,
    backend::tools::linux2rest::SERVICE,
    backend::tools::mavlink_camera_manager::SERVICE,
    backend::tools::mavlink2rest::SERVICE,
    backend::services::nmea_injector::SERVICE,
    backend::tools::nginx::SERVICE,
    backend::services::pardal::SERVICE,
    backend::services::ping::SERVICE,
    backend::tools::recorder::SERVICE,
    backend::services::recorder_extractor::SERVICE,
    backend::tools::ttyd::SERVICE,
    backend::tools::user_terminal::SERVICE,
    backend::services::versionchooser::SERVICE,
    backend::services::wifi::SERVICE,
    backend::tools::zenohd::SERVICE,
];

pub fn all_services() -> Vec<Service> {
    SERVICES.to_vec()
}

pub fn all_journeys() -> Vec<UseCase> {
    journeys::all_journeys()
}

pub fn all_pages() -> Vec<Page> {
    frontend::all_pages()
}
