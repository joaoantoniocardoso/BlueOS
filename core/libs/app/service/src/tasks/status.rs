//! Service `status` State publication.

use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use tokio::sync::watch;
use tracing::warn;

use blueos_api::Message;
use blueos_comms::{CommsBackend, Sample};
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{ServiceStatus, ServiceStatusStatus},
};

use crate::sync::lock_unpoisoned;

#[derive(Clone)]
pub(super) struct StatusPublisher {
    pub(super) service: &'static str,
    pub(super) backend: Arc<dyn CommsBackend>,
    pub(super) key: String,
    pub(super) encoding: String,
    pub(super) latest: watch::Sender<Option<Bytes>>,
    pub(super) degraded_tasks: Arc<Mutex<BTreeSet<String>>>,
}

impl StatusPublisher {
    pub(super) async fn mark_running(&self, task: &str) {
        lock_unpoisoned(&self.degraded_tasks).remove(task);
        self.refresh_status().await;
    }

    pub(super) async fn mark_degraded(&self, task: &str) {
        lock_unpoisoned(&self.degraded_tasks).insert(task.to_owned());
        self.refresh_status().await;
    }

    async fn refresh_status(&self) {
        let (status, detail) = {
            let names = lock_unpoisoned(&self.degraded_tasks);
            service_status_from_task_names(&names)
        };
        self.publish(status, detail).await;
    }

    async fn publish(&self, status: ServiceStatusStatus, detail: String) {
        let message = ServiceStatus { status, detail };
        let sent: Result<(), PublishError> = async {
            let payload = Bytes::from(message.encode()?);
            if self.latest.borrow().as_ref() == Some(&payload) {
                return Ok(());
            }
            let sample = Sample::new(
                self.key.as_str(),
                Bytes::clone(&payload),
                self.encoding.as_str(),
            );
            self.backend.publish(sample).await?;
            self.latest.send_replace(Some(payload));
            Ok(())
        }
        .await;
        if let Err(error) = sent {
            warn!(%error, key = %self.key, "Failed to publish service status");
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum PublishError {
    #[error("the Message does not encode: {0}")]
    Encode(#[from] IdlError),
    #[error(transparent)]
    Comms(#[from] blueos_comms::CommsError),
}

fn service_status_from_task_names(names: &BTreeSet<String>) -> (ServiceStatusStatus, String) {
    let detail = names
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let status = if detail.is_empty() {
        ServiceStatusStatus::Ready
    } else {
        ServiceStatusStatus::Degraded
    };
    (status, detail)
}
