//! The Recorder Service wiring.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use blueos_recorder_domain::{RecorderDomain, RecorderRequest, RecorderSnapshot};
use blueos_recorder_storage::RecordingsFolder;
use blueos_service::{RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError};

use crate::{
    cli::RecorderArguments, context::RecorderContext, data_plane::run_data_plane, endpoints,
    handlers::RecorderHandlers, library_io::run_library_io, settings::RecorderSettings,
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
        let recordings_folder = Arc::new(
            RecordingsFolder::new(context.arguments().recorder_path.clone())
                .map_err(|error| ServiceError::Build(error.into()))?,
        );
        let config_parent = context.settings_path().map(PathBuf::from);
        let (builder, record_gate) = ServiceBuilder::new(RecorderSnapshot::default())
            .projection(|snapshot: &RecorderSnapshot| snapshot.capture.record_gate());
        Ok(endpoints::register(
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
                })
                .blocking_io(|context: &RecorderContext, snapshot, request| {
                    run_library_io(context, snapshot, request)
                })
                .settings(
                    Self::NAME,
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
                ),
            RecorderHandlers,
        )
        .service_metadata(Self::VERSION, Self::BUILD, Self::CAPABILITIES))
    }
}
