//! Settings write failures, durable rollback, and hostile document rejection.

mod settings_kernel_fixture;

use core::time::Duration;

use blueos_idl::msg::blueos_msgs::{CommandAckStatus, SettingsEnvelope};
use blueos_jobs::JobId;
use blueos_service::{Service, testing::Harness};
use blueos_settings::{SettingsSchema, settings_file_name};
use tokio::time;

use settings_kernel_fixture::*;

#[tokio::test(start_paused = true)]
async fn write_failure_restores_snapshot_and_rejects() {
    let parent = temp_settings_parent("write-fail");
    let harness = start_with_settings_folder(parent.clone()).await;
    let settings_path = settings_document_path(&parent);
    std::fs::remove_file(&settings_path).unwrap();
    std::fs::create_dir(&settings_path).unwrap();

    let updated = settings_tank_document(99, "blocked");
    let ack = harness
        .send("UpdateSettings", &envelope_for(&updated))
        .await
        .unwrap();
    assert!(!ack.accepted);

    let envelope = harness.settings::<SettingsEnvelope>().await.unwrap();
    let running: SettingsTankDocument = serde_json::from_str(&envelope.document_json).unwrap();
    assert_eq!(running.live_field, 1);
    assert!(harness.jobs().await.unwrap().jobs.is_empty());

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn settings_write_failure_does_not_queue_durable_snapshot() {
    let parent = temp_settings_parent("durable-rollback");
    let harness = start_with_settings_folder(parent.clone()).await;

    let accepted = settings_tank_document(2, "ok");
    let ack = harness
        .send("UpdateSettings", &envelope_for(&accepted))
        .await
        .unwrap();
    assert!(ack.accepted);
    time::advance(Duration::from_secs(1)).await;
    harness.flush_durable_writes().await;
    assert_eq!(durable_live_field_on_disk(&parent), Some(2));

    let settings_path = settings_document_path(&parent);
    std::fs::remove_file(&settings_path).unwrap();
    std::fs::create_dir(&settings_path).unwrap();

    let blocked = SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 99,
        restart_field: "blocked".into(),
    };
    let blocked_ack = harness
        .send("UpdateSettings", &envelope_for(&blocked))
        .await
        .unwrap();
    assert!(!blocked_ack.accepted);

    time::advance(Duration::from_secs(1)).await;
    harness.flush_durable_writes().await;
    assert_eq!(
        durable_live_field_on_disk(&parent),
        Some(2),
        "durable file must match the rolled-back snapshot, not the rejected update"
    );

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn rejects_foreign_version() {
    let parent = temp_settings_parent("foreign-version");
    let harness = start_with_settings_folder(parent.clone()).await;

    let document = serde_json::json!({
        "VERSION": 9,
        "live_field": 1,
        "restart_field": "x"
    });
    let envelope = SettingsEnvelope {
        document_json: document.to_string(),
        fields: Vec::new(),
    };
    let job_id = JobId::from_u128(7);
    let ack = harness
        .submit("UpdateSettings", job_id, &envelope)
        .await
        .unwrap();
    assert!(!ack.accepted);
    assert_eq!(ack.job_id, job_id.to_string());
    assert_eq!(ack.status, CommandAckStatus::StatusUnknown);

    let running: SettingsTankDocument = serde_json::from_str(
        &harness
            .settings::<SettingsEnvelope>()
            .await
            .unwrap()
            .document_json,
    )
    .unwrap();
    assert_eq!(running, SettingsTankDocument::default());
    let on_disk: SettingsTankDocument = serde_json::from_str(
        &std::fs::read_to_string(parent.join(format!(
            "{}/{}",
            SettingsTankService::NAME,
            settings_file_name(SettingsTankDocument::VERSION)
        )))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(on_disk, SettingsTankDocument::default());
    assert!(harness.jobs().await.unwrap().jobs.is_empty());

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn rejects_unknown_fields_in_document() {
    let parent = temp_settings_parent("unknown-field");
    let harness = start_with_settings_folder(parent.clone()).await;

    let document = serde_json::json!({
        "VERSION": 1,
        "live_field": 1,
        "restart_field": "x",
        "unexpected": true
    });
    let envelope = SettingsEnvelope {
        document_json: document.to_string(),
        fields: Vec::new(),
    };
    let ack = harness.send("UpdateSettings", &envelope).await.unwrap();
    assert!(!ack.accepted);

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn harness_ignores_hostile_settings_in_the_user_config_folder() {
    if std::env::var_os("BLUEOS_CONFIG_ISOLATION_PROBE").is_none() {
        let config_home = temp_settings_parent("hostile-user-config");
        let service_folder = config_home.join(SettingsTankService::NAME);
        std::fs::create_dir_all(&service_folder).unwrap();
        let hostile = SettingsTankDocument {
            version: SettingsTankDocument::VERSION,
            live_field: 99,
            restart_field: "hostile".into(),
        };
        std::fs::write(
            service_folder.join(settings_file_name(SettingsTankDocument::VERSION)),
            serde_json::to_string(&hostile).unwrap(),
        )
        .unwrap();

        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("harness_ignores_hostile_settings_in_the_user_config_folder")
            .arg("--exact")
            .env("XDG_CONFIG_HOME", &config_home)
            .env("BLUEOS_CONFIG_ISOLATION_PROBE", "1")
            .status()
            .unwrap();
        let untouched: SettingsTankDocument = serde_json::from_str(
            &std::fs::read_to_string(
                service_folder.join(settings_file_name(SettingsTankDocument::VERSION)),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(untouched.live_field, 99);
        let _ = std::fs::remove_dir_all(config_home);
        assert!(status.success());
        return;
    }

    let harness = Harness::<SettingsTankService>::start(SettingsTankArguments)
        .await
        .expect("harness");
    let envelope = harness.settings::<SettingsEnvelope>().await.unwrap();
    let running: SettingsTankDocument = serde_json::from_str(&envelope.document_json).unwrap();
    assert_eq!(running.live_field, 1);
}
