//! ServiceContext exposes the backbone session before the Kernel runs.

use std::sync::Arc;

use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError};

#[derive(Clone, clap::Args)]
struct SessionProbeArguments;

struct SessionProbeContext {
    session: Arc<dyn CommsBackend>,
}

struct SessionProbeService;

impl Service for SessionProbeService {
    type Domain = blueos_example_domain::Pump;
    type Context = SessionProbeContext;
    type Arguments = SessionProbeArguments;

    const NAME: &'static str = "session_probe";
    const VERSION: &'static str = "0.0.0";

    fn context(
        service: &ServiceContext<SessionProbeArguments>,
    ) -> Result<SessionProbeContext, ServiceError> {
        Ok(SessionProbeContext {
            session: Arc::clone(service.session()),
        })
    }

    fn build(
        _service: &ServiceContext<SessionProbeArguments>,
        _context: &SessionProbeContext,
    ) -> Result<ServiceBuilder<Self::Domain, SessionProbeContext>, ServiceError> {
        Ok(ServiceBuilder::new(
            blueos_example_domain::PumpSnapshot::default(),
        ))
    }
}

#[test]
fn context_receives_the_backbone_session() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let service = ServiceContext::new(SessionProbeArguments, Arc::clone(&backend));
    let context = SessionProbeService::context(&service).expect("context with session");
    assert!(Arc::ptr_eq(&context.session, &backend));
}
