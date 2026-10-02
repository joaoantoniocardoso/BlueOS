//! Ordered IO Effects for one Decision.

#![expect(
    type_alias_bounds,
    reason = "the executor alias documents the Domain bound; call sites always use a concrete Domain"
)]

use core::{future::Future, panic::AssertUnwindSafe, pin::Pin};
use std::sync::Arc;

use tokio::sync::mpsc;

use blueos_domain::{Command, Domain, IoError};

use super::Delivery;

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

/// Runs every IO [`Effect`] of one Decision in order in one task and delivers each result to the Inbox.
pub(crate) fn spawn_io_chain<D: Domain, Context: Send + Sync + 'static>(
    executors: IoExecutors<D, Context>,
    context: Arc<Context>,
    snapshot: D::Snapshot,
    requests: Vec<D::IoRequest>,
    inbox: mpsc::Sender<Delivery<D>>,
) {
    if requests.is_empty() {
        return;
    }
    tokio::spawn(async move {
        for request in requests {
            let failed_request = request.clone();
            let command = if D::io_runs_on_blocking_thread(&request) {
                let Some(blocking) = executors.blocking.clone() else {
                    let command = D::io_failed(
                        failed_request,
                        IoError::new(
                            "the Decision scheduled blocking IO but the Service did not register a blocking IO executor",
                        ),
                    );
                    if inbox
                        .send(Delivery {
                            command,
                            reply: None,
                            persist_settings: false,
                        })
                        .await
                        .is_err()
                    {
                        return;
                    }
                    continue;
                };
                let context = Arc::clone(&context);
                let snapshot = snapshot.clone();
                match tokio::task::spawn_blocking(move || {
                    std::panic::catch_unwind(AssertUnwindSafe(|| {
                        blocking(&*context, &snapshot, request)
                    }))
                })
                .await
                {
                    Ok(Ok(Ok(Some(io_result)))) => Command::IoResult(io_result),
                    Ok(Ok(Ok(None))) => continue,
                    Ok(Ok(Err(error))) => D::io_failed(failed_request, error),
                    Ok(Err(_panic)) => {
                        D::io_failed(failed_request, IoError::new("the IO request panicked"))
                    }
                    Err(_join_error) => {
                        D::io_failed(failed_request, IoError::new("the IO request panicked"))
                    }
                }
            } else {
                let Some(r#async) = executors.r#async.clone() else {
                    let command = D::io_failed(
                        failed_request,
                        IoError::new(
                            "the Decision scheduled IO but the Service did not register an IO executor",
                        ),
                    );
                    if inbox
                        .send(Delivery {
                            command,
                            reply: None,
                            persist_settings: false,
                        })
                        .await
                        .is_err()
                    {
                        return;
                    }
                    continue;
                };
                let context = Arc::clone(&context);
                let snapshot = snapshot.clone();
                let future = r#async(&*context, &snapshot, request);
                match tokio::spawn(future).await {
                    Ok(Ok(Some(io_result))) => Command::IoResult(io_result),
                    Ok(Ok(None)) => continue,
                    Ok(Err(error)) => D::io_failed(failed_request, error),
                    Err(_panic) => {
                        D::io_failed(failed_request, IoError::new("the IO request panicked"))
                    }
                }
            };
            if inbox
                .send(Delivery {
                    command,
                    reply: None,
                    persist_settings: false,
                })
                .await
                .is_err()
            {
                return;
            }
        }
    });
}
