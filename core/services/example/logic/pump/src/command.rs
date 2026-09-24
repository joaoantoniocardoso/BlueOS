use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;

use crate::phase;
use blueos_cqrs::{Decision, Effect};
use blueos_jobs::{JobId, JobStatus, Jobs};

use crate::event::PumpEvent;
use crate::jobs::{PumpJobSpec, self_test_job_graph};
use crate::settings::ExampleSettings;
use crate::snapshot::PumpSnapshot;
use crate::timers::SELF_TEST_TIMEOUT_TIMER;

/// Inbox commands: user intents, settings updates, IO completions, and scheduled timeouts.
#[derive(Clone, Debug, PartialEq)]
pub enum PumpCommand {
    SetLevel { level: u8 },
    StartSelfTest,
    CancelSelfTest,
    UpdateSettings(ExampleSettings),
    JobIoFinished { job_id: JobId, succeeded: bool },
    SelfTestTimedOut,
}

pub fn handle_command(
    snapshot: &mut PumpSnapshot,
    jobs: &mut Jobs<PumpJobSpec>,
    command: PumpCommand,
) -> Decision<crate::domain::PumpDomain> {
    match command {
        PumpCommand::SetLevel { level } => handle_set_level(snapshot, level),
        PumpCommand::StartSelfTest => handle_start_self_test(snapshot, jobs),
        PumpCommand::CancelSelfTest => handle_cancel_self_test(snapshot, jobs),
        PumpCommand::UpdateSettings(settings) => handle_update_settings(snapshot, settings),
        PumpCommand::JobIoFinished { job_id, succeeded } => {
            handle_job_io_finished(snapshot, jobs, job_id, succeeded)
        }
        PumpCommand::SelfTestTimedOut => handle_self_test_timed_out(snapshot, jobs),
    }
}

fn handle_set_level(snapshot: &mut PumpSnapshot, level: u8) -> Decision<crate::domain::PumpDomain> {
    if snapshot.self_test_active {
        return Decision::new();
    }
    let clamped = snapshot.clamp_level(level);
    if clamped == snapshot.level {
        return Decision::new();
    }
    snapshot.level = clamped;
    Decision {
        events: vec![PumpEvent::LevelChanged { level: clamped }],
        effects: Vec::new(),
    }
}

fn handle_start_self_test(
    snapshot: &mut PumpSnapshot,
    jobs: &mut Jobs<PumpJobSpec>,
) -> Decision<crate::domain::PumpDomain> {
    if snapshot.self_test_active {
        return Decision::new();
    }
    let timeout = Duration::from_secs(snapshot.settings.self_test_timeout_seconds.max(1) as u64);
    snapshot.self_test_phase = phase::SELF_TEST_RUNNING;
    snapshot.self_test_active = true;
    let root = jobs.enqueue(self_test_job_graph());
    snapshot.self_test_root_job = Some(root);
    Decision {
        events: vec![PumpEvent::SelfTestStarted],
        effects: vec![Effect::Schedule {
            after: timeout,
            timer: SELF_TEST_TIMEOUT_TIMER,
            command: PumpCommand::SelfTestTimedOut,
        }],
    }
}

fn handle_cancel_self_test(
    snapshot: &mut PumpSnapshot,
    jobs: &mut Jobs<PumpJobSpec>,
) -> Decision<crate::domain::PumpDomain> {
    if !snapshot.self_test_active {
        return Decision::new();
    }
    cancel_self_test_jobs(snapshot, jobs);
    snapshot.self_test_phase = phase::SELF_TEST_CANCELLED;
    snapshot.self_test_active = false;
    snapshot.self_test_root_job = None;
    Decision {
        events: vec![PumpEvent::SelfTestCompleted {
            passed: false,
            detail: "cancelled".into(),
        }],
        effects: vec![Effect::CancelSchedule(SELF_TEST_TIMEOUT_TIMER)],
    }
}

fn handle_update_settings(
    snapshot: &mut PumpSnapshot,
    parsed: ExampleSettings,
) -> Decision<crate::domain::PumpDomain> {
    let restart_fields = diff_restart_fields(&snapshot.settings, &parsed);
    snapshot.settings = parsed;
    snapshot.level = snapshot.clamp_level(snapshot.level);
    let mut events = Vec::new();
    if !restart_fields.is_empty() {
        events.push(PumpEvent::RestartRequired {
            fields: restart_fields,
        });
    }
    Decision {
        events,
        effects: vec![Effect::Persist],
    }
}

fn handle_job_io_finished(
    snapshot: &mut PumpSnapshot,
    jobs: &mut Jobs<PumpJobSpec>,
    job_id: JobId,
    succeeded: bool,
) -> Decision<crate::domain::PumpDomain> {
    if jobs.status(job_id) == Some(JobStatus::Cancelling) {
        let _ = jobs.complete(job_id, false);
        return Decision::new();
    }
    let _ = jobs.complete(job_id, succeeded);
    if !snapshot.self_test_active {
        return Decision::new();
    }
    let root = snapshot.self_test_root_job;
    if !succeeded {
        return finish_self_test(snapshot, jobs, false, "step failed");
    }
    if let Some(root_job) = root
        && jobs.status(root_job) == Some(JobStatus::Succeeded)
    {
        return finish_self_test(snapshot, jobs, true, "ok");
    }
    Decision::new()
}

fn handle_self_test_timed_out(
    snapshot: &mut PumpSnapshot,
    jobs: &mut Jobs<PumpJobSpec>,
) -> Decision<crate::domain::PumpDomain> {
    if !snapshot.self_test_active {
        return Decision::new();
    }
    cancel_self_test_jobs(snapshot, jobs);
    finish_self_test(snapshot, jobs, false, "timed out")
}

fn finish_self_test(
    snapshot: &mut PumpSnapshot,
    jobs: &mut Jobs<PumpJobSpec>,
    passed: bool,
    detail: &str,
) -> Decision<crate::domain::PumpDomain> {
    snapshot.self_test_active = false;
    snapshot.self_test_root_job = None;
    snapshot.self_test_phase = if passed {
        phase::SELF_TEST_PASSED
    } else {
        phase::SELF_TEST_FAILED
    };
    let _jobs = jobs;
    Decision {
        events: vec![PumpEvent::SelfTestCompleted {
            passed,
            detail: detail.into(),
        }],
        effects: vec![Effect::CancelSchedule(SELF_TEST_TIMEOUT_TIMER)],
    }
}

fn cancel_self_test_jobs(snapshot: &mut PumpSnapshot, jobs: &mut Jobs<PumpJobSpec>) {
    if let Some(root) = snapshot.self_test_root_job {
        let _ = jobs.cancel(root);
    }
}

fn diff_restart_fields(previous: &ExampleSettings, next: &ExampleSettings) -> Vec<String> {
    let mut fields = Vec::new();
    if previous.device_model != next.device_model {
        fields.push("device_model".into());
    }
    fields
}
