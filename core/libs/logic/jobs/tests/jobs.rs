//! L1 tests: Job graphs driven by plain function calls, as a Domain drives them from `handle`.

use blueos_jobs::{JobEnd, JobGraph, JobId, JobKind, JobStatus, JobView, Jobs, JobsError, LeafJob};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    Fill,
    Heat,
    Drain,
}

#[test]
fn a_sequence_runs_its_steps_in_order() {
    let mut jobs = Jobs::default();

    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Leaf(Step::Heat),
    ]));

    assert_eq!(started.job_id.get(), 1);
    assert_eq!(steps(&started.leaves), [Step::Fill]);
    assert_eq!(jobs.status(started.job_id), Some(JobStatus::Running));
    let heat = jobs
        .finish(started.leaves[0].job_id, JobEnd::Succeeded)
        .expect("Fill is running");
    assert_eq!(steps(&heat), [Step::Heat]);
    assert_eq!(jobs.status(started.job_id), Some(JobStatus::Running));
    let nothing = jobs
        .finish(heat[0].job_id, JobEnd::Succeeded)
        .expect("Heat is running");
    assert!(nothing.is_empty());
    assert_eq!(
        jobs.status(started.job_id),
        Some(JobStatus::Finished(JobEnd::Succeeded))
    );
}

#[test]
fn a_parallel_runs_every_step_at_once_and_joins_them_all() {
    let mut jobs = Jobs::default();

    let started = jobs.start(JobGraph::Parallel(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Leaf(Step::Drain),
    ]));

    assert_eq!(steps(&started.leaves), [Step::Fill, Step::Drain]);
    let [fill, drain] = [started.leaves[0].job_id, started.leaves[1].job_id];
    let nothing = jobs
        .finish(drain, JobEnd::Failed)
        .expect("Drain is running");
    assert!(nothing.is_empty());
    assert_eq!(jobs.status(started.job_id), Some(JobStatus::Running));
    jobs.finish(fill, JobEnd::Succeeded)
        .expect("Fill is running");
    assert_eq!(
        jobs.status(started.job_id),
        Some(JobStatus::Finished(JobEnd::Failed))
    );
}

#[test]
fn a_sequence_after_a_parallel_starts_once_the_parallel_has_joined() {
    let mut jobs = Jobs::default();

    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Parallel(vec![JobGraph::Leaf(Step::Fill), JobGraph::Leaf(Step::Heat)]),
        JobGraph::Leaf(Step::Drain),
    ]));

    assert_eq!(steps(&started.leaves), [Step::Fill, Step::Heat]);
    let after_fill = jobs
        .finish(started.leaves[0].job_id, JobEnd::Succeeded)
        .expect("Fill is running");
    assert!(after_fill.is_empty());
    let after_heat = jobs
        .finish(started.leaves[1].job_id, JobEnd::Succeeded)
        .expect("Heat is running");
    assert_eq!(steps(&after_heat), [Step::Drain]);
}

#[test]
fn a_failed_step_ends_its_sequence_and_cancels_the_steps_after_it() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Leaf(Step::Heat),
    ]));

    let nothing = jobs
        .finish(started.leaves[0].job_id, JobEnd::Failed)
        .expect("Fill is running");

    assert!(nothing.is_empty());
    assert_eq!(
        jobs.status(started.job_id),
        Some(JobStatus::Finished(JobEnd::Failed))
    );
    assert_eq!(
        leaf(&jobs, Step::Heat).status,
        JobStatus::Finished(JobEnd::Cancelled)
    );
}

#[test]
fn cancelling_a_root_job_cancels_its_running_leaves() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Parallel(vec![JobGraph::Leaf(Step::Fill), JobGraph::Leaf(Step::Heat)]),
        JobGraph::Leaf(Step::Drain),
    ]));

    let cancelling = jobs.cancel(started.job_id).expect("the root is running");

    assert_eq!(cancelling, started.leaves);
    assert_eq!(jobs.status(started.job_id), Some(JobStatus::Cancelling));
    assert_eq!(leaf(&jobs, Step::Fill).status, JobStatus::Cancelling);
    assert_eq!(
        leaf(&jobs, Step::Drain).status,
        JobStatus::Finished(JobEnd::Cancelled)
    );
    jobs.finish(cancelling[0].job_id, JobEnd::Cancelled)
        .expect("Fill is cancelling");
    let nothing = jobs
        .finish(cancelling[1].job_id, JobEnd::Succeeded)
        .expect("Heat is cancelling");
    assert!(nothing.is_empty());
    assert_eq!(
        leaf(&jobs, Step::Heat).status,
        JobStatus::Finished(JobEnd::Succeeded)
    );
    assert_eq!(
        jobs.status(started.job_id),
        Some(JobStatus::Finished(JobEnd::Cancelled))
    );
}

#[test]
fn a_cancelled_step_never_starts() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Leaf(Step::Drain),
    ]));
    let drain = leaf(&jobs, Step::Drain).job_id;

    let cancelling = jobs.cancel(drain).expect("Drain is queued");

    assert!(cancelling.is_empty());
    let nothing = jobs
        .finish(started.leaves[0].job_id, JobEnd::Succeeded)
        .expect("Fill is running");
    assert!(nothing.is_empty());
    assert_eq!(
        jobs.status(started.job_id),
        Some(JobStatus::Finished(JobEnd::Cancelled))
    );
}

#[test]
fn cancelling_twice_stops_nothing_more_and_a_finished_job_cannot_be_cancelled() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Leaf(Step::Fill));

    jobs.cancel(started.job_id).expect("Fill is running");
    let again = jobs.cancel(started.job_id).expect("Fill is cancelling");
    jobs.finish(started.job_id, JobEnd::Cancelled)
        .expect("Fill is cancelling");

    assert!(again.is_empty());
    assert_eq!(
        jobs.cancel(started.job_id),
        Err(JobsError::Finished(started.job_id))
    );
}

#[test]
fn only_a_running_leaf_can_finish() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Leaf(Step::Drain),
    ]));
    let fill = started.leaves[0].job_id;
    let drain = leaf(&jobs, Step::Drain).job_id;
    let unknown = JobId::new(1000).expect("1000 is not 0");
    let before = jobs.clone();

    assert_eq!(
        jobs.finish(started.job_id, JobEnd::Succeeded),
        Err(JobsError::NotRunning(started.job_id))
    );
    assert_eq!(
        jobs.finish(drain, JobEnd::Succeeded),
        Err(JobsError::NotRunning(drain))
    );
    assert_eq!(
        jobs.finish(unknown, JobEnd::Succeeded),
        Err(JobsError::Unknown(unknown))
    );
    assert_eq!(jobs.cancel(unknown), Err(JobsError::Unknown(unknown)));
    assert_eq!(jobs, before);
    jobs.finish(fill, JobEnd::Failed).expect("Fill is running");
    assert_eq!(
        jobs.finish(fill, JobEnd::Succeeded),
        Err(JobsError::NotRunning(fill))
    );
}

#[test]
fn a_job_id_is_never_0() {
    assert_eq!(JobId::new(0), None);
    assert_eq!(JobId::new(7).map(JobId::get), Some(7));
}

#[test]
fn finished_root_jobs_beyond_the_retention_count_are_dropped_oldest_finished_first() {
    let mut jobs = Jobs::with_retention(2);
    let long = jobs.start(JobGraph::Leaf(Step::Heat)).job_id;
    let quick: Vec<JobId> = (0..3)
        .map(|_| {
            let job_id = jobs.start(JobGraph::Leaf(Step::Fill)).job_id;
            jobs.finish(job_id, JobEnd::Succeeded)
                .expect("Fill is running");
            job_id
        })
        .collect();

    assert_eq!(roots(&jobs), [long, quick[1], quick[2]]);
    assert_eq!(jobs.status(quick[0]), None);
    assert_eq!(jobs.cancel(quick[0]), Err(JobsError::Unknown(quick[0])));
    jobs.finish(long, JobEnd::Failed).expect("Heat is running");
    assert_eq!(roots(&jobs), [quick[2], long]);
}

#[test]
fn a_root_job_that_finishes_as_it_starts_is_kept_in_the_history() {
    let mut jobs = Jobs::with_retention(1);

    let empty = jobs.start(JobGraph::Parallel(vec![]));

    assert!(empty.leaves.is_empty());
    assert_eq!(
        jobs.status(empty.job_id),
        Some(JobStatus::Finished(JobEnd::Succeeded))
    );
    assert_eq!(roots(&jobs), [empty.job_id]);
}

#[test]
fn with_a_retention_of_0_a_cancelled_root_job_is_listed_until_its_running_step_ends() {
    let mut jobs = Jobs::with_retention(0);
    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Leaf(Step::Drain),
    ]));

    jobs.cancel(started.job_id).expect("the root is running");
    assert_eq!(roots(&jobs), [started.job_id]);
    jobs.finish(started.leaves[0].job_id, JobEnd::Cancelled)
        .expect("Fill is cancelling");

    assert!(roots(&jobs).is_empty());
}

#[test]
fn a_finished_step_of_a_running_job_cannot_be_cancelled() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Leaf(Step::Drain),
    ]));
    let fill = started.leaves[0].job_id;
    jobs.finish(fill, JobEnd::Succeeded)
        .expect("Fill is running");

    assert_eq!(jobs.cancel(fill), Err(JobsError::Finished(fill)));
}

#[test]
fn the_list_shows_each_root_job_followed_by_the_jobs_in_it() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Sequence(vec![
        JobGraph::Leaf(Step::Fill),
        JobGraph::Parallel(vec![
            JobGraph::Leaf(Step::Heat),
            JobGraph::Leaf(Step::Drain),
        ]),
    ]));

    let views = jobs.list();

    let [root, fill, parallel, heat, drain] = views.as_slice() else {
        panic!("expected five Jobs, got {views:?}");
    };
    assert_eq!(
        *root,
        JobView {
            job_id: started.job_id,
            parent: None,
            status: JobStatus::Running,
            kind: JobKind::Sequence,
        }
    );
    assert_eq!(
        *fill,
        JobView {
            job_id: started.leaves[0].job_id,
            parent: Some(started.job_id),
            status: JobStatus::Running,
            kind: JobKind::Leaf(&Step::Fill),
        }
    );
    assert_eq!(
        (parallel.parent, parallel.status, parallel.kind),
        (Some(started.job_id), JobStatus::Queued, JobKind::Parallel)
    );
    assert_eq!(
        (heat.parent, heat.status, heat.kind),
        (
            Some(parallel.job_id),
            JobStatus::Queued,
            JobKind::Leaf(&Step::Heat)
        )
    );
    assert_eq!(
        (drain.parent, drain.kind),
        (Some(parallel.job_id), JobKind::Leaf(&Step::Drain))
    );
}

#[test]
fn an_interrupted_leaf_can_finish_as_failed_or_cancelled() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Leaf(Step::Fill));
    let leaf_id = started.leaves[0].job_id;
    jobs.interrupt_running_leaves();
    assert_eq!(jobs.status(leaf_id), Some(JobStatus::Interrupted));

    let nothing = jobs
        .finish(leaf_id, JobEnd::Failed)
        .expect("Interrupted is finishable");
    assert!(nothing.is_empty());
    assert_eq!(
        jobs.status(started.job_id),
        Some(JobStatus::Finished(JobEnd::Failed))
    );
}

#[test]
fn retry_turns_an_interrupted_leaf_back_into_a_running_leaf_job() {
    let mut jobs = Jobs::default();
    let started = jobs.start(JobGraph::Leaf(Step::Heat));
    let leaf_id = started.leaves[0].job_id;
    jobs.interrupt_running_leaves();

    let leaf = jobs.retry(leaf_id).expect("Heat is interrupted");
    assert_eq!(leaf.job_id, leaf_id);
    assert_eq!(leaf.step, Step::Heat);
    assert_eq!(jobs.status(leaf_id), Some(JobStatus::Running));
}

#[test]
fn the_latest_root_job_is_the_one_started_last_even_once_dropped() {
    let mut jobs = Jobs::with_retention(0);
    assert_eq!(jobs.latest_root(), None);

    let first = jobs.start(JobGraph::Leaf(Step::Fill));
    jobs.finish(first.job_id, JobEnd::Succeeded)
        .expect("Fill is running");
    let second = jobs.start(JobGraph::Sequence(vec![]));

    assert_ne!(first.job_id, second.job_id);
    assert_eq!(jobs.latest_root(), Some(second.job_id));
    assert!(roots(&jobs).is_empty());
}

fn roots(jobs: &Jobs<Step>) -> Vec<JobId> {
    jobs.list()
        .into_iter()
        .filter(|job| job.parent.is_none())
        .map(|job| job.job_id)
        .collect()
}

fn steps(leaves: &[LeafJob<Step>]) -> Vec<Step> {
    leaves.iter().map(|leaf| leaf.step).collect()
}

fn leaf(jobs: &Jobs<Step>, step: Step) -> JobView<'_, Step> {
    jobs.list()
        .into_iter()
        .find(|job| job.kind == JobKind::Leaf(&step))
        .expect("a leaf runs the step")
}
