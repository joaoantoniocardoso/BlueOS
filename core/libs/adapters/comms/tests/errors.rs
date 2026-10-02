//! A backend error keeps the error the backend reported.

use core::error::Error;
use std::io;

use blueos_comms::CommsError;

#[test]
fn a_backend_error_keeps_the_error_it_wraps_as_its_source() {
    let error = CommsError::backend(io::Error::from(io::ErrorKind::ConnectionRefused));

    let source = error
        .source()
        .and_then(|source| source.downcast_ref::<io::Error>())
        .unwrap();
    assert_eq!(source.kind(), io::ErrorKind::ConnectionRefused);
}
