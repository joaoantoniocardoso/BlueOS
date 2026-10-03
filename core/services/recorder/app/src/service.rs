//! The Recorder Service wiring.

use core::{num::NonZeroU32, sync::atomic::AtomicU8};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;

use blueos_recorder_domain::{RecorderDomain, RecorderRequest, RecorderSnapshot};
use blueos_recorder_mcap::{rewrite, walk_index};
use blueos_recorder_storage::RecordingsFolder;
use blueos_service::{
    Backoff, RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError,
};

use crate::{
    capture::tasks::data_plane::run_data_plane,
    cli::RecorderArguments,
    context::{DEFAULT_MCAP_WRITER_QUEUE_CAPACITY, RecorderContext},
    endpoints,
    io::register_io,
    library::{
        handlers::{RECORDING_INDEX_WALK_TIMEOUT, RecorderHandlers},
        io::run_library_io,
        tasks::observed::run_library_observed_bridge,
    },
    settings::RecorderSettings,
    tasks::mavlink::run_mavlink_ingress,
};

const RECORDER_DURABLE_STATE_VERSION: NonZeroU32 = NonZeroU32::MIN;

/// The Recorder Service.
pub struct RecorderService;

impl Service for RecorderService {
    type Domain = RecorderDomain;
    type Context = RecorderContext;
    type Arguments = RecorderArguments;

    const NAME: &'static str = endpoints::NAME;
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn context(
        service: &ServiceContext<RecorderArguments>,
    ) -> Result<RecorderContext, ServiceError> {
        let recordings_folder = Arc::new(
            RecordingsFolder::new(service.arguments().recorder_path.clone())
                .map_err(|error| ServiceError::Build(error.into()))?,
        );
        let (observed_sender, observed_receiver) = mpsc::channel(64);
        Ok(RecorderContext {
            recordings_folder,
            library_footer_cache: Arc::new(Mutex::new(
                blueos_recorder_storage::LibraryFooterCache::default(),
            )),
            mcap_writer_queue_capacity: DEFAULT_MCAP_WRITER_QUEUE_CAPACITY,
            session: Arc::clone(service.session()),
            mavlink_sequence: Arc::new(AtomicU8::new(0)),
            library_observed_sender: observed_sender,
            library_observed_receiver: Arc::new(tokio::sync::Mutex::new(observed_receiver)),
            repair_cancel_flags: Arc::new(Mutex::new(BTreeMap::new())),
            rewriter: Arc::new(rewrite),
            index_walk_timeout: RECORDING_INDEX_WALK_TIMEOUT,
            index_walker: Arc::new(walk_index),
        })
    }

    fn build(
        service: &ServiceContext<RecorderArguments>,
        context: &RecorderContext,
    ) -> Result<ServiceBuilder<RecorderDomain, RecorderContext>, ServiceError> {
        let config_parent = service.settings_path().map(PathBuf::from);
        let (mut builder, record_gate) = ServiceBuilder::new(RecorderSnapshot::default())
            .projection(|snapshot: &RecorderSnapshot| snapshot.record_gate());
        builder = builder.durable_state_with_jobs(
            RecorderService::NAME,
            config_parent.clone(),
            RECORDER_DURABLE_STATE_VERSION,
        );
        Ok(
            register_io(
                endpoints::register(
                    builder
                        .blocking_io(|recorder_context: &RecorderContext, snapshot, request| {
                            run_library_io(recorder_context, snapshot, request)
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
                            RestartPolicy::Always {
                                backoff: Backoff::default(),
                            },
                            move |task_context| run_data_plane(task_context, record_gate.clone()),
                        )
                        .task(
                            "mavlink",
                            RestartPolicy::Always {
                                backoff: Backoff::default(),
                            },
                            |task_context| async move { run_mavlink_ingress(task_context).await },
                        )
                        .task(
                            "library_observed",
                            RestartPolicy::Always {
                                backoff: Backoff::default(),
                            },
                            |task_context| async move {
                                run_library_observed_bridge(task_context).await
                            },
                        ),
                    RecorderHandlers::new(context.clone()),
                )
                .service_metadata(
                    RecorderService::VERSION,
                    RecorderService::BUILD,
                    RecorderService::CAPABILITIES,
                ),
            ),
        )
    }
}
