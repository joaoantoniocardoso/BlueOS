pub mod ardupilot_manager;
pub mod disk_usage;
pub mod helper;
pub mod kraken;

use crate::observed::ObservedFacts;
use crate::runtime::RuntimeFacts;
use crate::service::ServiceDefinition;

pub fn all_observed() -> Vec<ObservedFacts> {
    vec![
        ardupilot_manager::observed_facts(),
        disk_usage::observed_facts(),
        helper::observed_facts(),
        kraken::observed_facts(),
    ]
}

pub fn all_runtime() -> Vec<RuntimeFacts> {
    vec![
        ardupilot_manager::runtime_facts(),
        disk_usage::runtime_facts(),
        kraken::runtime_facts(),
    ]
}

pub fn all_service_definitions() -> Vec<ServiceDefinition> {
    vec![
        ardupilot_manager::service_definition(),
        disk_usage::service_definition(),
        helper::service_definition(),
        kraken::service_definition(),
    ]
}
