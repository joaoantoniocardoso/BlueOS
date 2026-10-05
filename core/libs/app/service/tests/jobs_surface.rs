//! Jobs State, feedback, results, history, and `info` endpoints.

mod jobs_fixture;

use blueos_api::{job_feedback_key, job_result_key};
use blueos_idl::msg::{
    blueos_msgs::{JobResult, JobStatusStatus},
    std_msgs::Empty,
};
use blueos_jobs::{JobControl, JobId};
use blueos_service::Service;
use tokio::time::advance;

use jobs_fixture::helpers::*;
use jobs_fixture::{BREW_TIME, BrewerService, RETENTION};

#[tokio::test(start_paused = true)]
async fn info_reports_the_version_and_build_label_of_the_service_its_build_never_passes() {
    let harness = start().await;

    let info = harness.info().await.unwrap();

    assert_eq!(
        (
            info.name.as_str(),
            info.version.as_str(),
            info.build.as_str()
        ),
        (
            BrewerService::NAME,
            BrewerService::VERSION,
            BrewerService::BUILD
        )
    );
}

#[tokio::test(start_paused = true)]
async fn info_lists_the_jobs_state_and_the_feedback_result_and_history_of_each_job_type() {
    let harness = start().await;
    let info = harness.info().await.unwrap();
    let rows = info_endpoint_rows(&info);
    assert_eq!(
        rows.len(),
        1 + 1 + 1 + 3 * 6,
        "UpdateSettings, jobs, metrics, then three per Job type: {rows:?}"
    );
    let shown: Vec<_> = rows
        .into_iter()
        .filter(|(_, name, ..)| {
            name == "UpdateSettings"
                || name == "jobs"
                || name.starts_with("jobs/Brew/")
                || name.starts_with("jobs/UpdateSettings/")
        })
        .collect();
    assert_eq!(shown, expected_brewer_job_info_rows(BrewerService::NAME));
}

#[tokio::test(start_paused = true)]
async fn the_feedback_state_holds_the_latest_feedback_of_a_job_until_it_ends() {
    let harness = start().await;
    let mut feedback = harness
        .backend()
        .subscribe(&job_feedback_key(BrewerService::NAME, "Brew"))
        .await
        .unwrap();
    let job_id = JobId::from_u128(7);
    harness.submit("Brew", job_id, &cups(3)).await.unwrap();

    let first = next(&mut feedback).await;
    let second = next(&mut feedback).await;
    let latest = harness.job_feedback("Brew").await.unwrap();
    let ended = next(&mut feedback).await;

    assert_eq!(fed_back(first), [(job_id, poured(1, 3))]);
    assert_eq!(fed_back(second), [(job_id, poured(2, 3))]);
    assert_eq!(fed_back(latest), [(job_id, poured(2, 3))]);
    assert_eq!(fed_back(ended), []);
    assert_eq!(
        listed(&harness).await,
        [entry(job_id, "Brew", JobStatusStatus::Succeeded, "")]
    );
}

#[tokio::test(start_paused = true)]
async fn a_client_that_opens_the_feedback_state_mid_job_sees_the_latest_feedback() {
    let harness = start().await;
    let mut published = harness
        .backend()
        .subscribe(&job_feedback_key(BrewerService::NAME, "Brew"))
        .await
        .unwrap();
    let [brewing, waiting] = [7, 8].map(JobId::from_u128);
    let two_cups_each = [(brewing, poured(2, 3)), (waiting, poured(2, 3))];
    harness.submit("Brew", brewing, &cups(3)).await.unwrap();
    harness.submit("Brew", waiting, &cups(3)).await.unwrap();
    while fed_back(next(&mut published).await) != two_cups_each {}
    advance(BREW_TIME / 2).await;

    let feedback = harness.job_feedback("Brew").await.unwrap();

    assert_eq!(fed_back(feedback), two_cups_each);
}

#[tokio::test(start_paused = true)]
async fn the_result_event_carries_how_a_job_ended_and_its_job_result() {
    let harness = start().await;
    let mut results = harness
        .backend()
        .subscribe(&job_result_key(BrewerService::NAME, "Brew"))
        .await
        .unwrap();
    let [aborted, succeeded, canceled] = [1, 2, 3].map(JobId::from_u128);

    harness.submit("Brew", aborted, &cups(0)).await.unwrap();
    let abort = next(&mut results).await;
    harness.submit("Brew", succeeded, &cups(2)).await.unwrap();
    let success = next(&mut results).await;
    harness.submit("Brew", canceled, &cups(2)).await.unwrap();
    harness.control(canceled, JobControl::Cancel).await.unwrap();
    let cancel = next(&mut results).await;

    assert_eq!(
        ended(abort),
        (
            entry(aborted, "Brew", JobStatusStatus::Aborted, "no cups to brew"),
            cups(0)
        )
    );
    assert_eq!(
        ended(success),
        (
            entry(succeeded, "Brew", JobStatusStatus::Succeeded, ""),
            cups(2)
        )
    );
    assert_eq!(
        ended(cancel),
        (
            entry(canceled, "Brew", JobStatusStatus::Canceled, ""),
            cups(0)
        )
    );
}

#[tokio::test(start_paused = true)]
async fn a_job_the_kernel_ends_publishes_its_result_too() {
    let harness = start().await;
    let mut results = harness
        .backend()
        .subscribe(&job_result_key(BrewerService::NAME, "Pour"))
        .await
        .unwrap();
    let job_id = JobId::from_u128(7);
    harness.submit("Pour", job_id, &cups(2)).await.unwrap();

    harness
        .control(job_id, JobControl::AnswerPermission { granted: false })
        .await
        .unwrap();
    let result: JobResult = next(&mut results).await;

    assert_eq!(
        row(result.job),
        entry(
            job_id,
            "Pour",
            JobStatusStatus::Canceled,
            "permission denied"
        )
    );
    assert_eq!(
        result.result,
        Vec::<u8>::new(),
        "Pour declares no Job result"
    );
}

#[tokio::test(start_paused = true)]
async fn the_history_query_returns_the_last_finished_jobs_of_a_type_in_order() {
    let harness = start().await;
    let [first, second, third, brew] = [1, 2, 3, 4].map(JobId::from_u128);
    for ping in [first, second, third] {
        harness
            .submit("Ping", ping, &Empty::default())
            .await
            .unwrap();
    }
    harness.submit("Brew", brew, &cups(0)).await.unwrap();

    let pings = harness.job_history("Ping").await.unwrap();
    let brews = harness.job_history("Brew").await.unwrap();

    assert_eq!(
        pings.jobs.into_iter().map(row).collect::<Vec<_>>(),
        [
            entry(second, "Ping", JobStatusStatus::Succeeded, ""),
            entry(third, "Ping", JobStatusStatus::Succeeded, ""),
        ],
        "the last {RETENTION} Pings, oldest first"
    );
    assert_eq!(
        brews.jobs.into_iter().map(row).collect::<Vec<_>>(),
        [entry(
            brew,
            "Brew",
            JobStatusStatus::Aborted,
            "no cups to brew"
        )]
    );
}
