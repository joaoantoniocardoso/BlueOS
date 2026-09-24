//! Sans-IO domain for the teaching example: a simulated pump with level control and a timed self-test job
//! graph (D-03, D-20). Each concept lives in its own module; start with [`PumpDomain`] and [`command`].

#![no_std]

extern crate alloc;

mod command;
mod domain;
mod event;
mod jobs;
mod phase;
mod query;
mod settings;
mod snapshot;
mod timers;

pub use command::PumpCommand;
pub use domain::PumpDomain;
pub use event::PumpEvent;
pub use jobs::{PumpIoRequest, PumpJobSpec, job_spec_name, self_test_job_graph};
pub use phase::{
    SELF_TEST_CANCELLED, SELF_TEST_FAILED, SELF_TEST_IDLE, SELF_TEST_PASSED, SELF_TEST_RUNNING,
};
pub use query::{PumpQuery, PumpQueryView};
pub use settings::{ExampleSettings, restart_required_field_names, settings_version};
pub use snapshot::PumpSnapshot;
pub use timers::SELF_TEST_TIMEOUT_TIMER;

#[cfg(test)]
mod tests;
