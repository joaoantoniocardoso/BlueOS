//! L1 tests: the Jobs table driven by plain function calls, as the Kernel and a Domain drive it.

use blueos_jobs::{JobControl, JobEnd, JobId, JobNature, JobStatus, Jobs, JobsError, Submitted};

const REPAIR: &str = "RepairRecording";
const GOAL: &[u8] = b"/recordings/a.mcap";
const LASTING: JobNature = JobNature {
    lasting: true,
    cancellable: true,
    pausable: true,
    ..JobNature::INSTANT
};

#[test]
fn a_job_id_is_the_text_of_a_uuid() {
    let text = "0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10";

    let job_id: JobId = text.parse().expect("a UUID");

    assert_eq!(job_id.to_string(), text);
    assert_eq!(
        "0B5E8F5C-6F0A-4C4E-9A52-2F1E7D3C9B10".parse::<JobId>(),
        Ok(job_id)
    );
    for invalid in [
        "",
        "7",
        "0b5e8f5c6f0a4c4e9a522f1e7d3c9b10",
        "0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b1g",
        "0b5e8f5c+6f0a-4c4e-9a52-2f1e7d3c9b10",
    ] {
        assert!(invalid.parse::<JobId>().is_err(), "{invalid} is no UUID");
    }
    assert_eq!(
        JobId::from_u128(1).to_string(),
        "00000000-0000-0000-0000-000000000001"
    );
}

#[test]
fn ending_an_unknown_job_is_rejected() {
    let mut jobs = Jobs::default();
    let job_id = JobId::from_u128(99);
    assert_eq!(
        jobs.end(job_id, JobEnd::Succeeded),
        Err(JobsError::Unknown(job_id))
    );
}

#[test]
fn a_submitted_job_executes_until_its_domain_ends_it() {
    let mut jobs = Jobs::default();
    let job_id = JobId::from_u128(1);

    assert_eq!(
        jobs.submit(job_id, REPAIR, GOAL, LASTING),
        Ok(Submitted::New)
    );
    assert_eq!(status(&jobs, job_id), JobStatus::Executing);
    jobs.end(job_id, JobEnd::Succeeded).expect("active");

    assert_eq!(status(&jobs, job_id), JobStatus::Succeeded);
    assert_eq!(
        jobs.end(
            job_id,
            JobEnd::Aborted {
                reason: "late".to_owned()
            }
        ),
        Err(JobsError::AlreadyEnded(job_id))
    );
}

#[test]
fn the_same_id_with_the_same_goal_is_a_retry_and_with_another_goal_is_rejected() {
    let mut jobs = Jobs::default();
    let job_id = JobId::from_u128(1);
    jobs.submit(job_id, REPAIR, GOAL, LASTING).expect("new");

    assert_eq!(
        jobs.submit(job_id, REPAIR, GOAL, LASTING),
        Ok(Submitted::Retry)
    );
    let reused = jobs.submit(job_id, REPAIR, b"/recordings/b.mcap", LASTING);
    assert_eq!(reused, Err(JobsError::IdReused(job_id)));
    assert_eq!(reused.unwrap_err().to_string(), "id reused");
    assert_eq!(
        jobs.submit(job_id, "DeleteRecording", GOAL, LASTING),
        Err(JobsError::IdReused(job_id))
    );
    jobs.end(job_id, JobEnd::Succeeded).expect("active");
    assert_eq!(
        jobs.submit(job_id, REPAIR, GOAL, LASTING),
        Ok(Submitted::Retry)
    );
}

#[test]
fn an_id_is_free_again_once_its_job_leaves_the_history() {
    let mut jobs = Jobs::with_retention(1);
    let [first, second] = [JobId::from_u128(1), JobId::from_u128(2)];
    for job_id in [first, second] {
        jobs.submit(job_id, REPAIR, GOAL, LASTING).expect("new");
        jobs.end(job_id, JobEnd::Succeeded).expect("active");
    }

    assert_eq!(jobs.job(first), None);
    assert_eq!(
        jobs.submit(first, REPAIR, b"another goal", LASTING),
        Ok(Submitted::New)
    );
}

#[test]
fn cancel_pause_and_resume_change_the_status_the_domain_follows() {
    let mut jobs = Jobs::default();
    let job_id = JobId::from_u128(1);
    jobs.submit(job_id, REPAIR, GOAL, LASTING).expect("new");

    assert_eq!(
        jobs.control(job_id, JobControl::Pause),
        Ok(JobStatus::Paused)
    );
    assert_eq!(
        jobs.control(job_id, JobControl::Pause),
        Ok(JobStatus::Paused)
    );
    assert_eq!(
        jobs.control(job_id, JobControl::Resume),
        Ok(JobStatus::Executing)
    );
    assert_eq!(
        jobs.control(job_id, JobControl::Cancel),
        Ok(JobStatus::Canceling)
    );
    assert_eq!(
        jobs.control(job_id, JobControl::Pause),
        Err(JobsError::Refused {
            job_id,
            control: JobControl::Pause,
            status: JobStatus::Canceling,
        })
    );
    jobs.end(job_id, JobEnd::Canceled).expect("active");
    assert_eq!(status(&jobs, job_id), JobStatus::Canceled);
    assert_eq!(
        jobs.control(job_id, JobControl::Cancel),
        Err(JobsError::Refused {
            job_id,
            control: JobControl::Cancel,
            status: JobStatus::Canceled,
        })
    );
}

#[test]
fn a_control_the_nature_does_not_allow_is_rejected() {
    let mut jobs = Jobs::default();
    let job_id = JobId::from_u128(1);
    let nature = JobNature {
        lasting: true,
        ..JobNature::INSTANT
    };
    jobs.submit(job_id, REPAIR, GOAL, nature).expect("new");

    for control in [
        JobControl::Cancel,
        JobControl::Pause,
        JobControl::Resume,
        JobControl::AnswerPermission { granted: true },
    ] {
        let rejected = jobs.control(job_id, control);
        assert_eq!(
            rejected,
            Err(JobsError::NotAllowed {
                job_type: REPAIR.to_owned(),
                control,
            })
        );
        assert_eq!(
            rejected.unwrap_err().to_string(),
            format!("{REPAIR} does not allow {control}")
        );
    }
    assert_eq!(status(&jobs, job_id), JobStatus::Executing);
    assert_eq!(
        jobs.control(JobId::from_u128(2), JobControl::Cancel),
        Err(JobsError::Unknown(JobId::from_u128(2)))
    );
}

#[test]
fn a_job_that_needs_permission_waits_for_an_answer() {
    let mut jobs = Jobs::default();
    let [granted, denied, canceled] = [1, 2, 3].map(JobId::from_u128);
    let nature = JobNature {
        needs_permission: true,
        ..LASTING
    };
    for job_id in [granted, denied, canceled] {
        jobs.submit(job_id, REPAIR, GOAL, nature).expect("new");
        assert_eq!(status(&jobs, job_id), JobStatus::WaitingForPermission);
    }

    assert_eq!(
        jobs.control(granted, JobControl::AnswerPermission { granted: true }),
        Ok(JobStatus::Executing)
    );
    assert_eq!(
        jobs.control(granted, JobControl::AnswerPermission { granted: true }),
        Err(JobsError::Refused {
            job_id: granted,
            control: JobControl::AnswerPermission { granted: true },
            status: JobStatus::Executing,
        })
    );
    assert_eq!(
        jobs.control(denied, JobControl::AnswerPermission { granted: false }),
        Ok(JobStatus::Canceled)
    );
    assert_eq!(reason(&jobs, denied), "permission denied");
    assert_eq!(
        jobs.control(canceled, JobControl::Cancel),
        Ok(JobStatus::Canceled)
    );
}

#[test]
fn a_restore_aborts_the_jobs_that_were_running_and_keeps_the_waiting_ones() {
    let mut jobs = Jobs::default();
    let [executing, paused, waiting] = [1, 2, 3].map(JobId::from_u128);
    jobs.submit(executing, REPAIR, GOAL, LASTING).expect("new");
    jobs.submit(paused, REPAIR, GOAL, LASTING).expect("new");
    jobs.control(paused, JobControl::Pause).expect("pausable");
    let needs_permission = JobNature {
        needs_permission: true,
        ..LASTING
    };
    jobs.submit(waiting, REPAIR, GOAL, needs_permission)
        .expect("new");

    jobs.interrupt();

    for job_id in [executing, paused] {
        assert_eq!(status(&jobs, job_id), JobStatus::Aborted);
        assert_eq!(reason(&jobs, job_id), "interrupted");
    }
    assert_eq!(status(&jobs, waiting), JobStatus::WaitingForPermission);
}

#[test]
fn the_list_shows_active_jobs_then_the_retained_finished_ones() {
    let mut jobs = Jobs::with_retention(1);
    let [first, second, third, fourth] = [1, 2, 3, 4].map(JobId::from_u128);
    for job_id in [first, second, third, fourth] {
        jobs.submit(job_id, REPAIR, GOAL, LASTING).expect("new");
    }
    jobs.end(
        second,
        JobEnd::Aborted {
            reason: "disk full".to_owned(),
        },
    )
    .expect("active");
    jobs.end(first, JobEnd::Succeeded).expect("active");

    let listed: Vec<_> = jobs
        .list()
        .map(|job| (job.job_id, job.status, job.reason.as_str()))
        .collect();

    assert_eq!(
        listed,
        [
            (third, JobStatus::Executing, ""),
            (fourth, JobStatus::Executing, ""),
            (first, JobStatus::Succeeded, ""),
        ]
    );
}

#[test]
fn the_history_keeps_the_last_ended_jobs_of_each_type() {
    let mut jobs = Jobs::with_retention(1);
    let [first, second, snapshot] = [1, 2, 3].map(JobId::from_u128);
    for (job_id, job_type) in [
        (snapshot, "SnapshotRecording"),
        (first, REPAIR),
        (second, REPAIR),
    ] {
        jobs.submit(job_id, job_type, GOAL, LASTING).expect("new");
        jobs.end(job_id, JobEnd::Succeeded).expect("active");
    }

    let listed: Vec<_> = jobs.list().map(|job| job.job_id).collect();

    assert_eq!(listed, [snapshot, second]);
}

fn status(jobs: &Jobs, job_id: JobId) -> JobStatus {
    jobs.job(job_id).expect("the Job is in the table").status
}

fn reason(jobs: &Jobs, job_id: JobId) -> &str {
    &jobs.job(job_id).expect("the Job is in the table").reason
}
