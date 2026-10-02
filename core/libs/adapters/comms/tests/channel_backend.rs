//! The comms contract on the in-process channel backend.

use core::time::Duration;
use std::{borrow::Cow, sync::Arc, time::SystemTime};

use bytes::Bytes;
use tokio::time::Instant;

use blueos_comms::{
    CommsBackend, CommsError, QueryBody, Sample, Subscriber, channel::ChannelBackend,
};

const CDR: &str = "application/cdr;example_msgs/msg/Pump";
const ACK: &str = "application/cdr;blueos_msgs/msg/CommandAck";

#[tokio::test]
async fn a_subscriber_receives_the_published_sample_without_a_copy() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let mut subscriber = backend
        .subscribe("blueos/v1/example/state/pump")
        .await
        .unwrap();
    let published_payload = Bytes::from_static(b"pump on");

    backend
        .publish(Sample::new(
            "blueos/v1/example/state/pump",
            Bytes::clone(&published_payload),
            CDR,
        ))
        .await
        .unwrap();

    let sample = subscriber.recv().await.unwrap();
    assert_eq!(sample.key(), "blueos/v1/example/state/pump");
    assert_eq!(sample.encoding(), CDR);
    let Cow::Borrowed(received_payload) = sample.payload().to_bytes() else {
        panic!("a contiguous payload must be borrowed, not copied");
    };
    assert_eq!(received_payload.as_ptr(), published_payload.as_ptr());
    assert_eq!(received_payload, b"pump on");
}

#[tokio::test]
async fn the_attachment_and_timestamp_travel_with_the_sample() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let mut subscriber = backend.subscribe("blueos/v1/example/log").await.unwrap();
    let published_at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    backend
        .publish(
            Sample::new("blueos/v1/example/log", Bytes::from_static(b"record"), CDR)
                .with_attachment(Bytes::from_static(b"correlation 7"))
                .with_timestamp(published_at),
        )
        .await
        .unwrap();

    let sample = subscriber.recv().await.unwrap();
    assert_eq!(sample.timestamp(), Some(published_at));
    assert_eq!(
        sample.attachment().unwrap().to_bytes(),
        &b"correlation 7"[..]
    );
}

#[tokio::test]
async fn wildcards_match_as_in_zenoh() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let mut one_chunk = backend.subscribe("blueos/v1/*/state").await.unwrap();
    let mut any_chunks = backend.subscribe("blueos/**").await.unwrap();
    let mut part_of_a_chunk = backend.subscribe("blueos/v1/cam$*/state").await.unwrap();
    // The last key matches every expression, so each subscriber reads until it.
    for key in [
        "blueos/v1/recorder/state",
        "blueos/v1/recorder/settings",
        "blueos/v1/a/b/state",
        "blueos",
        "blueos/v1/cam/state",
        "blueos/v1/camera1/state",
    ] {
        backend
            .publish(Sample::new(key, Bytes::new(), CDR))
            .await
            .unwrap();
    }

    assert_eq!(
        received_keys(&mut one_chunk).await,
        [
            "blueos/v1/recorder/state",
            "blueos/v1/cam/state",
            "blueos/v1/camera1/state"
        ]
    );
    assert_eq!(
        received_keys(&mut any_chunks).await,
        [
            "blueos/v1/recorder/state",
            "blueos/v1/recorder/settings",
            "blueos/v1/a/b/state",
            "blueos",
            "blueos/v1/cam/state",
            "blueos/v1/camera1/state",
        ]
    );
    assert_eq!(
        received_keys(&mut part_of_a_chunk).await,
        ["blueos/v1/cam/state", "blueos/v1/camera1/state"]
    );
}

#[tokio::test]
async fn a_non_canonical_key_expression_is_rejected() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());

    for key_expression in ["blueos/**/**", "blueos/v1/$*", "blueos//state"] {
        let Err(CommsError::InvalidKeyExpression {
            key_expression: rejected,
            ..
        }) = backend.subscribe(key_expression).await
        else {
            panic!("{key_expression} must be rejected");
        };
        assert_eq!(rejected, key_expression);
    }
    assert!(matches!(
        backend
            .publish(Sample::new("blueos/**/**", Bytes::new(), CDR))
            .await,
        Err(CommsError::InvalidKeyExpression { .. })
    ));
}

#[tokio::test]
async fn a_get_reaches_the_queryable_and_returns_its_reply_without_a_copy() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let mut queryable = backend
        .declare_queryable("blueos/v1/example/set_speed")
        .await
        .unwrap();
    let request = Bytes::from_static(b"speed 3");
    let acknowledgement = Bytes::from_static(b"accepted");

    let get = backend.get(
        "blueos/v1/example/set_speed",
        Some(QueryBody::new(Bytes::clone(&request), CDR)),
        Duration::from_secs(1),
    );
    let answer = async {
        let query = queryable.recv().await.unwrap();
        assert_eq!(query.key_expression(), "blueos/v1/example/set_speed");
        let body = query.body().unwrap();
        assert_eq!(body.encoding(), CDR);
        let Cow::Borrowed(received_request) = body.payload().to_bytes() else {
            panic!("a contiguous query payload must be borrowed, not copied");
        };
        assert_eq!(received_request.as_ptr(), request.as_ptr());
        query
            .reply(Bytes::clone(&acknowledgement), ACK)
            .await
            .unwrap();
    };
    let (replies, ()) = tokio::join!(get, answer);

    let replies = replies.unwrap();
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one successful reply, got {replies:?}");
    };
    assert_eq!(reply.key(), "blueos/v1/example/set_speed");
    assert_eq!(reply.encoding(), ACK);
    let Cow::Borrowed(received_acknowledgement) = reply.payload().to_bytes() else {
        panic!("a contiguous reply payload must be borrowed, not copied");
    };
    assert_eq!(received_acknowledgement.as_ptr(), acknowledgement.as_ptr());
}

#[tokio::test]
async fn a_wildcard_get_tells_replies_apart_by_their_declared_key() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    for (key, answer) in [
        ("blueos/v1/pump/state/status", "pump ready"),
        ("blueos/v1/light/state/status", "light ready"),
        ("blueos/v1/pump/set_speed", "speed changed"),
    ] {
        let mut queryable = backend.declare_queryable(key).await.unwrap();
        tokio::spawn(async move {
            while let Some(query) = queryable.recv().await {
                query
                    .reply(Bytes::from_static(answer.as_bytes()), "text/plain")
                    .await
                    .unwrap();
            }
        });
    }

    let replies = backend
        .get("blueos/v1/*/state/*", None, Duration::from_secs(1))
        .await
        .unwrap();

    let mut answers: Vec<(String, String)> = replies
        .into_iter()
        .map(|reply| {
            let sample = reply.unwrap();
            let answer = String::from_utf8(sample.payload().to_bytes().into_owned()).unwrap();
            (sample.key().to_owned(), answer)
        })
        .collect();
    answers.sort();
    assert_eq!(
        answers,
        [
            (
                "blueos/v1/light/state/status".to_owned(),
                "light ready".to_owned()
            ),
            (
                "blueos/v1/pump/state/status".to_owned(),
                "pump ready".to_owned()
            ),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn a_get_returns_what_arrived_when_the_timeout_expires() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let mut silent = backend
        .declare_queryable("blueos/v1/slow/state/status")
        .await
        .unwrap();
    let mut answering = backend
        .declare_queryable("blueos/v1/fast/state/status")
        .await
        .unwrap();
    let mut dropping = backend
        .declare_queryable("blueos/v1/empty/state/status")
        .await
        .unwrap();

    let get = backend.get("blueos/v1/*/state/status", None, Duration::from_secs(2));
    let answer = async {
        answering
            .recv()
            .await
            .unwrap()
            .reply(Bytes::from_static(b"ready"), "text/plain")
            .await
            .unwrap();
        drop(dropping.recv().await.unwrap());
        silent.recv().await.unwrap()
    };
    let started = Instant::now();
    let (replies, _held_query) = tokio::join!(get, answer);

    assert_eq!(started.elapsed(), Duration::from_secs(2));
    let replies = replies.unwrap();
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected only the fast reply, got {replies:?}");
    };
    assert_eq!(reply.key(), "blueos/v1/fast/state/status");
}

#[tokio::test(start_paused = true)]
async fn a_get_ends_without_waiting_once_every_queryable_has_answered() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let mut dropping = backend
        .declare_queryable("blueos/v1/example/state/status")
        .await
        .unwrap();
    let mut failing = backend
        .declare_queryable("blueos/v1/example/state/settings")
        .await
        .unwrap();

    let get = backend.get("blueos/v1/example/state/*", None, Duration::from_secs(2));
    let answer = async {
        drop(dropping.recv().await.unwrap());
        let query = failing.recv().await.unwrap();
        query
            .reply_error(Bytes::from_static(b"cannot decode"), "text/plain")
            .await
            .unwrap();
    };
    let started = Instant::now();
    let (replies, ()) = tokio::join!(get, answer);

    assert_eq!(started.elapsed(), Duration::ZERO);
    let replies = replies.unwrap();
    let [Err(error)] = replies.as_slice() else {
        panic!("expected one error reply, got {replies:?}");
    };
    assert_eq!(error.payload().to_bytes(), &b"cannot decode"[..]);
    assert_eq!(error.encoding(), "text/plain");
}

#[tokio::test]
async fn a_get_that_matches_no_queryable_returns_no_reply() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let _unrelated = backend
        .declare_queryable("blueos/v1/example/state/status")
        .await
        .unwrap();

    let replies = backend
        .get("blueos/v1/other/**", None, Duration::from_secs(1))
        .await
        .unwrap();

    assert!(replies.is_empty());
}

async fn received_keys(subscriber: &mut Subscriber) -> Vec<String> {
    let mut keys = Vec::new();
    while let Some(sample) = subscriber.recv().await {
        keys.push(sample.key().to_owned());
        if sample.key() == "blueos/v1/camera1/state" {
            return keys;
        }
    }
    keys
}
