//! Build generated endpoint source strings from one manifest.

use alloc::collections::BTreeSet;

use super::super::{
    Generated, Manifest, ManifestError,
    discover::manifest_parse,
    emit::{api_source, app_source, typescript_client_source},
};

/// Builds the generated endpoint sources for one `endpoints.toml` manifest.
pub fn generate(
    source: &str,
    api_crate: &str,
    messages: &BTreeSet<String>,
) -> Result<Generated, ManifestError> {
    let manifest: Manifest =
        toml::from_str(source).map_err(|error| ManifestError::Parse(error.to_string()))?;
    let (service, endpoint_list) = manifest_parse::endpoints(manifest, messages)?;
    Ok(Generated {
        api: api_source(&service, &endpoint_list),
        app: app_source(&service, api_crate, &endpoint_list),
        typescript: typescript_client_source(&service, &endpoint_list),
        service,
    })
}
