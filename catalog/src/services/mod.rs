pub mod ardupilot_manager;
pub mod beacon;
pub mod cable_guy;
pub mod commander;
pub mod disk_usage;
pub mod helper;
pub mod kraken;

use crate::observed::ObservedFacts;
use crate::runtime::RuntimeFacts;
use crate::service::ServiceDefinition;

pub fn all_observed() -> Vec<ObservedFacts> {
    vec![
        ardupilot_manager::observed_facts(),
        beacon::observed_facts(),
        cable_guy::observed_facts(),
        commander::observed_facts(),
        disk_usage::observed_facts(),
        helper::observed_facts(),
        kraken::observed_facts(),
    ]
}

pub fn all_runtime() -> Vec<RuntimeFacts> {
    vec![
        ardupilot_manager::runtime_facts(),
        beacon::runtime_facts(),
        cable_guy::runtime_facts(),
        commander::runtime_facts(),
        disk_usage::runtime_facts(),
        helper::runtime_facts(),
        kraken::runtime_facts(),
    ]
}

pub fn all_service_definitions() -> Vec<ServiceDefinition> {
    vec![
        ardupilot_manager::service_definition(),
        beacon::service_definition(),
        cable_guy::service_definition(),
        commander::service_definition(),
        disk_usage::service_definition(),
        helper::service_definition(),
        kraken::service_definition(),
    ]
}
