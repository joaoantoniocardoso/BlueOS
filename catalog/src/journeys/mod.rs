mod ardupilot_manager;

use crate::journey::UserJourney;

pub fn all_journeys() -> Vec<UserJourney> {
    ardupilot_manager::journeys()
}
