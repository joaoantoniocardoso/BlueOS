//! Command endpoint declaration during Kernel startup.

use std::{collections::HashMap, sync::Arc};

use blueos_api::{Message, command_key};
use blueos_comms::{CommsBackend, Queryable};
use blueos_domain::Domain;
use blueos_idl::msg::blueos_msgs::PermissionAnswer;
use blueos_jobs::JobControl;

use crate::{
    builder::{CommandEndpoint, Decode},
    inbox::Input,
    service::ServiceError,
};

use super::super::{
    endpoints::declare,
    types::{CONTROLS, IntoInput, Rejection},
};

pub(crate) async fn declare_command_endpoints<D: Domain>(
    service: &str,
    backend: &dyn CommsBackend,
    commands: Vec<CommandEndpoint<D>>,
) -> Result<(Vec<(Queryable, IntoInput<D>)>, HashMap<String, Decode<D>>), ServiceError> {
    let mut pending = Vec::new();
    let mut goal_decoders = HashMap::new();
    for command in commands {
        let (queryable, into_input, name, decode) =
            declare_one_command(service, backend, command).await?;
        pending.push((queryable, into_input));
        goal_decoders.insert(name, decode);
    }
    Ok((pending, goal_decoders))
}

async fn declare_one_command<D: Domain>(
    service: &str,
    backend: &dyn CommsBackend,
    command: CommandEndpoint<D>,
) -> Result<(Queryable, IntoInput<D>, String, Decode<D>), ServiceError> {
    let queryable = declare(backend, command_key(service, &command.name)).await?;
    let job_type_name = command.name.clone();
    let job_type = Arc::new(job_type_name.clone());
    let decode = Arc::clone(&command.decode);
    let nature = command.nature;
    let into_input: IntoInput<D> = Box::new(move |job_id, goal| {
        let request = decode(job_id, &goal)?;
        Ok(Input::Submit {
            job_id,
            job_type: job_type.as_ref().clone(),
            goal,
            nature,
            request,
        })
    });
    Ok((queryable, into_input, job_type_name, command.decode))
}

pub(crate) async fn declare_control_endpoints<D: Domain>(
    service: &str,
    backend: &dyn CommsBackend,
) -> Result<Vec<(Queryable, IntoInput<D>)>, ServiceError> {
    let mut pending = Vec::new();
    for control in CONTROLS {
        let queryable = declare(backend, command_key(service, &control.to_string())).await?;
        let into_input: IntoInput<D> = Box::new(move |job_id, body| {
            let control = match control {
                JobControl::AnswerPermission { .. } => JobControl::AnswerPermission {
                    granted: PermissionAnswer::decode(&body)
                        .map_err(Rejection::InvalidBody)?
                        .granted,
                },
                JobControl::Cancel | JobControl::Pause | JobControl::Resume => control,
            };
            Ok(Input::Control { job_id, control })
        });
        pending.push((queryable, into_input));
    }
    Ok(pending)
}
