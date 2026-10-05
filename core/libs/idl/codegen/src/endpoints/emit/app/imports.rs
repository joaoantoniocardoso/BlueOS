//! Import block for generated app `endpoints.rs`.

use super::import_plan::planned_import_block;

pub(super) struct ImportBlockInput<'a> {
    pub(super) api_crate: &'a str,
    pub(super) domain: &'a str,
    pub(super) endpoints: &'a [crate::endpoints::Endpoint],
    pub(super) handled: &'a [&'a crate::endpoints::Endpoint],
    pub(super) conversions: bool,
    pub(super) packages: &'a alloc::collections::BTreeSet<String>,
}

pub(super) fn import_block(input: ImportBlockInput<'_>) -> String {
    planned_import_block(&input)
}
