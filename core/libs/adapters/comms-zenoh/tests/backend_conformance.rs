//! L5 backend conformance against a `zenohd` router.

#[path = "../../comms/tests/support/conformance_body.rs"]
mod conformance;

use std::sync::Arc;

use blueos_comms::CommsBackend;
use blueos_comms_zenoh::ZenohBackend;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn zenoh_backend_conformance() {
    let Some(endpoint) = std::env::var("BLUEOS_ZENOH_ENDPOINT").ok() else {
        eprintln!(
            "skip: set BLUEOS_ZENOH_ENDPOINT (for example tcp/127.0.0.1:7447) to run zenoh conformance"
        );
        return;
    };
    let backend: Arc<dyn CommsBackend> = Arc::new(
        ZenohBackend::connect(&endpoint)
            .await
            .unwrap_or_else(|error| panic!("could not connect to zenohd at {endpoint}: {error}")),
    );
    conformance::run_all(&|| Arc::clone(&backend)).await;
}
