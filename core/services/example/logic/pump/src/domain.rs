use blueos_cqrs::{Decision, Domain};
use blueos_jobs::{JobId, Jobs};

use crate::command::{self, PumpCommand};
use crate::event::PumpEvent;
use crate::jobs::{PumpIoRequest, PumpJobSpec};
use crate::query::{PumpQuery, PumpQueryView};
use crate::snapshot::PumpSnapshot;

pub struct PumpDomain;

impl Domain for PumpDomain {
    type Command = PumpCommand;
    type Event = PumpEvent;
    type Query = PumpQuery;
    type View = PumpQueryView;
    type Snapshot = PumpSnapshot;
    type IoRequest = PumpIoRequest;
    type JobSpec = PumpJobSpec;

    fn handle_command(
        snapshot: &mut Self::Snapshot,
        jobs: &mut Jobs<Self::JobSpec>,
        command: Self::Command,
    ) -> Decision<Self> {
        command::handle_command(snapshot, jobs, command)
    }

    fn handle_query(
        snapshot: &Self::Snapshot,
        _jobs: &Jobs<Self::JobSpec>,
        query: Self::Query,
    ) -> Self::View {
        match query {
            PumpQuery::Level => PumpQueryView {
                level: snapshot.level,
                max_level: snapshot.effective_max_level(),
            },
        }
    }

    fn io_from_job(job_id: JobId, job_spec: &Self::JobSpec) -> Self::IoRequest {
        PumpIoRequest::RunStep {
            job_id,
            step: job_spec.clone(),
        }
    }
}
