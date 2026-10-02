//! The Recorder Service wiring.

use core::sync::atomic::AtomicU8;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::sync::watch;

use blueos_recorder_capture::RecordGate;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest, RecorderSnapshot};
use blueos_recorder_storage::RecordingsFolder;
use blueos_service::{RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError};

use crate::{
    cameras_io::register_io, cli::RecorderArguments, context::RecorderContext,
    data_plane::run_data_plane, endpoints, handlers::RecorderHandlers, library_io::run_library_io,
    mavlink::run_mavlink_ingress, settings::RecorderSettings,
};

/// The Recorder Service.
pub struct RecorderService;

impl Service for RecorderService {
    type Domain = RecorderDomain;
    type Context = RecorderContext;
    type Arguments = RecorderArguments;

    const NAME: &'static str = endpoints::NAME;
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn build(
        context: &ServiceContext<RecorderArguments>,
    ) -> Result<ServiceBuilder<RecorderDomain, RecorderContext>, ServiceError> {
        build_with_record_gate(context).map(|(builder, _gate_receiver)| builder)
    }
}

/// Like [`RecorderService::build`], but also returns a [`RecordGate`] watcher for integration tests.
pub fn build_with_record_gate(
    context: &ServiceContext<RecorderArguments>,
) -> Result<
    (
        ServiceBuilder<RecorderDomain, RecorderContext>,
        watch::Receiver<RecordGate>,
    ),
    ServiceError,
> {
    let (builder, gate_receiver) = assemble_builder(context)?;
    Ok((builder, gate_receiver))
}

fn assemble_builder(
    context: &ServiceContext<RecorderArguments>,
) -> Result<
    (
        ServiceBuilder<RecorderDomain, RecorderContext>,
        watch::Receiver<RecordGate>,
    ),
    ServiceError,
> {
    let recordings_folder = Arc::new(
        RecordingsFolder::new(context.arguments().recorder_path.clone())
            .map_err(|error| ServiceError::Build(error.into()))?,
    );
    let config_parent = context.settings_path().map(PathBuf::from);
    let (builder, record_gate) = ServiceBuilder::new(RecorderSnapshot::default())
        .projection(|snapshot: &RecorderSnapshot| snapshot.record_gate());
    let gate_receiver = record_gate.subscribe();
    let builder = register_io(
        endpoints::register(
            builder
                .context(RecorderContext {
                    record_gate,
                    recordings_folder,
                    library_footer_cache: Arc::new(Mutex::new(
                        blueos_recorder_storage::LibraryFooterCache::default(),
                    )),
                    mcap_writer_queue_capacity: context
                        .arguments()
                        .mcap_writer_queue_capacity
                        .unwrap_or(4096),
                    session: Arc::clone(context.session()),
                    mavlink_sequence: Arc::new(AtomicU8::new(0)),
                })
                .blocking_io(|context: &RecorderContext, snapshot, request| {
                    run_library_io(context, snapshot, request)
                })
                .settings(
                    RecorderService::NAME,
                    config_parent,
                    |snapshot: &mut RecorderSnapshot, settings: RecorderSettings| {
                        snapshot.capture.settings = settings.into_capture_settings();
                    },
                    |snapshot: &RecorderSnapshot| {
                        RecorderSettings::from_capture(&snapshot.capture.settings)
                    },
                    |envelope| {
                        let document: RecorderSettings =
                            serde_json::from_str(&envelope.document_json)?;
                        Ok(RecorderRequest::UpdateSettings(
                            document.into_capture_settings(),
                        ))
                    },
                )
                .on_start(RecorderRequest::Startup)
                .task(
                    "data_plane",
                    RestartPolicy::Always,
                    |task_context| async move { run_data_plane(task_context).await },
                )
                .task(
                    "mavlink",
                    RestartPolicy::Always,
                    |task_context| async move { run_mavlink_ingress(task_context).await },
                ),
            RecorderHandlers,
        )
        .service_metadata(
            RecorderService::VERSION,
            RecorderService::BUILD,
            RecorderService::CAPABILITIES,
        ),
    );
    Ok((builder, gate_receiver))
}
