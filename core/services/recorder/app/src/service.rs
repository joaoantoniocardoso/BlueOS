//! The Recorder Service wiring.

use core::{num::NonZeroU32, sync::atomic::AtomicU8};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::sync::{mpsc, watch};

use blueos_recorder_capture::RecordGate;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest, RecorderSnapshot};
use blueos_recorder_storage::RecordingsFolder;
use blueos_service::{
    Backoff, RestartPolicy, Service, ServiceBuilder, ServiceContext, ServiceError,
};

use crate::{
    cli::RecorderArguments,
    context::{
        DEFAULT_MCAP_WRITER_QUEUE_CAPACITY, IndexQuerySetup, RecorderContext, RepairIoSetup,
        default_index_walker,
    },
    data_plane::run_data_plane,
    endpoints,
    handlers::RecorderHandlers,
    index_io::RECORDING_INDEX_WALK_TIMEOUT,
    io::register_io,
    library_io::run_library_io,
    library_observed::run_library_observed_bridge,
    mavlink::run_mavlink_ingress,
    settings::RecorderSettings,
};

const RECORDER_DURABLE_STATE_VERSION: NonZeroU32 = NonZeroU32::MIN;

/// What the integration-test wiring variants return: the builder, the Context the Kernel takes with it, and a
/// [`RecordGate`] watcher.
type TestWiring = (
    ServiceBuilder<RecorderDomain, RecorderContext>,
    RecorderContext,
    watch::Receiver<RecordGate>,
);

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
            repair_before_rewrite: Arc::new(|_cancel| {}),
            index_walk_timeout: RECORDING_INDEX_WALK_TIMEOUT,
            index_walker: default_index_walker(),
        })
    }

    fn build(
        service: &ServiceContext<RecorderArguments>,
        context: &RecorderContext,
    ) -> Result<ServiceBuilder<RecorderDomain, RecorderContext>, ServiceError> {
        Ok(wire(service, context).0)
    }
}

/// Like [`RecorderService::build`], but also returns the Context and a [`RecordGate`] watcher for integration tests.
pub fn build_with_record_gate(
    service: &ServiceContext<RecorderArguments>,
) -> Result<TestWiring, ServiceError> {
    build_with_record_gate_index_and_repair(
        service,
        IndexQuerySetup::default(),
        RepairIoSetup::default(),
        DEFAULT_MCAP_WRITER_QUEUE_CAPACITY,
    )
}

/// Like [`build_with_record_gate`], with integration-test wiring overrides.
#[doc(hidden)]
pub fn build_with_record_gate_and_index(
    service: &ServiceContext<RecorderArguments>,
    index: IndexQuerySetup,
    mcap_writer_queue_capacity: usize,
) -> Result<TestWiring, ServiceError> {
    build_with_record_gate_index_and_repair(
        service,
        index,
        RepairIoSetup::default(),
        mcap_writer_queue_capacity,
    )
}

/// Like [`build_with_record_gate_and_index`], with a repair-IO hold for tests.
#[doc(hidden)]
pub fn build_with_record_gate_index_and_repair(
    service: &ServiceContext<RecorderArguments>,
    index: IndexQuerySetup,
    repair: RepairIoSetup,
    mcap_writer_queue_capacity: usize,
) -> Result<TestWiring, ServiceError> {
    let mut context = RecorderService::context(service)?;
    context.mcap_writer_queue_capacity = mcap_writer_queue_capacity;
    context.repair_before_rewrite = repair.before_rewrite;
    context.index_walk_timeout = index.walk_timeout;
    context.index_walker = index.walker;
    let (builder, gate_receiver) = wire(service, &context);
    Ok((builder, context, gate_receiver))
}

/// The pure wiring of [`RecorderService::build`], and a watcher of the [`RecordGate`] Projection for the test
/// variants above.
fn wire(
    service: &ServiceContext<RecorderArguments>,
    context: &RecorderContext,
) -> (
    ServiceBuilder<RecorderDomain, RecorderContext>,
    watch::Receiver<RecordGate>,
) {
    let config_parent = service.settings_path().map(PathBuf::from);
    let (mut builder, record_gate) = ServiceBuilder::new(RecorderSnapshot::default())
        .projection(|snapshot: &RecorderSnapshot| snapshot.record_gate());
    builder = builder.durable_state_with_jobs(
        RecorderService::NAME,
        config_parent.clone(),
        RECORDER_DURABLE_STATE_VERSION,
    );
    let gate_receiver = record_gate.subscribe();
    let builder = register_io(
        endpoints::register(
            builder
                .blocking_io(|recorder_context: &RecorderContext, snapshot, request| {
                    run_library_io(recorder_context, snapshot, request)
                })
                .jobs()
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
                    |task_context| async move { run_library_observed_bridge(task_context).await },
                ),
            RecorderHandlers::new(context.clone()),
        )
        .service_metadata(
            RecorderService::VERSION,
            RecorderService::BUILD,
            RecorderService::CAPABILITIES,
        ),
    );
    (builder, gate_receiver)
}
