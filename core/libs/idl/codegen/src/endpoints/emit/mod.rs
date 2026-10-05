mod api;
mod app;
mod manifest;
mod typescript;

pub(in crate::endpoints) use api::api_source;
pub(in crate::endpoints) use app::app_source;
pub(in crate::endpoints) use typescript::typescript_client_source;
