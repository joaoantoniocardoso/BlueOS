//! Shared L5 conformance body for every [`CommsBackend`] implementation.

use core::time::Duration;
use std::{sync::Arc, time::SystemTime};

use bytes::Bytes;
use tokio::time::Instant;

use blueos_comms::{CommsBackend, CommsError, LivelinessEvent, QueryBody, Sample, Subscriber};

const CDR: &str = "application/cdr;example_msgs/msg/Pump";
const ACK: &str = "application/cdr;blueos_msgs/msg/CommandAck";

/// Runs every conformance check against a fresh backend from `factory`.
#[expect(
    unreachable_pub,
    reason = "included from blueos-comms-zenoh integration tests"
)]
pub async fn run_all(factory: &dyn Fn() -> Arc<dyn CommsBackend>) {
    a_subscriber_receives_the_published_sample_without_a_copy(factory()).await;
    the_attachment_and_timestamp_travel_with_the_sample(factory()).await;
    the_attachment_travels_with_the_query_body(factory()).await;
    wildcards_match_as_in_zenoh(factory()).await;
    a_non_canonical_key_expression_is_rejected(factory()).await;
    a_get_reaches_the_queryable_and_returns_its_reply_without_a_copy(factory()).await;
    a_wildcard_get_tells_replies_apart_by_their_declared_key(factory()).await;
    a_get_returns_what_arrived_when_the_timeout_expires(factory()).await;
    a_get_ends_without_waiting_once_every_queryable_has_answered(factory()).await;
    a_get_that_matches_no_queryable_returns_no_reply(factory()).await;
    state_subscribe_first_then_query_ignores_a_stale_reply(factory()).await;
    liveliness_put_and_delete(factory()).await;
    get_liveliness_lists_alive_tokens(factory()).await;
}

async fn a_subscriber_receives_the_published_sample_without_a_copy(backend: Arc<dyn CommsBackend>) {
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
    assert_eq!(sample.payload().to_bytes().as_ref(), b"pump on");
    if let Some(received) = sample.payload().downcast_ref::<Bytes>() {
        assert_eq!(received.as_ptr(), published_payload.as_ptr());
    }
}

async fn the_attachment_and_timestamp_travel_with_the_sample(backend: Arc<dyn CommsBackend>) {
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

async fn the_attachment_travels_with_the_query_body(backend: Arc<dyn CommsBackend>) {
    let mut queryable = backend
        .declare_queryable("blueos/v1/example/command/SetLevel")
        .await
        .unwrap();

    let get = backend.get(
        "blueos/v1/example/command/SetLevel",
        Some(
            QueryBody::new(Bytes::from_static(b"level 3"), CDR)
                .with_attachment(Bytes::from_static(b"job 7")),
        ),
        Duration::from_secs(1),
    );
    let answer = async {
        let query = queryable.recv().await.unwrap();
        let attachment = query.body().unwrap().attachment().unwrap().to_bytes();
        assert_eq!(attachment, &b"job 7"[..]);
        query.reply(Bytes::from_static(b"ok"), ACK).await.unwrap();
    };
    let (replies, ()) = tokio::join!(get, answer);

    assert_eq!(replies.unwrap().len(), 1);
}

async fn wildcards_match_as_in_zenoh(backend: Arc<dyn CommsBackend>) {
    let mut one_chunk = backend.subscribe("blueos/v1/*/state").await.unwrap();
    let mut any_chunks = backend.subscribe("blueos/**").await.unwrap();
    let mut part_of_a_chunk = backend.subscribe("blueos/v1/cam$*/state").await.unwrap();
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

async fn a_non_canonical_key_expression_is_rejected(backend: Arc<dyn CommsBackend>) {
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

async fn a_get_reaches_the_queryable_and_returns_its_reply_without_a_copy(
    backend: Arc<dyn CommsBackend>,
) {
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
        assert_eq!(body.payload().to_bytes(), request.as_ref());
        if let Some(received) = body.payload().downcast_ref::<Bytes>() {
            assert_eq!(received.as_ptr(), request.as_ptr());
        }
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
    assert_eq!(reply.payload().to_bytes(), acknowledgement.as_ref());
    if let Some(received) = reply.payload().downcast_ref::<Bytes>() {
        assert_eq!(received.as_ptr(), acknowledgement.as_ptr());
    }
}

async fn a_wildcard_get_tells_replies_apart_by_their_declared_key(backend: Arc<dyn CommsBackend>) {
    for (key, answer) in [
        ("blueos/v1/conformance/pump/state/status", "pump ready"),
        ("blueos/v1/conformance/light/state/status", "light ready"),
        ("blueos/v1/conformance/pump/set_speed", "speed changed"),
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
        .get(
            "blueos/v1/conformance/*/state/*",
            None,
            Duration::from_secs(1),
        )
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
                "blueos/v1/conformance/light/state/status".to_owned(),
                "light ready".to_owned()
            ),
            (
                "blueos/v1/conformance/pump/state/status".to_owned(),
                "pump ready".to_owned()
            ),
        ]
    );
}

async fn a_get_returns_what_arrived_when_the_timeout_expires(backend: Arc<dyn CommsBackend>) {
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

    assert!(started.elapsed() >= Duration::from_secs(2));
    let replies = replies.unwrap();
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected only the fast reply, got {replies:?}");
    };
    assert_eq!(reply.key(), "blueos/v1/fast/state/status");
}

async fn a_get_ends_without_waiting_once_every_queryable_has_answered(
    backend: Arc<dyn CommsBackend>,
) {
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

    assert!(started.elapsed() < Duration::from_secs(1));
    let replies = replies.unwrap();
    let [Err(error)] = replies.as_slice() else {
        panic!("expected one error reply, got {replies:?}");
    };
    assert_eq!(error.payload().to_bytes(), &b"cannot decode"[..]);
    assert_eq!(error.encoding(), "text/plain");
}

async fn a_get_that_matches_no_queryable_returns_no_reply(backend: Arc<dyn CommsBackend>) {
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

async fn state_subscribe_first_then_query_ignores_a_stale_reply(backend: Arc<dyn CommsBackend>) {
    const STATE_KEY: &str = "blueos/v1/example/state/status";

    let mut queryable = backend.declare_queryable(STATE_KEY).await.unwrap();
    tokio::spawn(async move {
        while let Some(query) = queryable.recv().await {
            query
                .reply(Bytes::from_static(b"stale"), "text/plain")
                .await
                .unwrap();
        }
    });

    let mut updates = backend.subscribe(STATE_KEY).await.unwrap();
    backend
        .publish(Sample::new(
            STATE_KEY,
            Bytes::from_static(b"fresh"),
            "text/plain",
        ))
        .await
        .unwrap();

    let stream_sample = updates.recv().await.unwrap();
    assert_eq!(stream_sample.payload().to_bytes(), &b"fresh"[..]);

    let replies = backend
        .get(STATE_KEY, None, Duration::from_secs(1))
        .await
        .unwrap();
    let [Ok(query_sample)] = replies.as_slice() else {
        panic!("expected one query reply, got {replies:?}");
    };
    assert_eq!(query_sample.payload().to_bytes(), &b"stale"[..]);

    // D-10: subscribe first, then query; ignore the query when the stream already delivered a sample.
    assert_eq!(stream_sample.payload().to_bytes(), &b"fresh"[..]);
}

async fn liveliness_put_and_delete(backend: Arc<dyn CommsBackend>) {
    let mut watcher = backend
        .subscribe_liveliness("blueos/v1/services/*")
        .await
        .unwrap();
    let token = backend
        .declare_liveliness("blueos/v1/services/recorder")
        .await
        .unwrap();

    let event = tokio::time::timeout(Duration::from_secs(1), watcher.recv())
        .await
        .expect("timeout")
        .expect("event");
    assert_eq!(
        event,
        LivelinessEvent::Put {
            key: "blueos/v1/services/recorder".to_owned(),
        }
    );

    drop(token);
    let delete_event = tokio::time::timeout(Duration::from_secs(1), watcher.recv())
        .await
        .expect("timeout")
        .expect("event");
    assert_eq!(
        delete_event,
        LivelinessEvent::Delete {
            key: "blueos/v1/services/recorder".to_owned(),
        }
    );
}

async fn get_liveliness_lists_alive_tokens(backend: Arc<dyn CommsBackend>) {
    let token = backend
        .declare_liveliness("blueos/v1/services/recorder")
        .await
        .unwrap();

    let keys = backend
        .get_liveliness("blueos/v1/services/*", Duration::from_millis(100))
        .await
        .unwrap();
    assert_eq!(keys, ["blueos/v1/services/recorder".to_owned()]);

    drop(token);
    let missing_keys = backend
        .get_liveliness("blueos/v1/missing/**", Duration::from_millis(50))
        .await
        .unwrap();
    assert!(missing_keys.is_empty());
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
