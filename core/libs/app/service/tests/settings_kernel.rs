//! Kernel settings: load, `UpdateSettings`, persist rollback, and pending restart fields.

mod settings_kernel_fixture;

use core::time::Duration;

use blueos_api::{CommandAck, Message, job_result_key};
use blueos_idl::msg::blueos_msgs::{
    CommandAckStatus, JobResult, JobStatus, JobStatusStatus, SettingsEnvelope, UpdateSettingsGoal,
    UpdateSettingsResult,
};
use blueos_jobs::JobId;
use blueos_service::Service;
use blueos_settings::{SettingsSchema, settings_file_name};
use tokio::time;

use settings_kernel_fixture::*;

#[tokio::test(start_paused = true)]
async fn update_settings_persists_and_publishes_pending_restart() {
    let parent = temp_settings_parent("persist");
    let harness = start_with_settings_folder(parent.clone()).await;

    let updated = settings_tank_document(2, "changed");
    let ack = harness
        .send("UpdateSettings", &envelope_for(&updated))
        .await
        .unwrap();
    assert!(ack.accepted);

    let envelope = harness.settings::<SettingsEnvelope>().await.unwrap();
    assert_eq!(envelope.fields.len(), 1);
    assert_eq!(envelope.fields[0].path, "restart_field");
    assert!(envelope.fields[0].restart_required);

    let on_disk_path = parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        settings_file_name(SettingsTankDocument::VERSION)
    ));
    let on_disk: SettingsTankDocument =
        serde_json::from_str(&std::fs::read_to_string(on_disk_path).unwrap()).unwrap();
    assert_eq!(on_disk.restart_field, "changed");

    let restored = settings_tank_document(2, "initial");
    let restore_ack = harness
        .send("UpdateSettings", &envelope_for(&restored))
        .await
        .unwrap();
    assert!(restore_ack.accepted);
    let cleared_state = harness.settings::<SettingsEnvelope>().await.unwrap();
    assert!(cleared_state.fields.is_empty());

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn update_settings_is_an_instant_job_that_acks_its_final_status() {
    let parent = temp_settings_parent("instant-job");
    let harness = start_with_settings_folder(parent.clone()).await;
    let job_id = JobId::from_u128(7);
    let updated = settings_tank_document(2, "changed");

    let ack = harness
        .submit("UpdateSettings", job_id, &envelope_for(&updated))
        .await
        .unwrap();

    assert_eq!(
        ack,
        CommandAck {
            accepted: true,
            job_id: job_id.to_string(),
            status: CommandAckStatus::Succeeded,
            reason: String::new(),
        }
    );
    let running: SettingsTankDocument = serde_json::from_str(
        &harness
            .settings::<SettingsEnvelope>()
            .await
            .unwrap()
            .document_json,
    )
    .unwrap();
    assert_eq!(running, updated);
    assert_eq!(
        harness.jobs().await.unwrap().jobs,
        [JobStatus {
            job_id: job_id.to_string(),
            job_type: "UpdateSettings".to_owned(),
            status: JobStatusStatus::Succeeded,
            reason: String::new(),
        }]
    );

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn update_settings_takes_its_action_goal_and_publishes_its_action_result() {
    let parent = temp_settings_parent("action");
    let harness = start_with_settings_folder(parent.clone()).await;
    let mut results = harness
        .backend()
        .subscribe(&job_result_key(SettingsTankService::NAME, "UpdateSettings"))
        .await
        .unwrap();
    let job_id = JobId::from_u128(7);
    let goal = UpdateSettingsGoal {
        envelope: envelope_for(&SettingsTankDocument {
            live_field: 2,
            ..SettingsTankDocument::default()
        }),
    };

    let ack = harness
        .submit("UpdateSettings", job_id, &goal)
        .await
        .unwrap();
    let sample = time::timeout(Duration::from_secs(10), results.recv())
        .await
        .unwrap()
        .unwrap();
    let ended = JobResult::decode(&sample.payload().to_bytes()).unwrap();

    assert_eq!(ack.status, CommandAckStatus::Succeeded);
    assert_eq!(ended.job.status, JobStatusStatus::Succeeded);
    assert_eq!(
        UpdateSettingsResult::decode(&ended.result),
        Ok(UpdateSettingsResult::default())
    );

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn update_settings_appears_in_the_history_query_like_any_other_job_type() {
    let parent = temp_settings_parent("history");
    let harness = start_with_settings_folder(parent.clone()).await;
    let [first, second] = [7, 8].map(JobId::from_u128);
    for (job_id, live_field) in [(first, 2), (second, 3)] {
        let updated = SettingsTankDocument {
            version: SettingsTankDocument::VERSION,
            live_field,
            restart_field: "changed".into(),
        };
        harness
            .submit("UpdateSettings", job_id, &envelope_for(&updated))
            .await
            .unwrap();
    }

    let history = harness.job_history("UpdateSettings").await.unwrap();

    assert_eq!(
        history.jobs,
        [first, second].map(|job_id| JobStatus {
            job_id: job_id.to_string(),
            job_type: "UpdateSettings".to_owned(),
            status: JobStatusStatus::Succeeded,
            reason: String::new(),
        })
    );

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn a_retry_of_update_settings_does_not_write_again() {
    let parent = temp_settings_parent("retry");
    let harness = start_with_settings_folder(parent.clone()).await;
    let job_id = JobId::from_u128(7);
    let updated = envelope_for(&SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 2,
        restart_field: "changed".into(),
    });
    let first = harness
        .submit("UpdateSettings", job_id, &updated)
        .await
        .unwrap();
    let settings_path = parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        settings_file_name(SettingsTankDocument::VERSION)
    ));
    std::fs::remove_file(&settings_path).unwrap();

    let retry = harness
        .submit("UpdateSettings", job_id, &updated)
        .await
        .unwrap();

    assert_eq!(retry, first);
    assert!(!settings_path.exists());
    assert_eq!(harness.jobs().await.unwrap().jobs.len(), 1);

    let _ = std::fs::remove_dir_all(parent);
}
