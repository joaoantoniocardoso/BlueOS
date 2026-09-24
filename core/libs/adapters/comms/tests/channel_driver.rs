use std::time::Duration;

use blueos_comms::{ChannelBackend, CommsError, LivelinessEvent, Payload, Session};
use bytes::Bytes;
use futures::StreamExt;

#[tokio::test]
async fn channel_publish_subscribe_wildcard() {
    let (left, right) = ChannelBackend::pair();
    let publisher = Session::with_channel(left);
    let subscriber = Session::with_channel(right);
    let mut stream = subscriber
        .subscribe("blueos/v1/*/events")
        .await
        .expect("subscribe");
    publisher
        .publish(
            "blueos/v1/recorder/events",
            Payload::from_bytes(Bytes::from_static(b"event")),
            "text/plain",
            None,
        )
        .await
        .expect("publish");
    let sample = tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("timeout")
        .expect("stream");
    assert_eq!(sample.key, "blueos/v1/recorder/events");
}

#[tokio::test]
async fn channel_queryable_query_timeout() {
    let (service, client) = ChannelBackend::pair();
    let service_session = Session::with_channel(service);
    let client_session = Session::with_channel(client);
    let mut queries = service_session
        .declare_queryable("blueos/v1/svc/query")
        .await
        .expect("queryable");
    tokio::spawn(async move {
        while let Some(query) = queries.next().await {
            let _ = query
                .reply(
                    Payload::from_bytes(Bytes::from_static(b"answer")),
                    "text/plain",
                )
                .await;
        }
    });
    let reply = client_session
        .query(
            "blueos/v1/svc/query",
            Payload::empty(),
            "",
            Duration::from_secs(1),
        )
        .await
        .expect("query");
    assert_eq!(reply.payload.as_slice(), b"answer");
    let timeout = client_session
        .query(
            "blueos/v1/missing",
            Payload::empty(),
            "",
            Duration::from_millis(50),
        )
        .await;
    assert!(matches!(timeout, Err(CommsError::NoReplier)));
}

#[tokio::test]
async fn channel_state_late_joiner() {
    let (service, client) = ChannelBackend::pair();
    let service_session = Session::with_channel(service);
    let client_session = Session::with_channel(client);
    let state = service_session
        .declare_state("blueos/v1/svc/state")
        .await
        .expect("state");
    state
        .publish(Payload::from_bytes(Bytes::from_static(b"v1")), "text/plain")
        .await
        .expect("publish state");
    let reply = client_session
        .query(
            "blueos/v1/svc/state",
            Payload::empty(),
            "",
            Duration::from_secs(1),
        )
        .await
        .expect("late joiner query");
    assert_eq!(reply.payload.as_slice(), b"v1");
}

#[tokio::test]
async fn channel_liveliness() {
    let (left, right) = ChannelBackend::pair();
    let publisher = Session::with_channel(left);
    let subscriber = Session::with_channel(right);
    let mut stream = subscriber
        .subscribe_liveliness("blueos/v1/services/*")
        .await
        .expect("liveliness subscribe");
    let token = publisher
        .declare_liveliness("blueos/v1/services/recorder")
        .await
        .expect("token");
    let event = tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("timeout")
        .expect("event");
    assert_eq!(
        event,
        LivelinessEvent::Put {
            key: "blueos/v1/services/recorder".into(),
        }
    );
    drop(token);
    let delete_event = tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("timeout")
        .expect("delete");
    assert_eq!(
        delete_event,
        LivelinessEvent::Delete {
            key: "blueos/v1/services/recorder".into(),
        }
    );
}

#[tokio::test]
async fn channel_large_payload() {
    let (left, right) = ChannelBackend::pair();
    let publisher = Session::with_channel(left);
    let subscriber = Session::with_channel(right);
    let mut stream = subscriber.subscribe("test/large").await.expect("subscribe");
    let large = vec![0xCD_u8; 4096];
    publisher
        .publish(
            "test/large",
            Payload::from_bytes(Bytes::from(large.clone())),
            "application/octet-stream",
            None,
        )
        .await
        .expect("publish");
    let sample = tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("timeout")
        .expect("sample");
    assert_eq!(sample.payload.as_slice(), large.as_slice());
}
