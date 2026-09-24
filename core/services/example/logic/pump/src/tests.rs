use alloc::vec;
use core::time::Duration;

use crate::phase;
use blueos_cqrs::App;
use blueos_cqrs::Effect;
use blueos_jobs::JobId;

use crate::command::PumpCommand;
use crate::domain::PumpDomain;
use crate::event::PumpEvent;
use crate::jobs::PumpJobSpec;
use crate::query::PumpQuery;
use crate::settings::ExampleSettings;
use crate::snapshot::PumpSnapshot;
use crate::timers::SELF_TEST_TIMEOUT_TIMER;

fn test_app() -> App<PumpDomain> {
    App::new(PumpSnapshot::default())
}

#[test]
fn set_level_clamps_to_max_and_emits_event() {
    let mut application = test_app();
    application.snapshot.settings.max_level = 50;
    let decision = application.handle(PumpCommand::SetLevel { level: 80 });
    assert_eq!(decision.events, vec![PumpEvent::LevelChanged { level: 50 }]);
    assert_eq!(application.snapshot.level, 50);
}

#[test]
fn set_level_ignored_during_self_test() {
    let mut application = test_app();
    application.snapshot.self_test_active = true;
    application.snapshot.level = 10;
    let decision = application.handle(PumpCommand::SetLevel { level: 40 });
    assert!(decision.events.is_empty());
    assert_eq!(application.snapshot.level, 10);
}

#[test]
fn start_self_test_schedules_timeout_and_io() {
    let mut application = test_app();
    application.snapshot.settings.self_test_timeout_seconds = 5;
    let decision = application.handle(PumpCommand::StartSelfTest);
    assert_eq!(decision.events, vec![PumpEvent::SelfTestStarted]);
    assert!(application.snapshot.self_test_active);
    assert!(decision.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Schedule {
                after,
                timer: SELF_TEST_TIMEOUT_TIMER,
                command: PumpCommand::SelfTestTimedOut,
            } if *after == Duration::from_secs(5)
        )
    }));
    assert!(
        decision
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::Io(_)))
    );
}

#[test]
fn job_sequence_completes_self_test() {
    let mut application = test_app();
    application.handle(PumpCommand::StartSelfTest);
    for (index, step_name) in ["verify_off", "ramp_up", "verify_on"]
        .into_iter()
        .enumerate()
    {
        let job_id = job_id_for_step(&application, step_name);
        let decision = application.handle(PumpCommand::JobIoFinished {
            job_id,
            succeeded: true,
        });
        if index < 2 {
            assert!(decision.events.is_empty());
        } else {
            assert_eq!(
                decision.events,
                vec![PumpEvent::SelfTestCompleted {
                    passed: true,
                    detail: "ok".into(),
                }]
            );
            assert_eq!(
                application.snapshot.self_test_phase,
                phase::SELF_TEST_PASSED
            );
        }
    }
}

fn job_id_for_step(application: &App<PumpDomain>, step_name: &str) -> JobId {
    application
        .jobs
        .snapshot()
        .jobs
        .into_iter()
        .find(|job| job.job_spec.as_ref().map(PumpJobSpec::step_name) == Some(step_name))
        .map(|job| job.job_id)
        .unwrap_or(JobId(0))
}

#[test]
fn self_test_timeout_fails_and_cancels_timer() {
    let mut application = test_app();
    application.snapshot.settings.self_test_timeout_seconds = 1;
    application.handle(PumpCommand::StartSelfTest);
    let decision = application.handle(PumpCommand::SelfTestTimedOut);
    assert_eq!(
        decision.events,
        vec![PumpEvent::SelfTestCompleted {
            passed: false,
            detail: "timed out".into(),
        }]
    );
    assert_eq!(
        decision.effects,
        vec![Effect::CancelSchedule(SELF_TEST_TIMEOUT_TIMER)]
    );
}

#[test]
fn cancel_self_test_emits_cancelled_event() {
    let mut application = test_app();
    application.handle(PumpCommand::StartSelfTest);
    let decision = application.handle(PumpCommand::CancelSelfTest);
    assert_eq!(
        application.snapshot.self_test_phase,
        phase::SELF_TEST_CANCELLED
    );
    assert_eq!(
        decision.events,
        vec![PumpEvent::SelfTestCompleted {
            passed: false,
            detail: "cancelled".into(),
        }]
    );
}

#[test]
fn update_settings_emits_restart_required_for_device_model() {
    let mut application = test_app();
    let decision = application.handle(PumpCommand::UpdateSettings(ExampleSettings {
        version: 1,
        max_level: 100,
        self_test_timeout_seconds: 30,
        device_model: "other".into(),
    }));
    assert!(
        decision
            .events
            .iter()
            .any(|event| matches!(event, PumpEvent::RestartRequired { .. }))
    );
    assert_eq!(decision.effects, vec![Effect::Persist]);
}

#[test]
fn level_query_returns_clamped_max() {
    let mut application = test_app();
    application.snapshot.settings.max_level = 42;
    application.snapshot.level = 7;
    let view = application.query(PumpQuery::Level);
    assert_eq!(view.level, 7);
    assert_eq!(view.max_level, 42);
}
