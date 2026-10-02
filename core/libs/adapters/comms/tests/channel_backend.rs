//! The comms contract on the in-process channel backend.

#[path = "support/conformance_body.rs"]
mod conformance;

use std::sync::Arc;

use blueos_comms::{CommsBackend, channel::ChannelBackend};

#[tokio::test]
async fn channel_backend_conformance() {
    conformance::run_all(&|| Arc::new(ChannelBackend::default()) as Arc<dyn CommsBackend>).await;
}
