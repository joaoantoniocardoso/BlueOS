//! Declares every boot endpoint and builds the Kernel that will serve them.

use std::sync::Arc;

use tokio::{sync::watch, task::JoinSet};

use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::{UpdateSettingsFeedback, UpdateSettingsResult};
use blueos_jobs::Jobs;

use crate::{
    command_sender::Session,
    durable_state::{DurablePersister, DurableStateHandle},
    service::ServiceError,
    shutdown::IoInflight,
};

use super::{
    super::{
        super::{
            timers::TimerWheel,
            types::{Kernel, UPDATE_SETTINGS},
        },
        commands::declare_command_endpoints,
        request::KernelBootRequest,
        spawn::BootSpawnPlan,
    },
    BootDeclared,
};

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    // qual:allow(complexity, max_function_lines=120) reason: "one literal per Kernel and BootSpawnPlan field; splitting it only moves the fields into carrier structs"
    pub(super) async fn declare_boot(
        request: KernelBootRequest<D, Context>,
    ) -> Result<BootDeclared<D, Context>, ServiceError> {
        let KernelBootRequest {
            service,
            builder,
            context,
            backend,
            clock,
            #[cfg(feature = "testing")]
            effect_log,
        } = request;
        let builder = builder
            .job_feedback(UPDATE_SETTINGS, |_, _| None::<UpdateSettingsFeedback>)
            .job_result(UPDATE_SETTINGS, |_, _| UpdateSettingsResult::default());
        let (mut builder, durable) =
            Self::prepare_durable_state_at_boot(service, Arc::clone(&clock), builder)?;
        let inbox = Self::prepare_inbox_and_service_info(service, &builder);
        let core =
            Self::declare_core_service_queryables(service, Arc::clone(&backend), &builder, &inbox)
                .await?;
        let task_specs = core::mem::take(&mut builder.tasks);
        let settings =
            Self::declare_settings_endpoint_at_boot(service, Arc::clone(&backend), &mut builder)
                .await?;
        let (pending_commands, goal_decoders) = declare_command_endpoints(
            service,
            backend.as_ref(),
            core::mem::take(&mut builder.commands),
        )
        .await?;
        let jobs = Self::declare_job_endpoints_at_boot(
            service,
            Arc::clone(&backend),
            &mut builder,
            inbox.job_type_names,
            pending_commands,
        )
        .await?;
        let queries = Self::declare_state_and_query_endpoints_at_boot(
            service,
            Arc::clone(&backend),
            &mut builder,
        )
        .await?;
        let spawn_plan = BootSpawnPlan {
            info_queryable: core.info_queryable,
            info_key: core.info_key,
            info_payload: core.info_payload,
            info_encoding: core.info_encoding,
            status_queryable: core.status_queryable,
            status_key: core.status_key.clone(),
            status_encoding: core.status_encoding.clone(),
            status_latest: core.status_latest.subscribe(),
            metrics_queryable: core.metrics_queryable,
            metrics_key: core.metrics_key,
            pending_settings_serve: settings.pending_settings_serve,
            update_settings_queryable: core.update_settings_queryable,
            pending_commands: jobs.pending_commands,
            jobs_queryable: jobs.jobs_queryable,
            pending_job_outputs: jobs.pending_job_outputs,
            pending_states: queries.pending_states,
            pending_queries: queries.pending_queries,
            pending_io_queries: queries.pending_io_queries,
        };
        let session: Session = Arc::clone(&backend);
        let kernel = Self {
            service,
            snapshot: builder.snapshot,
            inbox: inbox.inbox,
            inbox_sender: Some(inbox.inbox_sender),
            states: queries.states,
            settings: settings.settings,
            durable: durable
                .durable_registration
                .map(|registration| DurableStateHandle {
                    persister: DurablePersister::spawn(Arc::clone(&clock), registration.store),
                    serialize: registration.serialize,
                    changed: registration.changed,
                }),
            events: builder.events,
            goal_decoders,
            jobs_access: builder.jobs,
            own_jobs: Jobs::default(),
            jobs_latest: jobs.jobs_latest,
            job_outputs: jobs.job_outputs,
            backend,
            clock,
            timers: TimerWheel::new(),
            context: Arc::new(context),
            io: builder.io,
            snapshot_for_queries: inbox.snapshot_for_queries,
            #[cfg(feature = "testing")]
            effect_log,
            endpoints: JoinSet::new(),
            shutdown_request: durable.shutdown_request,
            shutdown_receiver: builder.shutdown_receiver,
            io_inflight: IoInflight::new(),
            shutting_down: false,
            tasks: core.task_supervisor,
            projections: queries.projections,
            log_publisher: None,
            metrics: builder.metrics,
            metrics_latest: watch::Sender::new(None),
            inbox_step_time: core.inbox_step_time,
            inbox_depth: core.inbox_depth,
            runtime_gauges: core.runtime_gauges,
        };
        kernel.projections.refresh(&kernel.snapshot);
        Ok(BootDeclared {
            kernel,
            startup_commands: durable.startup_commands,
            task_specs,
            session,
            service,
            status_key: core.status_key,
            status_encoding: core.status_encoding,
            status_latest: core.status_latest,
            spawn_plan,
        })
    }
}
