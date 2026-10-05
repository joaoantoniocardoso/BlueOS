//! Ordered IO Effects for one Decision.

#![expect(
    type_alias_bounds,
    reason = "the executor alias documents the Domain bound; call sites always use a concrete Domain"
)]

use core::{future::Future, panic::AssertUnwindSafe, pin::Pin};
use std::sync::Arc;

use futures_util::FutureExt;
use tokio::sync::mpsc;

use blueos_domain::{Command, Domain, IoError};

use crate::{
    builder::InboxCommand,
    inbox::{Delivery, Input},
    shutdown::IoInflight,
    tasks::TaskSpawner,
};

/// Runs one IO request through the service's async executor.
pub(crate) type IoExecutor<D, Context>
where
    D: Domain,
= Arc<
    dyn Fn(
            &Context,
            &D::Snapshot,
            D::IoRequest,
        ) -> Pin<Box<dyn Future<Output = Result<Option<D::IoResult>, IoError>> + Send>>
        + Send
        + Sync,
>;

/// Runs one IO request on a blocking thread through the service's blocking executor.
pub(crate) type BlockingIoExecutor<D, Context>
where
    D: Domain,
= Arc<
    dyn Fn(&Context, &D::Snapshot, D::IoRequest) -> Result<Option<D::IoResult>, IoError>
        + Send
        + Sync,
>;

/// The async and blocking IO executors a Service registers in `build`.
pub(crate) struct IoExecutors<D: Domain, Context> {
    pub(crate) r#async: Option<IoExecutor<D, Context>>,
    pub(crate) blocking: Option<BlockingIoExecutor<D, Context>>,
}

impl<D: Domain, Context> Clone for IoExecutors<D, Context> {
    fn clone(&self) -> Self {
        Self {
            r#async: self.r#async.clone(),
            blocking: self.blocking.clone(),
        }
    }
}

impl<D: Domain, Context> IoExecutors<D, Context> {
    /// Whether every IO request in `requests` has a registered executor.
    pub(crate) fn can_run(&self, requests: &[D::IoRequest]) -> bool {
        requests.iter().all(|request| {
            if D::io_runs_on_blocking_thread(request) {
                self.blocking.is_some()
            } else {
                self.r#async.is_some()
            }
        })
    }
}

/// Work passed to the IO chain task for one Decision.
pub(crate) struct IoChainWork<D: Domain, Context> {
    pub(crate) spawner: TaskSpawner,
    pub(crate) executors: IoExecutors<D, Context>,
    pub(crate) context: Arc<Context>,
    pub(crate) snapshot: D::Snapshot,
    pub(crate) requests: Vec<D::IoRequest>,
    pub(crate) inbox: mpsc::Sender<Delivery<D>>,
    pub(crate) io_inflight: IoInflight,
}

impl<D: Domain, Context: Send + Sync + 'static> IoChainWork<D, Context> {
    fn spawn(self) {
        if self.requests.is_empty() {
            return;
        }
        let _inflight = self.io_inflight.track();
        self.spawner.spawn(async move {
            let _inflight = _inflight;
            let snapshot = Arc::new(self.snapshot);
            let blocking_executor = self.executors.blocking;
            let async_executor = self.executors.r#async;
            for request in self.requests {
                let work = RunOneIoRequestWork {
                    blocking_executor: &blocking_executor,
                    async_executor: &async_executor,
                    context: &self.context,
                    snapshot: &snapshot,
                    inbox: &self.inbox,
                };
                if work.run(request).await {
                    return;
                }
            }
        });
    }
}

struct RunOneIoRequestWork<'a, D: Domain, Context> {
    blocking_executor: &'a Option<BlockingIoExecutor<D, Context>>,
    async_executor: &'a Option<IoExecutor<D, Context>>,
    context: &'a Arc<Context>,
    snapshot: &'a Arc<D::Snapshot>,
    inbox: &'a mpsc::Sender<Delivery<D>>,
}

impl<'a, D: Domain, Context: Send + Sync + 'static> RunOneIoRequestWork<'a, D, Context> {
    async fn run(self, request: D::IoRequest) -> bool {
        let command = if D::io_runs_on_blocking_thread(&request) {
            run_blocking_io_request::<D, Context>(
                self.blocking_executor,
                self.context,
                self.snapshot,
                request,
            )
            .await
        } else {
            run_async_io_request::<D, Context>(
                self.async_executor,
                self.context,
                self.snapshot,
                request,
            )
            .await
        };
        match command {
            Some(command) => deliver_io_command(self.inbox, command).await,
            None => false,
        }
    }
}

async fn run_blocking_io_request<D: Domain, Context: Send + Sync + 'static>(
    blocking_executor: &Option<BlockingIoExecutor<D, Context>>,
    context: &Arc<Context>,
    snapshot: &Arc<D::Snapshot>,
    request: D::IoRequest,
) -> Option<InboxCommand<D>> {
    let failed_request = request.clone();
    let Some(blocking) = blocking_executor.as_ref() else {
        return Some(D::io_failed(
            failed_request,
            IoError::new(
                "the Decision scheduled blocking IO but the Service did not register a blocking IO executor",
            ),
        ));
    };
    let context = Arc::clone(context);
    let snapshot = Arc::clone(snapshot);
    let blocking = Arc::clone(blocking);
    match tokio::task::spawn_blocking(move || {
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            blocking(&*context, snapshot.as_ref(), request)
        }))
    })
    .await
    {
        Ok(Ok(Ok(Some(io_result)))) => Some(Command::IoResult(io_result)),
        Ok(Ok(Ok(None))) => None,
        Ok(Ok(Err(error))) => Some(D::io_failed(failed_request, error)),
        Ok(Err(_)) | Err(_) => Some(D::io_failed(
            failed_request,
            IoError::new("the IO request panicked"),
        )),
    }
}

async fn run_async_io_request<D: Domain, Context: Send + Sync + 'static>(
    async_executor: &Option<IoExecutor<D, Context>>,
    context: &Arc<Context>,
    snapshot: &Arc<D::Snapshot>,
    request: D::IoRequest,
) -> Option<InboxCommand<D>> {
    let failed_request = request.clone();
    let Some(r#async) = async_executor.as_ref() else {
        return Some(D::io_failed(
            failed_request,
            IoError::new(
                "the Decision scheduled IO but the Service did not register an IO executor",
            ),
        ));
    };
    let context = Arc::clone(context);
    let snapshot = Arc::clone(snapshot);
    let future = r#async(&*context, snapshot.as_ref(), request);
    match AssertUnwindSafe(future).catch_unwind().await {
        Ok(Ok(Some(io_result))) => Some(Command::IoResult(io_result)),
        Ok(Ok(None)) => None,
        Ok(Err(error)) => Some(D::io_failed(failed_request, error)),
        Err(_panic) => Some(D::io_failed(
            failed_request,
            IoError::new("the IO request panicked"),
        )),
    }
}

async fn deliver_io_command<D: Domain>(
    inbox: &mpsc::Sender<Delivery<D>>,
    command: InboxCommand<D>,
) -> bool {
    inbox
        .send(Delivery {
            input: Input::Command(command),
            reply: None,
            persist_settings: false,
        })
        .await
        .is_err()
}

/// Runs every IO [`Effect`] of one Decision in order in one task and delivers each result to the Inbox.
pub(crate) fn spawn_io_chain<D: Domain, Context: Send + Sync + 'static>(
    work: IoChainWork<D, Context>,
) {
    work.spawn();
}
