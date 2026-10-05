//! Projections, [`CommandSender`] and the reconcile pattern (layer L3).

mod projections_fixture;

use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_domain::Command;
use blueos_idl::msg::{blueos_example_msgs::LevelResponse, std_msgs::Empty};
use blueos_service::testing::{Harness, lock_unpoisoned};

use projections_fixture::{
    ReconcileArguments, ReconcileObserved, ReconcileRequest, ReconcileService, RecordingBackend,
};

#[tokio::test(start_paused = true)]
async fn a_projection_that_does_not_change_is_not_redelivered() {
    let projection_changes = Arc::new(AtomicUsize::new(0));
    let harness = Harness::<ReconcileService>::start(ReconcileArguments {
        projection_changes: Arc::clone(&projection_changes),
    })
    .await
    .expect("the reconcile harness starts");
    harness
        .command_sender()
        .send_awaiting_ack(Command::Request(ReconcileRequest::SetDesired(true)))
        .await
        .expect("the desired lamp turns on");
    let after_first = projection_changes.load(Ordering::SeqCst);
    harness
        .command_sender()
        .send_awaiting_ack(Command::Request(ReconcileRequest::BumpScratch))
        .await
        .expect("the scratch field changes");
    assert_eq!(projection_changes.load(Ordering::SeqCst), after_first);
}

#[tokio::test(start_paused = true)]
async fn send_awaiting_ack_returns_the_domain_verdict() {
    let harness = Harness::<ReconcileService>::start(ReconcileArguments {
        projection_changes: Arc::new(AtomicUsize::new(0)),
    })
    .await
    .expect("the reconcile harness starts");
    let accepted = harness
        .command_sender()
        .send_awaiting_ack(Command::Request(ReconcileRequest::SetDesired(true)))
        .await
        .expect("the Domain accepts the Command");
    assert!(accepted.accepted);
    let observed: LevelResponse = harness
        .query("Observed", &Empty::default())
        .await
        .expect("harness query")
        .expect("the Query answers");
    assert_eq!(observed.level, 1);
}

#[tokio::test(start_paused = true)]
async fn an_observed_fact_handled_twice_leaves_the_same_snapshot() {
    let harness = Harness::<ReconcileService>::start(ReconcileArguments {
        projection_changes: Arc::new(AtomicUsize::new(0)),
    })
    .await
    .expect("the reconcile harness starts");
    let sender = harness.command_sender();
    sender
        .send_awaiting_ack(Command::ObservedFact(ReconcileObserved::Lamp(true)))
        .await
        .expect("the first fact is applied");
    let first: LevelResponse = harness
        .query("Observed", &Empty::default())
        .await
        .expect("harness query")
        .expect("the Query answers");
    sender
        .send_awaiting_ack(Command::ObservedFact(ReconcileObserved::Lamp(true)))
        .await
        .expect("the second fact is applied");
    let second: LevelResponse = harness
        .query("Observed", &Empty::default())
        .await
        .expect("harness query")
        .expect("the Query answers");
    assert_eq!(first, second);
}

#[tokio::test(start_paused = true)]
async fn in_process_commands_never_use_the_backbone() {
    let journal = Arc::new(Mutex::new(Vec::new()));
    let backend: Arc<dyn CommsBackend> = Arc::new(RecordingBackend {
        bus: ChannelBackend::default(),
        journal: Arc::clone(&journal),
    });
    let harness = Harness::<ReconcileService>::start_on(
        backend,
        ReconcileArguments {
            projection_changes: Arc::new(AtomicUsize::new(0)),
        },
    )
    .await
    .expect("the reconcile harness starts");
    harness
        .command_sender()
        .send_awaiting_ack(Command::Request(ReconcileRequest::SetDesired(true)))
        .await
        .expect("the Command is applied");
    let entries = lock_unpoisoned(&journal).clone();
    assert!(entries.iter().all(|entry| !entry.contains("command")));
}
