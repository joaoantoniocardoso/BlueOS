//! Queryable adapters that answer backbone reads outside the Inbox loop.

use core::panic::AssertUnwindSafe;
use std::{
    panic,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use futures_util::FutureExt;
use tokio::sync::{mpsc, watch};
use tracing::warn;

use blueos_api::{CommandAck, Message, cdr_encoding};
use blueos_comms::{CommsBackend, Query, QueryBody, Queryable, Sample};
use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::{ServiceStatus, ServiceStatusStatus, UpdateSettingsGoal};
use blueos_jobs::{JobId, JobNature};

use crate::{
    builder::{AnswerQuery, MessageType, Respond},
    clock::Clock,
    command_sender::command_ack,
    inbox::{CommandReply, Delivery, Input},
    service::ServiceError,
    settings::SettingsDriver,
    sync::lock_unpoisoned,
};

use super::types::{
    IntoInput, REASON_ENCODING, Rejection, SCHEMA_SEPARATOR, SendError, UPDATE_SETTINGS, Unanswered,
};

pub(crate) use self::{serve_watch_state as serve_settings, serve_watch_state as serve_state};

pub(crate) async fn publish_standard_status(
    backend: &Arc<dyn CommsBackend>,
    key: &str,
    encoding: &str,
    latest: &watch::Sender<Option<Bytes>>,
) {
    let status = ServiceStatus {
        status: ServiceStatusStatus::Ready,
        detail: String::new(),
    };
    let sent: Result<(), SendError> = async {
        let payload = Bytes::from(status.encode()?);
        if latest.borrow().as_ref() == Some(&payload) {
            return Ok(());
        }
        let sample = Sample::new(key, Bytes::clone(&payload), encoding);
        backend.publish(sample).await?;
        latest.send_replace(Some(payload));
        Ok(())
    }
    .await;
    warn_on_failure("State", key, sent);
}

/// Answers every `info` query with the same encoded [`ServiceInfo`].
pub(crate) async fn serve_fixed_reply(
    mut queryable: Queryable,
    key: String,
    payload: Bytes,
    encoding: String,
) {
    while let Some(query) = queryable.recv().await {
        let sent = query.reply(Bytes::clone(&payload), encoding.as_str()).await;
        warn_on_failure("Query", &key, sent.map_err(SendError::from));
    }
}

pub(crate) async fn declare(
    backend: &dyn CommsBackend,
    key: impl Into<String>,
) -> Result<Queryable, ServiceError> {
    let key = key.into();
    match backend.declare_queryable(&key).await {
        Ok(queryable) => Ok(queryable),
        Err(source) => Err(ServiceError::DeclareEndpoint { key, source }),
    }
}

/// Reads the Job id from each Command's attachment and decodes its body outside the Inbox loop, so a Command that
/// names no Job, or whose body does not decode, never reaches it.
pub(crate) async fn serve_command<D: Domain>(
    mut queryable: Queryable,
    into_input: IntoInput<D>,
    inbox: mpsc::Sender<Delivery<D>>,
) {
    while let Some(query) = queryable.recv().await {
        let job_id = attached_job_id(&query);
        let body = query
            .body()
            .map(|body| body.payload().to_bytes().into_owned())
            .unwrap_or_default();
        let input = job_id
            .ok_or(Rejection::NoJobId)
            .and_then(|job_id| into_input(job_id, body));
        match input {
            Ok(input) => {
                let delivery = Delivery {
                    input,
                    reply: Some(CommandReply::Query(query)),
                    persist_settings: false,
                };
                drop(inbox.send(delivery).await);
            }
            Err(rejection) => {
                complete_command_reply(
                    Some(CommandReply::Query(query)),
                    command_ack(job_id, None, Err(rejection)),
                )
                .await;
            }
        }
    }
}

/// Decodes `UpdateSettings` into an instant Job, validates the document, and queues it with a flag to persist on
/// success. A Service without settings refuses it.
pub(crate) async fn serve_update_settings<D: Domain>(
    mut queryable: Queryable,
    driver: Option<Arc<Mutex<Box<dyn SettingsDriver<D>>>>>,
    inbox: mpsc::Sender<Delivery<D>>,
) {
    while let Some(query) = queryable.recv().await {
        let job_id = attached_job_id(&query);
        let goal = query
            .body()
            .map(|body| body.payload().to_bytes().into_owned())
            .unwrap_or_default();
        let input = job_id.ok_or(Rejection::NoJobId).and_then(|job_id| {
            let driver = driver.as_ref().ok_or(Rejection::NoSettings)?;
            let UpdateSettingsGoal { envelope } =
                UpdateSettingsGoal::decode(&goal).map_err(Rejection::InvalidBody)?;
            let request = lock_unpoisoned(driver)
                .request_from_envelope(envelope)
                .map_err(Rejection::Domain)?;
            Ok(Input::Submit {
                job_id,
                job_type: UPDATE_SETTINGS.to_owned(),
                goal,
                nature: JobNature::INSTANT,
                request,
            })
        });
        match input {
            Ok(input) => {
                let delivery = Delivery {
                    input,
                    reply: Some(CommandReply::Query(query)),
                    persist_settings: true,
                };
                drop(inbox.send(delivery).await);
            }
            Err(rejection) => {
                complete_command_reply(
                    Some(CommandReply::Query(query)),
                    command_ack(job_id, None, Err(rejection)),
                )
                .await;
            }
        }
    }
}

/// The id of the Job a client's Command names in its attachment, if it is one.
pub(crate) fn attached_job_id(query: &Query) -> Option<JobId> {
    let attachment = query.body().and_then(QueryBody::attachment)?;
    core::str::from_utf8(&attachment.to_bytes())
        .ok()?
        .parse()
        .ok()
}

/// Answers every get on the `settings` State with the last value the backbone accepted.
pub(crate) async fn serve_watch_state(
    mut queryable: Queryable,
    key: String,
    encoding: String,
    latest: watch::Receiver<Option<Bytes>>,
) {
    while let Some(query) = queryable.recv().await {
        let payload = latest.borrow().as_ref().cloned();
        if let Some(payload) = payload {
            let sent = query.reply(payload, encoding.as_str()).await;
            warn_on_failure("State", &key, sent.map_err(SendError::from));
        }
    }
}

/// Answers every get on a Domain Query endpoint from the current Snapshot.
pub(crate) async fn serve_query<D: Domain>(
    mut queryable: Queryable,
    answer: AnswerQuery<D>,
    snapshot: Arc<tokio::sync::RwLock<D::Snapshot>>,
    clock: Arc<dyn Clock>,
) {
    while let Some(query) = queryable.recv().await {
        let body = query
            .body()
            .map(|body| body.payload().to_bytes())
            .unwrap_or_default();
        let now = clock.now();
        let answered = {
            let shared = snapshot.read().await;
            panic::catch_unwind(AssertUnwindSafe(|| answer(&shared, &body, now)))
                .unwrap_or(Err(Unanswered::Panicked))
        };
        reply(query, answered).await;
    }
}

/// Answers each IO query in turn, so one slow answer delays the next instead of running beside it.
pub(crate) async fn serve_io_query(mut queryable: Queryable, respond: Respond, encoding: String) {
    while let Some(query) = queryable.recv().await {
        let body = query
            .body()
            .map(|body| body.payload().to_bytes().into_owned());
        let answered = AssertUnwindSafe(respond(body.unwrap_or_default()))
            .catch_unwind()
            .await
            .unwrap_or(Err(Unanswered::Panicked));
        reply_with_encoding(query, answered, encoding.as_str()).await;
    }
}

async fn reply_with_encoding(query: Query, answered: Result<Vec<u8>, Unanswered>, encoding: &str) {
    let key = query.key_expression().to_owned();
    let sent = match answered {
        Ok(payload) => query.reply(payload, encoding).await,
        Err(unanswered) => {
            let reason = unanswered.to_string().into_bytes();
            query.reply_error(reason, REASON_ENCODING).await
        }
    };
    warn_on_failure("Query", &key, sent.map_err(SendError::from));
}

pub(crate) async fn complete_command_reply(reply: Option<CommandReply>, ack: CommandAck) {
    let Some(reply) = reply else {
        return;
    };
    match reply {
        CommandReply::Ack(sender) => {
            drop(sender.send(ack));
        }
        CommandReply::Query(query) => {
            let key = query.key_expression().to_owned();
            let sent: Result<(), SendError> = async {
                let encoding = cdr_encoding(CommandAck::SCHEMA_NAME);
                query.reply(ack.encode()?, encoding).await?;
                Ok(())
            }
            .await;
            warn_on_failure("CommandAck", &key, sent);
        }
    }
}

/// Sends a Query's answer, or an error reply whose payload is the reason it got none.
pub(crate) async fn reply(query: Query, answered: Result<(Vec<u8>, String), Unanswered>) {
    match answered {
        Ok((payload, encoding)) => reply_with_encoding(query, Ok(payload), &encoding).await,
        Err(unanswered) => reply_with_encoding(query, Err(unanswered), REASON_ENCODING).await,
    }
}

/// A failed publish or reply is logged and never stops the Inbox loop.
pub(crate) fn warn_on_failure(kind: &'static str, key: &str, sent: Result<(), SendError>) {
    if let Err(error) = sent {
        warn!(%error, kind, key, "Failed to send");
    }
}

/// The schema text of a Job output key as `info` lists it: the wrapper `M` on the wire, then the part its bytes
/// carry, under the `MSG: <package>/<Name>` line ROS 2 gives a dependency. A Job type without the part gets `M`
/// alone.
pub(crate) fn carrying<M: Message>(part: MessageType) -> String {
    match (part.name.split_once('/'), part.name.rsplit_once('/')) {
        (Some((package, _)), Some((_, name))) => format!(
            "{}\n{SCHEMA_SEPARATOR}\nMSG: {package}/{name}\n{}",
            M::SCHEMA,
            part.schema
        ),
        _ => M::SCHEMA.to_owned(),
    }
}
