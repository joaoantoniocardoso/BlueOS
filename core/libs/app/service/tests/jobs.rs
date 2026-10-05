//! The Jobs surface every Service gets from the Kernel: submit, the ack, the controls, the `jobs` State, and the
//! Feedback, Job result and history of each Job type, through a real `build` (layer L3).

mod jobs_fixture;

use core::time::Duration;

use blueos_api::{
    CommandAck, Message, cdr_encoding, command_key, info_query_key, jobs_key, query_key,
    status_state_key,
};
use blueos_comms::QueryBody;
use blueos_idl::msg::{
    blueos_example_msgs::SetLevelGoal,
    blueos_msgs::{CommandAckStatus, JobList, JobStatusStatus, SettingsEnvelope},
    std_msgs::Empty,
};
use blueos_jobs::{JobControl, JobId};
use blueos_service::Service;
use bytes::Bytes;

use jobs_fixture::BrewerService;
use jobs_fixture::helpers::*;

#[tokio::test(start_paused = true)]
async fn a_submit_acks_the_client_job_id_and_the_job_executes() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);

    let ack = harness.submit("Brew", job_id, &cups(2)).await.unwrap();

    assert_eq!(ack, accepted(job_id, CommandAckStatus::Executing));
    assert_eq!(
        listed(&harness).await,
        [entry(job_id, "Brew", JobStatusStatus::Executing, "")]
    );
}

#[tokio::test(start_paused = true)]
async fn a_job_that_ends_in_the_step_that_accepts_it_acks_its_final_status() {
    let harness = start().await;
    let [ping, empty] = [1, 2].map(JobId::from_u128);

    let instant = harness
        .submit("Ping", ping, &Empty::default())
        .await
        .unwrap();
    let aborted = harness.submit("Brew", empty, &cups(0)).await.unwrap();

    assert_eq!(instant, accepted(ping, CommandAckStatus::Succeeded));
    assert_eq!(
        aborted,
        CommandAck {
            reason: "no cups to brew".to_owned(),
            ..accepted(empty, CommandAckStatus::Aborted)
        }
    );
    assert_eq!(
        listed(&harness).await,
        [
            entry(ping, "Ping", JobStatusStatus::Succeeded, ""),
            entry(empty, "Brew", JobStatusStatus::Aborted, "no cups to brew"),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn a_service_without_settings_refuses_update_settings() {
    let harness = start().await;
    let job_id = JobId::from_u128(3);

    let ack = harness
        .submit("UpdateSettings", job_id, &SettingsEnvelope::default())
        .await
        .unwrap();

    assert_eq!(
        ack,
        rejected(
            job_id,
            CommandAckStatus::StatusUnknown,
            "the Service has no settings"
        )
    );
    assert!(listed(&harness).await.is_empty());
}

#[tokio::test(start_paused = true)]
async fn the_same_id_and_goal_is_a_retry_and_runs_the_job_once() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);
    harness.submit("Brew", job_id, &cups(2)).await.unwrap();

    let retry = harness.submit("Brew", job_id, &cups(2)).await.unwrap();

    assert_eq!(retry, accepted(job_id, CommandAckStatus::Executing));
    assert_eq!(listed(&harness).await.len(), 1);
}

#[tokio::test(start_paused = true)]
async fn the_same_id_with_another_goal_is_rejected_as_reused() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);
    harness.submit("Brew", job_id, &cups(2)).await.unwrap();

    let reused = harness.submit("Brew", job_id, &cups(3)).await.unwrap();
    let other_type = harness.submit("Steep", job_id, &cups(2)).await.unwrap();

    for ack in [reused, other_type] {
        assert_eq!(
            ack,
            rejected(job_id, CommandAckStatus::StatusUnknown, "id reused")
        );
    }
    assert_eq!(
        listed(&harness).await,
        [entry(job_id, "Brew", JobStatusStatus::Executing, "")]
    );
}

#[tokio::test(start_paused = true)]
async fn a_rejected_submit_leaves_no_job() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);

    let refused = harness
        .submit("Refuse", job_id, &Empty::default())
        .await
        .unwrap();

    assert_eq!(
        refused,
        rejected(job_id, CommandAckStatus::StatusUnknown, "refused")
    );
    assert_eq!(harness.jobs().await.unwrap(), JobList::default());
}

#[tokio::test(start_paused = true)]
async fn a_command_without_a_job_id_is_rejected() {
    let harness = start().await;

    let mut acks = Vec::new();
    for attachment in [None, Some("not a uuid")] {
        let mut body = QueryBody::new(
            cups(2).encode().unwrap(),
            cdr_encoding(SetLevelGoal::SCHEMA_NAME),
        );
        if let Some(attachment) = attachment {
            body = body.with_attachment(Bytes::from_static(attachment.as_bytes()));
        }
        acks.push(get_ack(&harness, &command_key(BrewerService::NAME, "Brew"), body).await);
    }

    for ack in acks {
        assert!(!ack.accepted);
        assert_eq!(ack.job_id, "");
        assert_eq!(ack.reason, "the Command's attachment is not a Job id");
    }
    assert_eq!(harness.jobs().await.unwrap(), JobList::default());
}

#[tokio::test(start_paused = true)]
async fn cancel_pause_and_resume_drive_a_job_through_its_lifecycle() {
    let harness = start().await;
    let mut jobs = subscribe(&harness).await;
    let job_id = JobId::from_u128(7);
    harness.submit("Brew", job_id, &cups(2)).await.unwrap();
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Executing);

    let paused = harness.control(job_id, JobControl::Pause).await.unwrap();
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Paused);
    let resumed = harness.control(job_id, JobControl::Resume).await.unwrap();
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Executing);
    let canceled = harness.control(job_id, JobControl::Cancel).await.unwrap();
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Canceling);

    assert_eq!(paused, accepted(job_id, CommandAckStatus::Paused));
    assert_eq!(resumed, accepted(job_id, CommandAckStatus::Executing));
    assert_eq!(canceled, accepted(job_id, CommandAckStatus::Canceling));
    assert_eq!(
        status(&next(&mut jobs).await),
        JobStatusStatus::Canceled,
        "the brew ends Canceled when its time is up"
    );
}

#[tokio::test(start_paused = true)]
async fn a_lasting_job_succeeds_when_its_domain_ends_it() {
    let harness = start().await;
    let mut jobs = subscribe(&harness).await;
    harness
        .submit("Brew", JobId::from_u128(7), &cups(2))
        .await
        .unwrap();
    next::<JobList>(&mut jobs).await;

    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Succeeded);
}

#[tokio::test(start_paused = true)]
async fn a_control_the_job_type_does_not_allow_is_rejected() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);
    harness.submit("Steep", job_id, &cups(2)).await.unwrap();

    for control in [
        JobControl::Cancel,
        JobControl::Pause,
        JobControl::Resume,
        JobControl::AnswerPermission { granted: true },
    ] {
        assert_eq!(
            harness.control(job_id, control).await.unwrap(),
            rejected(
                job_id,
                CommandAckStatus::Executing,
                &format!("Steep does not allow {control}")
            )
        );
    }
    assert_eq!(
        listed(&harness).await,
        [entry(job_id, "Steep", JobStatusStatus::Executing, "")]
    );
}

#[tokio::test(start_paused = true)]
async fn a_control_that_does_not_apply_to_the_status_is_rejected() {
    let harness = start().await;
    let [brewing, unknown] = [7, 8].map(JobId::from_u128);
    harness.submit("Brew", brewing, &cups(2)).await.unwrap();

    let resume = harness.control(brewing, JobControl::Resume).await.unwrap();
    harness.control(brewing, JobControl::Cancel).await.unwrap();
    let pause = harness.control(brewing, JobControl::Pause).await.unwrap();
    let missing = harness.control(unknown, JobControl::Cancel).await.unwrap();

    assert_eq!(resume, accepted(brewing, CommandAckStatus::Executing));
    assert_eq!(
        pause,
        rejected(
            brewing,
            CommandAckStatus::Canceling,
            &format!("PauseJob does not apply to the Job {brewing}, which is Canceling")
        )
    );
    assert_eq!(
        missing,
        rejected(
            unknown,
            CommandAckStatus::StatusUnknown,
            &format!("there is no Job {unknown}")
        )
    );
}

#[tokio::test(start_paused = true)]
async fn a_job_waits_for_permission_and_executes_once_it_is_granted() {
    let harness = start().await;
    let mut jobs = subscribe(&harness).await;
    let job_id = JobId::from_u128(7);

    let submitted = harness.submit("Pour", job_id, &cups(2)).await.unwrap();
    assert_eq!(
        status(&next(&mut jobs).await),
        JobStatusStatus::WaitingForPermission
    );
    let granted = harness
        .control(job_id, JobControl::AnswerPermission { granted: true })
        .await
        .unwrap();
    assert_eq!(status(&next(&mut jobs).await), JobStatusStatus::Executing);

    assert_eq!(
        submitted,
        accepted(job_id, CommandAckStatus::WaitingForPermission)
    );
    assert_eq!(granted, accepted(job_id, CommandAckStatus::Executing));
    assert_eq!(
        status(&next(&mut jobs).await),
        JobStatusStatus::Succeeded,
        "the Domain received the Goal once permission was granted"
    );
}

#[tokio::test(start_paused = true)]
async fn a_denied_permission_cancels_the_job() {
    let harness = start().await;
    let job_id = JobId::from_u128(7);
    harness.submit("Pour", job_id, &cups(2)).await.unwrap();

    let denied = harness
        .control(job_id, JobControl::AnswerPermission { granted: false })
        .await
        .unwrap();

    assert_eq!(
        denied,
        CommandAck {
            reason: "permission denied".to_owned(),
            ..accepted(job_id, CommandAckStatus::Canceled)
        }
    );
    assert_eq!(
        listed(&harness).await,
        [entry(
            job_id,
            "Pour",
            JobStatusStatus::Canceled,
            "permission denied"
        )]
    );
}

#[tokio::test(start_paused = true)]
async fn no_key_outside_command_changes_anything() {
    let harness = start().await;
    let goal = cups(2).encode().unwrap();
    let service = BrewerService::NAME;
    let keys = [
        jobs_key(service),
        info_query_key(service),
        status_state_key(service),
        query_key(service, "Brew"),
        format!("blueos/v1/{service}/Brew"),
        format!("blueos/v1/{service}/CancelJob"),
    ];

    for key in &keys {
        let body = QueryBody::new(goal.clone(), cdr_encoding(SetLevelGoal::SCHEMA_NAME))
            .with_attachment(Bytes::from(JobId::from_u128(7).to_string()));
        drop(
            harness
                .backend()
                .get(key, Some(body), Duration::from_secs(1))
                .await,
        );
    }

    assert_eq!(harness.jobs().await.unwrap(), JobList::default());
}
