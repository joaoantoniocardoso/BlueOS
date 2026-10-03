//! Kernel-owned durable state load, debounced persist, and restore Tick (D-28).

use core::{num::NonZeroU32, time::Duration};
use std::{path::PathBuf, sync::Arc};

use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{mpsc, oneshot};
use tracing::warn;

use blueos_domain::{Domain, DomainDurable};
use blueos_jobs::{DomainJobs, Jobs};
use blueos_settings::ServiceStateStore;

use crate::clock::Clock;

/// How long the Kernel waits after the last change before writing durable state.
const WRITE_DEBOUNCE: Duration = Duration::from_secs(1);

/// Configuration collected in [`crate::ServiceBuilder::durable_state`]; the [`crate::Kernel`] opens the persister at
/// startup.
pub(crate) struct DurableStateRegistration<D: Domain> {
    pub(crate) store: ServiceStateStore,
    #[expect(clippy::type_complexity, reason = "one closure per registered Domain")]
    pub(crate) serialize: Box<dyn Fn(&D::Snapshot) -> Vec<u8> + Send + Sync>,
    #[expect(clippy::type_complexity, reason = "one closure per registered Domain")]
    pub(crate) restore_from_disk: Box<dyn Fn(&mut D::Snapshot, &dyn Clock) -> bool + Send + Sync>,
    #[expect(clippy::type_complexity, reason = "one closure per registered Domain")]
    pub(crate) changed: Box<dyn Fn(&D::Snapshot, &D::Snapshot) -> bool + Send + Sync>,
    pub(crate) restored_tick: D::Tick,
}

/// Debounced, off-runtime writes for one durable state file.
pub(crate) struct DurablePersister {
    request: mpsc::UnboundedSender<PersistMessage>,
    task: tokio::task::JoinHandle<()>,
}

/// Active durable state for one running Kernel.
pub(crate) struct DurableStateHandle<D: Domain> {
    pub(crate) persister: DurablePersister,
    #[expect(clippy::type_complexity, reason = "one closure per registered Domain")]
    pub(crate) serialize: Box<dyn Fn(&D::Snapshot) -> Vec<u8> + Send + Sync>,
    #[expect(clippy::type_complexity, reason = "one closure per registered Domain")]
    pub(crate) changed: Box<dyn Fn(&D::Snapshot, &D::Snapshot) -> bool + Send + Sync>,
}

enum PersistMessage {
    Document(Vec<u8>),
    Flush(oneshot::Sender<()>),
}

/// Lets tests wait for debounced durable writes without shutting the persister down.
#[cfg(feature = "testing")]
pub struct DurableWriteFlush {
    request: mpsc::UnboundedSender<PersistMessage>,
}

#[cfg(feature = "testing")]
impl DurableWriteFlush {
    /// Waits until any debounced durable document queued so far has been written.
    pub async fn flush(&self) {
        let (done, receiver) = oneshot::channel();
        if let Err(error) = self.request.send(PersistMessage::Flush(done)) {
            warn!(%error, "Durable state flush queue closed");
            return;
        }
        let _ = receiver.await;
    }
}

impl DurablePersister {
    pub(crate) fn spawn(clock: Arc<dyn Clock>, store: ServiceStateStore) -> Self {
        let store = Arc::new(store);
        let (request, receiver) = mpsc::unbounded_channel();
        let task = tokio::spawn(persist_loop(receiver, clock, store));
        Self { request, task }
    }

    pub(crate) fn queue_document(&self, bytes: Vec<u8>) {
        if let Err(error) = self.request.send(PersistMessage::Document(bytes)) {
            warn!(%error, "Durable state write queue closed");
        }
    }

    pub(crate) async fn flush_pending(&self) {
        let (done, receiver) = oneshot::channel();
        if let Err(error) = self.request.send(PersistMessage::Flush(done)) {
            warn!(%error, "Durable state flush queue closed");
            return;
        }
        let _ = receiver.await;
    }

    pub(crate) async fn flush_and_shutdown(&mut self) {
        self.flush_pending().await;
        self.task.abort();
    }

    #[cfg(feature = "testing")]
    pub(crate) fn flush_handle(&self) -> DurableWriteFlush {
        DurableWriteFlush {
            request: self.request.clone(),
        }
    }
}

#[derive(Serialize, serde::Deserialize)]
struct DomainOnlyEnvelope<D> {
    #[serde(rename = "VERSION")]
    version: u32,
    domain: D,
}

#[derive(Serialize, serde::Deserialize)]
struct PersistedEnvelopeWithJobs<D> {
    #[serde(rename = "VERSION")]
    version: u32,
    domain: D,
    jobs: Jobs,
}

async fn persist_loop(
    mut receiver: mpsc::UnboundedReceiver<PersistMessage>,
    clock: Arc<dyn Clock>,
    store: Arc<ServiceStateStore>,
) {
    let mut pending: Option<Vec<u8>> = None;
    let mut deadline: Option<Duration> = None;
    loop {
        let sleep = async {
            match deadline {
                Some(target) => {
                    let now = clock.now().monotonic;
                    if target <= now {
                        return;
                    }
                    tokio::time::sleep(target - now).await;
                }
                None => core::future::pending().await,
            }
        };
        tokio::select! {
            message = receiver.recv() => {
                match message {
                    Some(PersistMessage::Document(bytes)) => {
                        pending = Some(bytes);
                        if deadline.is_none() {
                            deadline = Some(clock.now().monotonic + WRITE_DEBOUNCE);
                        }
                    }
                    Some(PersistMessage::Flush(done)) => {
                        if let Some(bytes) = pending.take() {
                            write_on_blocking_pool(&store, bytes).await;
                        }
                        deadline = None;
                        let _ = done.send(());
                    }
                    None => break,
                }
            }
            () = sleep, if deadline.is_some() => {
                if let Some(bytes) = pending.take() {
                    write_on_blocking_pool(&store, bytes).await;
                }
                deadline = None;
            }
        }
    }
}

async fn write_on_blocking_pool(store: &ServiceStateStore, bytes: Vec<u8>) {
    let path = store.path().display().to_string();
    let store = store.clone();
    let result = tokio::task::spawn_blocking(move || store.write_document(&bytes)).await;
    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => warn!(%error, path, "Durable state write failed"),
        Err(error) => warn!(%error, path, "Durable state write task failed"),
    }
}

/// Records durable state without Jobs.
pub(crate) fn register_durable_state<D>(
    service_name: String,
    config_folder: Option<PathBuf>,
    version: NonZeroU32,
) -> DurableStateRegistration<D>
where
    D: DomainDurable,
    D::DurableState: Serialize + DeserializeOwned + PartialEq,
{
    let store = ServiceStateStore::open(service_name, config_folder, version);
    let version_raw = version.get();
    let serialize = move |snapshot: &D::Snapshot| {
        let envelope = DomainOnlyEnvelope {
            version: version_raw,
            domain: D::durable_state(snapshot).clone(),
        };
        serde_json::to_vec(&envelope).expect("durable state serializes")
    };
    let changed = |backup: &D::Snapshot, current: &D::Snapshot| {
        D::durable_state(backup) != D::durable_state(current)
    };
    let load_into = {
        let store = store.clone();
        move |snapshot: &mut D::Snapshot, clock: &dyn Clock| {
            restore_domain_only::<D>(&store, snapshot, clock)
        }
    };
    DurableStateRegistration {
        store,
        serialize: Box::new(serialize),
        restore_from_disk: Box::new(load_into),
        changed: Box::new(changed),
        restored_tick: D::restored_tick(),
    }
}

/// Records durable state together with Jobs (D-28).
pub(crate) fn register_durable_state_with_jobs<D>(
    service_name: String,
    config_folder: Option<PathBuf>,
    version: NonZeroU32,
) -> DurableStateRegistration<D>
where
    D: DomainDurable + DomainJobs,
    D::DurableState: Serialize + DeserializeOwned + PartialEq,
{
    let store = ServiceStateStore::open(service_name, config_folder, version);
    let version_raw = version.get();
    let serialize = move |snapshot: &D::Snapshot| {
        let envelope = PersistedEnvelopeWithJobs {
            version: version_raw,
            domain: D::durable_state(snapshot).clone(),
            jobs: D::jobs(snapshot).clone(),
        };
        serde_json::to_vec(&envelope).expect("durable state serializes")
    };
    let changed = |backup: &D::Snapshot, current: &D::Snapshot| {
        D::durable_state(backup) != D::durable_state(current) || D::jobs(backup) != D::jobs(current)
    };
    let load_into = {
        let store = store.clone();
        move |snapshot: &mut D::Snapshot, clock: &dyn Clock| {
            restore_domain_and_jobs::<D>(&store, snapshot, clock)
        }
    };
    DurableStateRegistration {
        store,
        serialize: Box::new(serialize),
        restore_from_disk: Box::new(load_into),
        changed: Box::new(changed),
        restored_tick: D::restored_tick(),
    }
}

fn restore_domain_only<D>(
    store: &ServiceStateStore,
    snapshot: &mut D::Snapshot,
    clock: &dyn Clock,
) -> bool
where
    D: DomainDurable,
    D::DurableState: DeserializeOwned,
{
    let Some(document) = store.read_document() else {
        return false;
    };
    let envelope: DomainOnlyEnvelope<D::DurableState> = match serde_json::from_value(document) {
        Ok(envelope) => envelope,
        Err(error) => {
            warn!(
                %error,
                path = %store.path().display(),
                "Durable state file does not match the schema; starting fresh"
            );
            store.move_aside_corrupt_at_wall_millis(Some(clock.now().wall.as_millis()));
            return false;
        }
    };
    D::set_durable_state(snapshot, envelope.domain);
    true
}

fn restore_domain_and_jobs<D>(
    store: &ServiceStateStore,
    snapshot: &mut D::Snapshot,
    clock: &dyn Clock,
) -> bool
where
    D: DomainDurable + DomainJobs,
    D::DurableState: DeserializeOwned,
{
    let Some(document) = store.read_document() else {
        return false;
    };
    let envelope: PersistedEnvelopeWithJobs<D::DurableState> =
        match serde_json::from_value(document) {
            Ok(envelope) => envelope,
            Err(error) => {
                warn!(
                    %error,
                    path = %store.path().display(),
                    "Durable state file does not match the schema; starting fresh"
                );
                store.move_aside_corrupt_at_wall_millis(Some(clock.now().wall.as_millis()));
                return false;
            }
        };
    D::set_durable_state(snapshot, envelope.domain);
    *D::jobs_mut(snapshot) = envelope.jobs;
    D::jobs_mut(snapshot).interrupt();
    true
}
