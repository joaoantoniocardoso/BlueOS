//! Async IO executor registered on the Recorder Service builder.

use std::sync::Arc;

use blueos_domain::IoError;
use blueos_recorder_domain::{RecorderDomain, RecorderIoRequest, RecorderSnapshot};
use blueos_service::ServiceBuilder;

use crate::{cameras, context::RecorderContext};

/// Registers the Service's single async IO executor (cameras egress).
pub(crate) fn register_io(
    builder: ServiceBuilder<RecorderDomain, RecorderContext>,
) -> ServiceBuilder<RecorderDomain, RecorderContext> {
    builder.io(
        |context: &RecorderContext, _snapshot: &RecorderSnapshot, request| {
            let session = Arc::clone(&context.session);
            let mavlink_sequence = Arc::clone(&context.mavlink_sequence);
            async move {
                match request {
                    RecorderIoRequest::Cameras(request) => {
                        cameras::io::publish_cameras_request(session, mavlink_sequence, request)
                            .await
                    }
                    RecorderIoRequest::Library(_) => Err(IoError::new(
                        "library scan and delete use the blocking executor",
                    )),
                }
            }
        },
    )
}
