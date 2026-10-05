//! Declares the settings state endpoint when the builder registered settings.

use std::sync::{Arc, Mutex};

use tokio::sync::watch;

use blueos_api::settings_key;
use blueos_comms::CommsBackend;
use blueos_domain::Domain;

use crate::{builder::ServiceBuilder, service::ServiceError, settings::settings_encoding};

use super::super::super::{
    endpoints::declare,
    types::{Kernel, SettingsEndpoint},
};

pub(super) struct SettingsBootPrepared<D: Domain> {
    pub settings: Option<SettingsEndpoint<D>>,
    pub pending_settings_serve: Option<(
        blueos_comms::Queryable,
        String,
        String,
        watch::Receiver<Option<bytes::Bytes>>,
    )>,
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    pub(super) async fn declare_settings_endpoint_at_boot(
        service: &'static str,
        backend: Arc<dyn CommsBackend>,
        builder: &mut ServiceBuilder<D, Context>,
    ) -> Result<SettingsBootPrepared<D>, ServiceError> {
        let mut pending_settings_serve = None;
        let mut settings = None;
        if let Some(registration) = builder.settings.take() {
            let mut driver =
                (registration.start)(service.to_owned(), builder.settings_folder.clone())?;
            driver.load_into(&mut builder.snapshot)?;
            let driver = Arc::new(Mutex::new(driver));
            let key = settings_key(service);
            let queryable = declare(&*backend, key.clone()).await?;
            let encoding = settings_encoding();
            let latest = watch::Sender::new(None);
            pending_settings_serve =
                Some((queryable, key.clone(), encoding.clone(), latest.subscribe()));
            settings = Some(SettingsEndpoint {
                key,
                encoding,
                driver,
                latest,
            });
        }
        Ok(SettingsBootPrepared {
            settings,
            pending_settings_serve,
        })
    }
}
