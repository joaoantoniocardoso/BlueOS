use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, serde::Serialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Catalog(pub catalog_core::catalog::Catalog);

impl Deref for Catalog {
    type Target = catalog_core::catalog::Catalog;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<catalog_core::catalog::Catalog> for Catalog {
    fn as_ref(&self) -> &catalog_core::catalog::Catalog {
        &self.0
    }
}

impl Catalog {
    pub fn new() -> Self {
        Self(catalog_core::catalog::Catalog::new())
    }

    pub fn with_parts(
        services: Vec<catalog_model::service::Service>,
        journeys: Vec<catalog_model::journey::UseCase>,
        pages: Vec<catalog_model::page::Page>,
    ) -> Self {
        Self(catalog_core::catalog::Catalog::with_parts(
            services, journeys, pages,
        ))
    }

    pub fn bootstrap() -> Self {
        Self(catalog_core::catalog::Catalog::bootstrap())
    }

    pub fn validate(&self) -> Result<(), Vec<catalog_core::validate::ValidationError>> {
        crate::validate::validate(self)
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
