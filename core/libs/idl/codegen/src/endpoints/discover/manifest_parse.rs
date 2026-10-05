//! Parse `endpoints.toml` into checked endpoints.

use alloc::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use convert_case::{Case, Casing};

use crate::endpoints::{
    Endpoint, InterfaceType, Kind, Manifest, ManifestError, STANDARD_ENDPOINTS,
};

use super::workspace::read;

/// The endpoints of `manifest` in registration order, once every name and interface type is checked.
pub(crate) fn endpoints(
    manifest: Manifest,
    messages: &BTreeSet<String>,
) -> Result<(String, Vec<Endpoint>), ManifestError> {
    let Manifest {
        service,
        job,
        query,
        state,
        event,
    } = manifest;
    check_name(&service)?;
    let raw = job
        .into_iter()
        .map(|(name, entry)| {
            (
                Kind::Job,
                name,
                entry.interface_type,
                entry.custom,
                entry.nature,
            )
        })
        .chain(query.into_iter().map(|(name, entry)| {
            let kind = if entry.io { Kind::IoQuery } else { Kind::Query };
            (kind, name, entry.interface_type, false, None)
        }))
        .chain(
            state
                .into_iter()
                .map(|(name, entry)| (Kind::State, name, entry.interface_type, false, None)),
        )
        .chain(
            event
                .into_iter()
                .map(|(name, entry)| (Kind::Event, name, entry.interface_type, false, None)),
        );
    let mut endpoints = Vec::new();
    for (kind, name, interface_type, custom, nature) in raw {
        endpoints.push(parse_endpoint(ParseEndpointInput {
            kind,
            name,
            interface_type,
            custom,
            nature,
            messages,
        })?);
    }
    duplicate_function_error(&endpoints)?;
    Ok((service, endpoints))
}

struct ParseEndpointInput<'a> {
    kind: Kind,
    name: String,
    interface_type: String,
    custom: bool,
    nature: Option<super::super::types::NatureEntry>,
    messages: &'a BTreeSet<String>,
}

fn parse_endpoint(input: ParseEndpointInput<'_>) -> Result<Endpoint, ManifestError> {
    let ParseEndpointInput {
        kind,
        name,
        interface_type,
        custom,
        nature,
        messages,
    } = input;
    check_name(&name)?;
    let function = name.to_case(Case::Snake);
    if STANDARD_ENDPOINTS
        .iter()
        .any(|standard| standard.name.to_case(Case::Snake) == function)
    {
        return Err(ManifestError::Reserved(name));
    }
    Ok(Endpoint {
        interface: interface(&name, kind, interface_type, messages)?,
        kind,
        name,
        function,
        custom,
        nature,
    })
}

fn duplicate_endpoint_function(
    endpoints: &[Endpoint],
    first_index: usize,
    second_index: usize,
    function: String,
) -> ManifestError {
    ManifestError::Duplicate {
        first: endpoints[first_index].name.clone(),
        second: endpoints[second_index].name.clone(),
        function,
    }
}

fn duplicate_function_error(endpoints: &[Endpoint]) -> Result<(), ManifestError> {
    let mut functions = BTreeMap::<String, usize>::new();
    for (endpoint_index, endpoint) in endpoints.iter().enumerate() {
        for endpoint_function in endpoint.functions() {
            if let Some(&first_index) = functions.get(&endpoint_function) {
                return Err(duplicate_endpoint_function(
                    endpoints,
                    first_index,
                    endpoint_index,
                    endpoint_function,
                ));
            }
            functions.insert(endpoint_function, endpoint_index);
        }
    }
    Ok(())
}

pub(super) fn read_manifest(path: &Path) -> Result<Manifest, crate::endpoints::EndpointsError> {
    let source = read(path)?;
    toml::from_str(&source).map_err(|error| crate::endpoints::EndpointsError::Manifest {
        path: path.to_path_buf(),
        error: ManifestError::Parse(error.to_string()),
    })
}

/// Accepts a name that is an identifier in both its spelling and its snake case.
pub(super) fn check_name(name: &str) -> Result<(), ManifestError> {
    let plain = name.starts_with(|character: char| character.is_ascii_alphabetic())
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_');
    if plain && syn::parse_str::<syn::Ident>(&name.to_case(Case::Snake)).is_ok() {
        Ok(())
    } else {
        Err(ManifestError::InvalidName(name.to_owned()))
    }
}

/// The interface type `interface_type` of the endpoint `endpoint`, once `messages` is known to hold its parts: a
/// `.action` for a Job type, a `.srv` for a Query and a `.msg` for a State or an Event.
pub(super) fn interface(
    endpoint: &str,
    kind: Kind,
    interface_type: String,
    messages: &BTreeSet<String>,
) -> Result<InterfaceType, ManifestError> {
    let (form, expected, part) = match kind {
        Kind::Job => ("action", ".action", "Goal"),
        Kind::Query | Kind::IoQuery => ("srv", ".srv", "Request"),
        Kind::State | Kind::Event => ("msg", ".msg", ""),
    };
    let interface = interface_type
        .split_once(&format!("/{form}/"))
        .map(|(package, name)| InterfaceType {
            package: package.to_owned(),
            form,
            name: name.to_owned(),
        });
    match interface {
        Some(interface) if messages.contains(&interface.part_schema_name(part)) => Ok(interface),
        _ => Err(ManifestError::UnknownType {
            endpoint: endpoint.to_owned(),
            interface_type,
            expected,
        }),
    }
}
