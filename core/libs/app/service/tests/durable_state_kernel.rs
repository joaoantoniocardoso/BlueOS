//! Durable state across restarts (ticket #56, D-28).

mod durable_state_kernel_fixture;

use core::time::Duration;
use std::{fs, sync::Arc};

use blueos_comms::channel::ChannelBackend;
use blueos_idl::msg::blueos_msgs::JobStatusStatus;
use blueos_service::{Kernel, Service, ServiceContext, testing::PausedClock};
use blueos_settings::STATE_NAME_PREFIX;
use tokio::time;

use durable_state_kernel_fixture::*;

#[tokio::test(start_paused = true)]
async fn ten_changes_within_one_second_produce_one_write() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let (backend, _sender, _shutdown, mut tasks, durable_flush) =
        start_vault_kernel(folder.clone(), false).await;

    for _ in 0..10 {
        send_command(&backend, "Bump").await;
    }
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 10).await;
    tasks.abort_all();
}

#[tokio::test(start_paused = true)]
async fn corrupt_file_is_moved_aside_and_the_service_starts_fresh() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let path = state_path(&folder);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(&path, b"{not json").expect("write corrupt file");

    let (backend, _sender, _shutdown, mut tasks, durable_flush) =
        start_vault_kernel(folder.clone(), false).await;
    send_command(&backend, "Bump").await;
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 1).await;
    tasks.abort_all();

    let corrupt = fs::read_dir(path.parent().expect("parent"))
        .expect("read dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(STATE_NAME_PREFIX) && name.contains("corrupt"))
        });
    assert!(corrupt.is_some(), "corrupt file was moved aside");
}

#[tokio::test(start_paused = true)]
async fn wrong_shape_with_matching_version_starts_fresh_without_restored_tick() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let path = state_path(&folder);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(
        &path,
        br#"{"VERSION":1,"domain":"not-an-object","jobs":{}}"#,
    )
    .expect("write malformed file");

    let (_backend, _sender, _shutdown, mut tasks, _durable_flush) =
        start_vault_kernel(folder.clone(), false).await;
    tasks.abort_all();

    let corrupt = fs::read_dir(path.parent().expect("parent"))
        .expect("read dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("corrupt"))
        });
    assert!(corrupt, "malformed file was moved aside");
    assert!(!path.is_file(), "state file was replaced");
}

#[tokio::test(start_paused = true)]
async fn a_restore_aborts_the_running_jobs_and_keeps_the_waiting_ones() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let (backend, _sender, _shutdown, mut tasks, durable_flush) =
        start_vault_kernel(folder.clone(), false).await;
    send_command(&backend, "StartJob").await;
    send_command(&backend, "AwaitApproval").await;
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 0).await;
    tasks.abort_all();

    let (restore_backend, _restore_sender, _restore_shutdown, mut restore_tasks, _restore_flush) =
        start_vault_kernel(folder.clone(), false).await;
    time::advance(Duration::from_millis(10)).await;
    let jobs = query_jobs(&restore_backend).await;
    restore_tasks.abort_all();

    let statuses: Vec<_> = jobs
        .jobs
        .iter()
        .map(|job| (job.job_type.as_str(), job.status, job.reason.as_str()))
        .collect();
    assert_eq!(
        statuses,
        [
            ("AwaitApproval", JobStatusStatus::WaitingForPermission, ""),
            ("StartJob", JobStatusStatus::Aborted, "interrupted"),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn observed_facts_and_rederivable_data_are_never_persisted() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let (backend, _sender, _shutdown, mut tasks, durable_flush) =
        start_vault_kernel(folder.clone(), false).await;
    send_command(&backend, "Bump").await;
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 1).await;
    tasks.abort_all();

    let raw = fs::read_to_string(state_path(&folder)).expect("state file");
    assert!(!raw.contains("observed"));
    assert!(!raw.contains("saw_restored_tick"));
}

#[tokio::test(start_paused = true)]
async fn shutdown_flush_persists_changes_from_the_final_inbox_drain() {
    let temp_folder = tempfile::tempdir().expect("tempdir");
    let folder = temp_folder.path().to_path_buf();
    let backend: Arc<dyn blueos_comms::CommsBackend> = Arc::new(ChannelBackend::default());
    let context = ServiceContext::with_settings_path(
        VaultArguments {},
        Some(folder.clone()),
        Arc::clone(&backend),
    );
    let (builder, shutdown) = vault_builder(&context, true);
    let shutdown = shutdown.expect("shutdown handle");
    let clock = Arc::new(PausedClock::start());
    let kernel = Kernel::start(VaultService::NAME, builder, (), Arc::clone(&backend), clock)
        .await
        .expect("vault starts");
    let durable_flush = kernel
        .durable_write_flush()
        .expect("vault registers durable state");
    let run = tokio::spawn(async move { kernel.run().await });

    send_command(&backend, "Bump").await;
    time::advance(Duration::from_secs(1)).await;
    flush_and_expect_durable(&durable_flush, &folder, 1).await;

    shutdown.trigger();
    let _ = run.await;

    assert_eq!(durable_value(&folder), Some(2));
}
