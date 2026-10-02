//! Ordered IO Effects for one Decision.

#![expect(
    type_alias_bounds,
    reason = "the executor alias documents the Domain bound; call sites always use a concrete Domain"
)]

use core::{future::Future, pin::Pin};
use std::sync::Arc;

use tokio::sync::mpsc;

use blueos_domain::{Command, Domain, IoError};

use super::Delivery;

/// Runs one IO request through the service's executor.
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

/// Runs every IO [`Effect`] of one Decision in order in one task and delivers each result to the Inbox.
pub(crate) fn spawn_io_chain<D: Domain, Context: Send + Sync + 'static>(
    executor: IoExecutor<D, Context>,
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
            let future = executor(&*context, &snapshot, request);
            let command = match tokio::spawn(future).await {
                Ok(Ok(Some(io_result))) => Command::IoResult(io_result),
                Ok(Ok(None)) => continue,
                Ok(Err(error)) => D::io_failed(failed_request, error),
                Err(_panic) => {
                    D::io_failed(failed_request, IoError::new("the IO request panicked"))
                }
            };
            if inbox
                .send(Delivery {
                    command,
                    reply: None,
                })
                .await
                .is_err()
            {
                return;
            }
        }
    });
}
