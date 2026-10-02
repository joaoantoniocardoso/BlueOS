//! `Handlers` that miss the IO query `Probe`.

use blueos_idl::msg::blueos_example_msgs::SetLevelRequest;
use blueos_service::Refusal;
use blueos_tank_app::endpoints::Handlers;
use blueos_tank_domain::{Tank, TankQuery, TankRequest};

struct ForgetfulHandlers;

impl Handlers<Tank> for ForgetfulHandlers {
    fn set_level(&self, _request: SetLevelRequest) -> Result<TankRequest, Refusal> {
        Ok(TankRequest::Drain)
    }

    fn level_after_fill(&self, _request: SetLevelRequest) -> Result<TankQuery, Refusal> {
        Ok(TankQuery::Level)
    }
}

fn main() {}
