mod ardupilot_manager;
mod disk_usage;
mod kraken;

use crate::journey::UserJourney;

pub fn all_journeys() -> Vec<UserJourney> {
    let mut journeys = ardupilot_manager::journeys();
    journeys.extend(kraken::journeys());
    journeys
}
