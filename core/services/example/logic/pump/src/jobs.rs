use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;

use blueos_jobs::JobGraph as Graph;
use blueos_jobs::{JobGraph, JobId};

/// Leaf work units for the self-test sequence. Names are published on the standard jobs state (D-12).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PumpJobSpec {
    VerifyOff,
    RampUp,
    VerifyOn,
}

impl PumpJobSpec {
    pub fn step_name(&self) -> &'static str {
        match self {
            PumpJobSpec::VerifyOff => "verify_off",
            PumpJobSpec::RampUp => "ramp_up",
            PumpJobSpec::VerifyOn => "verify_on",
        }
    }
}

/// IO requests for the simulated pump adapter (D-04). The kernel runs these asynchronously and delivers
/// [`PumpCommand::JobIoFinished`] back to the inbox.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PumpIoRequest {
    RunStep { job_id: JobId, step: PumpJobSpec },
}

pub fn self_test_job_graph() -> JobGraph<PumpJobSpec> {
    Graph::Sequence(vec![
        Graph::Leaf(PumpJobSpec::VerifyOff),
        Graph::Leaf(PumpJobSpec::RampUp),
        Graph::Leaf(PumpJobSpec::VerifyOn),
    ])
}

pub fn job_spec_name(job_spec: &PumpJobSpec) -> String {
    job_spec.step_name().to_string()
}
