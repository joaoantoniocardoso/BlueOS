//! Pure import-line fragments for generated app `endpoints.rs`.

use crate::endpoints::Kind;

use super::imports::ImportBlockInput;

pub(super) fn planned_import_lines(input: &ImportBlockInput<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    lines.extend(future_import(input));
    lines.extend(handled_imports(input));
    lines.extend(domain_import(input));
    lines.extend(message_import(input));
    lines.extend(jobs_import(input));
    lines.push(service_import(input));
    lines.extend(conversions_import(input));
    lines
}

fn future_import(input: &ImportBlockInput<'_>) -> Vec<String> {
    if input
        .handled
        .iter()
        .any(|endpoint| endpoint.kind == Kind::IoQuery)
    {
        vec!["use core::future::Future;".to_owned()]
    } else {
        Vec::new()
    }
}

fn handled_imports(input: &ImportBlockInput<'_>) -> Vec<String> {
    if input.handled.is_empty() {
        return Vec::new();
    }
    vec!["use std::sync::Arc;".to_owned(), String::new()]
}

fn domain_import(input: &ImportBlockInput<'_>) -> Vec<String> {
    if input.handled.is_empty() && input.conversions {
        return Vec::new();
    }
    vec![format!("use blueos_domain::{};", input.domain)]
}

fn message_import(input: &ImportBlockInput<'_>) -> Vec<String> {
    if input.packages.is_empty() {
        return Vec::new();
    }
    let joined = input
        .packages
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    vec![format!("use blueos_idl::msg::{{{joined}}};")]
}

fn jobs_import(input: &ImportBlockInput<'_>) -> Vec<String> {
    if !input
        .endpoints
        .iter()
        .any(|endpoint| endpoint.nature.is_some())
    {
        return Vec::new();
    }
    let job_id = if input
        .handled
        .iter()
        .any(|endpoint| endpoint.nature.is_some())
    {
        "JobId, "
    } else {
        ""
    };
    vec![format!(
        "use blueos_jobs::{{DomainJobs, {job_id}JobNature}};"
    )]
}

fn service_import(input: &ImportBlockInput<'_>) -> String {
    let refusal = if !input.handled.is_empty()
        || input
            .endpoints
            .iter()
            .any(|endpoint| endpoint.kind == Kind::Job && !endpoint.custom)
    {
        "Refusal, "
    } else {
        ""
    };
    format!("use blueos_service::{{{refusal}ServiceBuilder}};")
}

fn conversions_import(input: &ImportBlockInput<'_>) -> Vec<String> {
    if !input.conversions {
        return Vec::new();
    }
    vec![format!("use {}::endpoints::Conversions;", input.api_crate)]
}
