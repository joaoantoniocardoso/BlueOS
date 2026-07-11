mod ardupilot_manager;
mod commander;
mod disk_usage;
mod helper;
mod kraken;

use crate::journey::UserJourney;

pub fn all_journeys() -> Vec<UserJourney> {
    let mut journeys = ardupilot_manager::journeys();
    journeys.extend(commander::journeys());
    journeys.extend(disk_usage::journeys());
    journeys.extend(helper::journeys());
    journeys.extend(kraken::journeys());
    journeys
}
