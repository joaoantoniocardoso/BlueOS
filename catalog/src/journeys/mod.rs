mod ardupilot_manager;
mod bag_of_holding;
mod beacon;
mod cable_guy;
mod commander;
mod disk_usage;
mod helper;
mod kraken;
mod versionchooser;
mod wifi;

use crate::journey::UserJourney;

pub fn all_journeys() -> Vec<UserJourney> {
    let mut journeys = ardupilot_manager::journeys();
    journeys.extend(bag_of_holding::journeys());
    journeys.extend(beacon::journeys());
    journeys.extend(cable_guy::journeys());
    journeys.extend(commander::journeys());
    journeys.extend(disk_usage::journeys());
    journeys.extend(helper::journeys());
    journeys.extend(kraken::journeys());
    journeys.extend(versionchooser::journeys());
    journeys.extend(wifi::journeys());
    journeys
}
