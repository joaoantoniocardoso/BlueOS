//! Async IO executor registered on the Recorder Service builder.

use std::sync::Arc;

use blueos_domain::IoError;
use blueos_recorder_domain::{RecorderDomain, RecorderIoRequest, RecorderSnapshot};
use blueos_recorder_library::LibraryIoRequest;
use blueos_service::ServiceBuilder;

use crate::{
    cameras_io,
    context::RecorderContext,
    library_io::{run_cancel_repair_io, run_library_repair_io},
};

/// Registers the Service's single async IO executor (cameras egress and library repair).
pub(crate) fn register_io(
    builder: ServiceBuilder<RecorderDomain, RecorderContext>,
) -> ServiceBuilder<RecorderDomain, RecorderContext> {
    builder.io(
        |context: &RecorderContext, _snapshot: &RecorderSnapshot, request| {
            let session = Arc::clone(&context.session);
            let mavlink_sequence = Arc::clone(&context.mavlink_sequence);
            let folder = Arc::clone(&context.recordings_folder);
            let progress_sender = context.library_observed_sender.clone();
            let cancel_flags = Arc::clone(&context.repair_cancel_flags);
            async move {
                match request {
                    RecorderIoRequest::Cameras(request) => {
                        cameras_io::publish_cameras_request(session, mavlink_sequence, request)
                            .await
                    }
                    RecorderIoRequest::Library(LibraryIoRequest::Repair { path }) => {
                        run_library_repair_io(folder, progress_sender, cancel_flags, path).await
                    }
                    RecorderIoRequest::Library(LibraryIoRequest::CancelRepair { path }) => {
                        run_cancel_repair_io(&cancel_flags, &path)
                    }
                    RecorderIoRequest::Library(_) => Err(IoError::new(
                        "library scan and delete use the blocking executor",
                    )),
                }
            }
        },
    )
}
