use catalog_core::validate::{validate as validate_core, ValidationError};

use crate::catalog::Catalog;
use crate::validate_extensions::{
    check_functions, check_harness_annotation_stubs, check_requirements,
};

pub fn validate(catalog: &Catalog) -> Result<(), Vec<ValidationError>> {
    let mut errors = validate_core(catalog.as_ref()).err().unwrap_or_default();
    errors.extend(check_harness_annotation_stubs(catalog));
    errors.extend(check_requirements(catalog));
    errors.extend(check_functions(catalog));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
