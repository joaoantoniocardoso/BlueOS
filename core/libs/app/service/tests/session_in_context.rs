//! ServiceContext exposes the backbone session before the Kernel runs.

use std::sync::Arc;

use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError};

#[derive(Clone, clap::Args)]
struct SessionProbeArguments;

struct SessionProbeContext;

struct SessionProbeService;

impl Service for SessionProbeService {
    type Domain = blueos_example_domain::Pump;
    type Context = SessionProbeContext;
    type Arguments = SessionProbeArguments;

    const NAME: &'static str = "session_probe";
    const VERSION: &'static str = "0.0.0";

    fn build(
        context: &ServiceContext<SessionProbeArguments>,
    ) -> Result<ServiceBuilder<Self::Domain, SessionProbeContext>, ServiceError> {
        let _session: &Arc<dyn CommsBackend> = context.session();
        Ok(
            ServiceBuilder::new(blueos_example_domain::PumpSnapshot::default())
                .context(SessionProbeContext),
        )
    }
}

#[test]
fn build_receives_the_backbone_session() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let context = ServiceContext::new(SessionProbeArguments, Arc::clone(&backend));
    let _builder = SessionProbeService::build(&context).expect("build with session");
}
