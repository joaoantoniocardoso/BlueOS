//! A payload keeps the buffer it was made from.

use bytes::Bytes;

use blueos_comms::Payload;

#[test]
fn an_encoded_message_becomes_a_payload_without_a_copy() {
    let encoded = vec![0, 1, 0, 0, 7];
    let encoded_address = encoded.as_ptr();

    let payload = Payload::from(encoded);

    assert_eq!(payload.to_bytes().as_ptr(), encoded_address);
}

#[test]
fn a_backend_gets_its_own_buffer_back_without_a_copy() {
    let received = Bytes::from_static(b"received");

    let payload = Payload::from(Bytes::clone(&received));

    assert_eq!(
        payload.downcast_ref::<Bytes>().unwrap().as_ptr(),
        received.as_ptr()
    );
}
